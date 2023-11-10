
#[macro_use]
extern crate rocket;

use std::collections::HashSet;
use database::get_tree_from_node_id;
use models::ApiProjectConfigListResponseResult;
use mongodb::bson::doc;
use rocket::{http::Method, Config, serde::json::Json, figment::Figment};


mod constants;
mod database;
mod errors;
mod helpers;
mod models;
mod auth;
mod expression_evaluator;
mod history;

#[cfg(test)]
mod tests;

pub struct CORS;

#[rocket::async_trait]
impl rocket::fairing::Fairing for CORS {
    fn info(&self) -> rocket::fairing::Info {
        rocket::fairing::Info {
            name: "Add CORS headers to responses",
            kind: rocket::fairing::Kind::Response,
        }
    }

    async fn on_response<'r>(&self, request: &'r rocket::Request<'_>, response: &mut rocket::Response<'r>) {
        if request.method() == Method::Options {
            response.set_status(rocket::http::Status::NoContent);
            response.set_header(rocket::http::Header::new(
                "Access-Control-Allow-Methods",
                "POST, PUT, PATCH, GET, DELETE",
            ));
            response.set_header(rocket::http::Header::new("Access-Control-Allow-Headers", "*"));
        }

        response.set_header(rocket::http::Header::new(
            "Access-Control-Allow-Origin",
            "*",
        ));
        response.set_header(rocket::http::Header::new("Access-Control-Allow-Credentials", "true"));
    }
}


#[get("/")]
async fn index() -> &'static str {
    let db_client = database::get_instance().await;
    match db_client {
        Ok(client) => {
            "Hello, world!"
        }
        Err(e) => "Failed to create user",
    }
}

#[get("/auth/login?<code>&<state>&<scope>")]
async fn auth_login_get(code: Option<String>, state: Option<String>, scope: Option<String>) -> Json<models::ApiAuthLoginResponse> {
    let db_client = database::get_instance().await;
    match db_client {
        Ok(client) => {
            // Validate flow

            if state.is_none() || code.is_none()  {
                Json(models::ApiAuthLoginResponse {
                    ok: false,
                    message: "Expected a state and code parameter".to_owned(),
                    result: None,
                })
            } else {
                match database::validate_csrf_token(&state.expect("Asserted"), &client).await {
                    Ok(nonce) => {
                        println!("Start trade");
                        let email = auth::trade_token(&code.as_ref().expect("Asserted"), nonce).await;
                        println!("End trade");
                        match email {
                            Ok(email) => {
                                // Create user if user does not exist
                                let tenant = database::get_tenant_for_user_email(&client, email.clone()).await;

                                match tenant {
                                    Some(tenant) => {
                                        let user_exists = database::get_user(&client, tenant, email.clone()).await;
                                        match user_exists {
                                            Some(user) => {},
                                            None => {
                                                database::new_user(&client, email.clone());
                                                ()
                                            }
                                        }
                                    },
                                    None => {
                                        database::new_user(&client, email.clone());
                                    }
                                }


                                // Generate JWT
                                let session_token = auth::generate_user_jwt(&email);
                                match session_token {
                                    Ok(session_token) => Json(models::ApiAuthLoginResponse {
                                        ok: true,
                                        message: "Logged in".to_owned(),
                                        result: Some(models::AuthLoginResponseResult {
                                            sessionToken: session_token,
                                            loginRequest: "".to_owned()
                                        }),
                                    }),
                                    Err(err) => Json(models::ApiAuthLoginResponse {
                                        ok: false,
                                        message: "Token generation failed".to_owned(),
                                        result: None,
                                    })
                                }
                            },
                            Err(err) => {
                                eprintln!("{}", err);
                                Json(models::ApiAuthLoginResponse {
                                    ok: false,
                                    message: "Login Validation failed".to_owned(),
                                    result: None,
                                })
                            }
                        }
                    },
                    Err(err) => Json(models::ApiAuthLoginResponse {
                        ok: false,
                        message: "CSRF Validation failed".to_owned(),
                        result: None,
                    })
                }
            }
        }
        
        Err(e) => {
            eprintln!("{}", e);
            Json(models::ApiAuthLoginResponse {
                ok: false,
                message: "Could not connect to DB".to_owned(),
                result: None,
            })
        },
    }
}

#[post("/auth/login?<provider>")]
async fn auth_login_post(provider: Option<String>) -> Json<models::ApiAuthLoginResponse> {
    let db_client = database::get_instance().await;
    match db_client {
        Ok(client) => {
            // Start flow
            let start_data = auth::start_flow();
            match start_data {
                Ok(start_data) => {
                    // Store csrf_token for lookup later
                    match database::store_csrf_token(&start_data.csrf_token, &start_data.nonce, &client).await {
                        Ok(_) => Json(models::ApiAuthLoginResponse {
                            ok: true,
                            message: "Got request URI".to_owned(),
                            result: Some(models::AuthLoginResponseResult {
                                sessionToken: "".to_owned(),
                                loginRequest: start_data.url.to_string()
                            }),
                        }),
                        Err(err) => {
                            eprintln!("{}", err);
                            Json(models::ApiAuthLoginResponse {
                                ok: false,
                                message: "CSRF generation failed.".to_owned(),
                                result: None,
                            })
                        }
                    }

                },
                Err(err) => {
                    eprintln!("{}", err);
                    Json(models::ApiAuthLoginResponse {
                        ok: false,
                        message: "Could not generate OAuth request URL".to_owned(),
                        result: None,
                    })
                }
            }
        }
        
        Err(e) => {
            eprintln!("{}", e);
            Json(models::ApiAuthLoginResponse {
                ok: false,
                message: "Could not connect to DB".to_owned(),
                result: None,
            })
        },
    }
}

#[post("/auth/logout")]
fn auth_logout() -> &'static str {
    "todo"
}

#[get("/projects")]
async fn projects_get(key: auth::ApiKey) -> Json<models::ApiProjectsListResponse> {
    if key.tenant.is_none() {
        Json(models::ApiProjectsListResponse {
            ok: false,
            message: "Could not find a tenant".to_owned(),
            result: None,
        })
    } else {
        let db_client = database::get_instance().await;
        match db_client {
            Ok(client) => {
                let ids = database::get_available_project_ids(&client, key.tenant.clone().expect("checked")).await;
                match ids {
                    Ok(ids) => Json(models::ApiProjectsListResponse {
                        ok: true,
                        message: "Got projects".to_owned(),
                        result: Some(models::ApiProjectsListResponseResult {
                            projects: database::get_projects_from_ids(&client, key.tenant.expect("checked"), ids).await
                        }),
                    }),
                    Err(err) => Json(models::ApiProjectsListResponse {
                        ok: false,
                        message: "Failed to find project ids".to_owned(),
                        result: None,
                    })
                }
            }
            Err(e) => Json(models::ApiProjectsListResponse {
                ok: false,
                message: "Could not connect to DB".to_owned(),
                result: None,
            }),
        }
    }
}

#[post("/projects", data = "<body>")]
async fn projects_post(body: Json<models::ApiCreateProject>, key: auth::ApiKey) -> Json<models::ApiCreateProjectResponse> {
    if key.tenant.is_none() {
        Json(models::ApiCreateProjectResponse {
            ok: false,
            message: "Could not find a tenant".to_owned(),
            result: None,
        })
    } else {
        let db_client = database::get_instance().await;
        match db_client {
            Ok(client) => {
                let project = database::get_project_by_title(client.to_owned(), key.tenant.clone().expect("checked"), body.title.to_owned()).await;
                match project {
                    Some(project) => Json(models::ApiCreateProjectResponse {
                        ok: true,
                        message: "Tree already exists".to_owned(),
                        result: None,
                    }),
                    None => match database::new_project(client, key.tenant.expect("checked"), body.title.to_owned()).await {
                        Ok(new_project_id) => Json(models::ApiCreateProjectResponse {
                            ok: true,
                            message: "Project created succesfully".to_owned(),
                            result: Some(models::CreateProjectResponseResult {
                                title: body.title.to_owned(),
                                id: new_project_id,
                            }),
                        }),
                        Err(err) => Json(models::ApiCreateProjectResponse {
                            ok: false,
                            message: format!("Failed to create project: {}", err),
                            result: None,
                        }),
                    },
                }
            }
            Err(e) => Json(models::ApiCreateProjectResponse {
                ok: false,
                message: "Could not connect to DB".to_owned(),
                result: None,
            }),
        }
    }

}

#[put("/projects/<id>", data = "<body>")]
async fn projects_put(id: String, body: Json<models::ApiCreateProject>, key: auth::ApiKey) -> Json<models::ApiCreateProjectResponse> {
    if key.tenant.is_none() {
        Json(models::ApiCreateProjectResponse {
            ok: false,
            message: "Could not find a tenant".to_owned(),
            result: None,
        })
    } else {
        let db_client = database::get_instance().await;
        match db_client {
            Ok(client) => {
                let mut project = database::get_project_by_id(&client, key.tenant.clone().expect("checked"), id).await;
    
                match project {
                    Some(mut project) => {
                        project.title = body.title.clone();
    
                        let updated_project = database::update_project(client, key.tenant.expect("checked"), &project).await;
                        match updated_project {
                            Ok(proj) => {
                                Json(models::ApiCreateProjectResponse {
                                    ok: true,
                                    message: "Project updated".to_owned(),
                                    result: Some(models::CreateProjectResponseResult {
                                        title: body.title.to_owned(),
                                        id: proj.id,
                                    })
                                })
                            },
                            Err(err) => {
                                Json(models::ApiCreateProjectResponse {
                                    ok: false,
                                    message: format!(
                                        "Failed to update project: {}",
                                        err
                                    ),
                                    result: None,
                                })
                            }
                        }
                    },
                    None => Json(models::ApiCreateProjectResponse {
                        ok: false,
                        message: "Project not found".to_owned(),
                        result: None,
                    })
                }
            }
            Err(e) => Json(models::ApiCreateProjectResponse {
                ok: false,
                message: "Could not connect to DB".to_owned(),
                result: None,
            }),
        }
    }

}

#[post("/projects/<id>/trees", data = "<body>")]
async fn projects_trees_post(
    id: String,
    body: Json<models::ApiCreateTree>,
    key: auth::ApiKey,
) -> Json<models::ApiCreateTreeResponse> {
    if key.tenant.is_none() {
        Json(models::ApiCreateTreeResponse {
            ok: false,
            message: "Could not find a tenant".to_owned(),
            result: None,
        })
    } else {
        let db_client = database::get_instance().await;
        match db_client {
            Ok(client) => {
                let project = database::get_project_by_id(&client, key.tenant.clone().expect("checked"), id.to_owned()).await;
                match project {
                    Some(project) => {
                        // Create tree
                        match database::create_project_tree(
                            client.to_owned(),
                            key.tenant.expect("checked"),
                            body.title.to_owned(),
                            id,
                        ).await {
                            Ok(db_res) => Json(models::ApiCreateTreeResponse {
                                ok: true,
                                message: "Added tree".to_owned(),
                                result: Some(models::CreateTreeResponseResult {
                                    title: body.title.to_owned(),
                                    id: db_res,
                                }),
                            }),
                            Err(err) => Json(models::ApiCreateTreeResponse {
                                ok: false,
                                message: format!(
                                    "Failed to create new tree and link to project: {}",
                                    err
                                ),
                                result: None,
                            }),
                        }
                    }
                    None => Json(models::ApiCreateTreeResponse {
                        ok: true,
                        message: "Could not find project".to_owned(),
                        result: None,
                    }),
                }
            }
            Err(e) => Json(models::ApiCreateTreeResponse {
                ok: false,
                message: "Could not connect to DB".to_owned(),
                result: None,
            }),
        }    
    }

}

#[get("/projects/<id>/trees")]
async fn projects_trees_get(id: String, key: auth::ApiKey) -> Json<models::ApiListTreeResponse> {
    if key.tenant.is_none() {
        Json(models::ApiListTreeResponse {
            ok: false,
            message: "Could not find a tenant".to_owned(),
            result: None,
        })
    } else {
        let db_client = database::get_instance().await;
        match db_client {
            Ok(client) => {
                let project: Option<models::Project> =
                    database::get_project_by_id(&client, key.tenant.clone().expect("checked"), id.to_owned()).await;
                match project {
                    Some(project) => {
                        // Get trees
                        let trees = database::get_trees_by_project_id(&client, key.tenant.expect("checked"), project.id).await;
    
                        match trees {
                            Ok(trees) => Json(models::ApiListTreeResponse {
                                ok: true,
                                message: "Got trees succesfully.".to_owned(),
                                result: Some(models::ListTreeResponseResult { trees }),
                            }),
                            Err(_) => Json(models::ApiListTreeResponse {
                                ok: false,
                                message: "Failed to get projects from db".to_owned(),
                                result: None,
                            }),
                        }
                    }
                    None => Json(models::ApiListTreeResponse {
                        ok: false,
                        message: "Could not find project".to_owned(),
                        result: None,
                    }),
                }
            }
            Err(e) => Json(models::ApiListTreeResponse {
                ok: false,
                message: "Could not connect to DB".to_owned(),
                result: None,
            }),
        }
    
    }
}

#[get("/projects/<id>/trees/<tree_id>")]
async fn projects_trees_tree_get(id: String, tree_id: String, key: auth::ApiKey) -> Json<models::ApiTreeComputedResponse> {
    if key.tenant.is_none() {
        Json(models::ApiTreeComputedResponse {
            ok: false,
            message: "Could not find a tenant".to_owned(),
            result: None,
        })
    } else {
        let db_client = database::get_instance().await;
        match db_client {
            Ok(client) => {
                let project: Option<models::Project> =
                    database::get_project_by_id(&client, key.tenant.clone().expect("checked"), id.to_owned()).await;
                match project {
                    Some(project) => {
                        let tree = database::get_tree_by_id(&client, key.tenant.expect("checked"), tree_id.to_owned(), id.to_owned()).await;
                        match tree {
                            Ok(tree) => {
                                Json(models::ApiTreeComputedResponse {
                                    ok: true,
                                    message: "Found tree".to_owned(),
                                    result: Some(tree),
                                })
                            },
                            Err(err) => {
                                Json(models::ApiTreeComputedResponse {
                                    ok: false,
                                    message: "Could not find tree using id".to_owned(),
                                    result: None,
                                })
                            }
                        }
    
                    }
                    None => Json(models::ApiTreeComputedResponse {
                        ok: false,
                        message: "Could not find project".to_owned(),
                        result: None,
                    }),
                }
            }
            Err(e) => Json(models::ApiTreeComputedResponse {
                ok: false,
                message: "Could not connect to DB".to_owned(),
                result: None,
            }),
        }
    }
}

#[put("/projects/<id>/trees/<tree_id>", data = "<body>")]
async fn projects_trees_tree_put(id: String, tree_id: String, body: Json<models::ApiFullTreeData>, key: auth::ApiKey) -> Json<models::ApiTreeComputedResponse> {
    if key.tenant.is_none() {
        Json(models::ApiTreeComputedResponse {
            ok: false,
            message: "Could not find a tenant".to_owned(),
            result: None,
        })
    } else {
        let db_client = database::get_instance().await;
        match db_client {
            Ok(client) => {
                let project: Option<models::Project> =
                    database::get_project_by_id(&client, key.tenant.clone().expect("checked"), id.to_owned()).await;
                match project {
                    Some(project) => {
                        let tenant = key.tenant.expect("checked");
                        let title = body.title.to_owned();
                        let root_node_id = body.rootNodeId.to_owned();
                        let nodes = body.nodes.clone();

                        // Save current tree state for undo
                        history::record_tree_update(&client, tenant.to_owned(), tree_id.clone(), body.into_inner()).await;

                        // Update tree and return
                        let tree = database::update_tree_by_id(&client, tenant, tree_id.to_owned(), id.to_owned(), models::ApiFullTreeData {
                            title: title,
                            rootNodeId: root_node_id,
                            nodes: nodes
                        }).await;
                        match tree {
                            Ok(tree) => {
                                Json(models::ApiTreeComputedResponse {
                                    ok: true,
                                    message: "Found tree".to_owned(),
                                    result: Some(tree),
                                })
                            },
                            Err(err) => {
                                Json(models::ApiTreeComputedResponse {
                                    ok: false,
                                    message: "Could not find tree using id".to_owned(),
                                    result: None,
                                })
                            }
                        }
    
                    }
                    None => Json(models::ApiTreeComputedResponse {
                        ok: false,
                        message: "Could not find project".to_owned(),
                        result: None,
                    }),
                }
            }
            Err(e) => Json(models::ApiTreeComputedResponse {
                ok: false,
                message: "Could not connect to DB".to_owned(),
                result: None,
            }),
        }    
    }
}

#[put("/projects/<id>/trees/<tree_id>/undo")]
async fn projects_trees_tree_undo_put(id: String, tree_id: String, key: auth::ApiKey) -> Json<models::ApiTreeComputedResponse> {
    if key.tenant.is_none() {
        Json(models::ApiTreeComputedResponse {
            ok: false,
            message: "Could not find a tenant".to_owned(),
            result: None,
        })
    } else {
        let db_client = database::get_instance().await;
        match db_client {
            Ok(client) => {
                match history::move_back_tree_update(&client, key.tenant.expect("checked"), tree_id, id).await {
                    Some(tree) => {
                        Json(models::ApiTreeComputedResponse {
                            ok: true,
                            message: "Found tree".to_owned(),
                            result: Some(tree),
                        })
                    },
                    None => {
                        Json(models::ApiTreeComputedResponse {
                            ok: false,
                            message: "Nothing to undo".to_owned(),
                            result: None,
                        })
                    }
                }
            }
            Err(e) => Json(models::ApiTreeComputedResponse {
                ok: false,
                message: "Could not connect to DB".to_owned(),
                result: None,
            }),
        }    
    }
}

#[get("/projects/<id>/model")]
async fn projects_model_get(id: String, key: auth::ApiKey) -> Json<models::ApiSelectedModelResponse> {
    if key.tenant.is_none() {
        Json(models::ApiSelectedModelResponse {
            ok: false,
            message: "Could not find a tenant".to_owned(),
            result: None,
        })
    } else {
        let db_client = database::get_instance().await;
        match db_client {
            Ok(client) => {
                let project: Option<models::Project> =
                    database::get_project_by_id(&client, key.tenant.expect("checked"), id.to_owned()).await;
                match project {
                    Some(project) => {
                        Json(models::ApiSelectedModelResponse {
                            ok: true,
                            message: "Found tree".to_owned(),
                            result: Some(models::SelectedModelResult {
                                modelId: project.selected_model.unwrap_or_default(),
                            }),
                        })
                    
                    }
                    None => Json(models::ApiSelectedModelResponse {
                        ok: false,
                        message: "Could not find project".to_owned(),
                        result: None,
                    }),
                }
            }
            Err(e) => Json(models::ApiSelectedModelResponse {
                ok: false,
                message: "Could not connect to DB".to_owned(),
                result: None,
            }),
        }    
    }

}

#[put("/projects/<id>/model", data = "<body>")]
async fn projects_model_put(id: String, body: Json<models::SelectedModelResult>, key: auth::ApiKey) -> Json<models::ApiSelectedModelResponse> {
    if key.tenant.is_none() {
        Json(models::ApiSelectedModelResponse {
            ok: false,
            message: "Could not find a tenant".to_owned(),
            result: None,
        })
    } else {
        let db_client = database::get_instance().await;
        match db_client {
            Ok(client) => {
                let project: Option<models::Project> =
                    database::get_project_by_id(&client, key.tenant.clone().expect("checked"), id.to_owned()).await;
                match project {
                    Some(project) => {
                        match database::update_project_model(client, key.tenant.expect("checked"), id.to_owned(), body.modelId.to_owned()).await {
                            Ok(_) => {
                                Json(models::ApiSelectedModelResponse {
                                    ok: true,
                                    message: "Updated project".to_owned(),
                                    result: Some(models::SelectedModelResult {
                                        modelId: body.modelId.to_owned(),
                                    }),
                                })
                            },
                            Err(err) => {
                                Json(models::ApiSelectedModelResponse {
                                    ok: false,
                                    message: "Could not update project".to_owned(),
                                    result: None,
                                })
                            }
                        }
                    
                    }
                    None => Json(models::ApiSelectedModelResponse {
                        ok: false,
                        message: "Could not find project".to_owned(),
                        result: None,
                    }),
                }
            }
            Err(e) => Json(models::ApiSelectedModelResponse {
                ok: false,
                message: "Could not connect to DB".to_owned(),
                result: None,
            }),
        }    
    }
}

#[get("/models")]
async fn models_get(key: auth::ApiKey) -> Json<models::ApiListModelResponse> {
    if key.tenant.is_none() {
        Json(models::ApiListModelResponse {
            ok: false,
            message: "Could not find a tenant".to_owned(),
            result: None,
        })
    } else {
        let db_client = database::get_instance().await;

        let model_list = vec![
            models::ListModelResponseItem {
                id: "b9ff54e0-37cf-41d4-80ea-f3a9b1e3af74".to_owned(), // V4 UUID
                title: "Attacker Likelihood".to_owned(),
            },
            models::ListModelResponseItem {
                id: "f1644cb9-b2a5-4abb-813f-98d0277e42f2".to_owned(), // V4 UUID
                title: "Risk of Attack".to_owned(),
            },
            models::ListModelResponseItem {
                id: "bf4397f7-93ae-4502-a4a2-397f40f5cc49".to_owned(), // V4 UUID
                title: "EVITA".to_owned(),
            }
        ];
    
        match db_client {
            Ok(client) => {
                Json(models::ApiListModelResponse {
                    ok: true,
                    message: "Got models".to_owned(),
                    result: Some(models::ListModelResult { 
                        models: model_list
                    }),
                })
            }
            Err(e) => Json(models::ApiListModelResponse {
                ok: false,
                message: "Could not connect to DB".to_owned(),
                result: None,
            }),
        }
    
    }
    
}

#[get("/nodes/<id>")]
async fn node_get(id: String, key: auth::ApiKey) -> Json<models::ApiGetNodeResponse> {
    if key.tenant.is_none() {
        Json(models::ApiGetNodeResponse {
            ok: false,
            message: "Could not find a tenant".to_owned(),
            result: None,
        })
    } else {
        let db_client = database::get_instance().await;

        match db_client {
            Ok(client) => {
                match get_tree_from_node_id(&client, key.tenant.expect("checked"), id).await {
                    Ok(res) => Json(res),
                    Err(err) => Json(models::ApiGetNodeResponse {
                        ok: false,
                        message: "Could not find node".to_owned(),
                        result: None,
                    })
                }
            },
            Err(err) => Json(models::ApiGetNodeResponse {
                ok: false,
                message: "Could not connect to DB".to_owned(),
                result: None,
            })
        }
    
    }

}

#[get("/projects/<projectId>/trees/<treeId>/dag/down")]
async fn projects_trees_tree_dag_down_get(projectId: String, treeId: String, key: auth::ApiKey) -> Json<models::ApiTreeDagResponse> {
    if key.tenant.is_none() {
        Json(models::ApiTreeDagResponse {
            ok: false,
            message: "Could not find a tenant".to_owned(),
            result: None,
        })
    } else {
        let db_client = database::get_instance().await;

        match db_client {
            Ok(client) => {
                match database::get_tree_by_id(&client, key.tenant.clone().expect("checked"), treeId.clone(), projectId.clone()).await {
                    Ok(tree_data) => {
                        let result = models::ApiTreeDagResponseResult {
                            root: models::ApiTreeDagItem {
                                id: treeId.clone(),
                                title: tree_data.title,
                                children: database::get_tree_relationships_down(&client, key.tenant.expect("checked"), &treeId, &projectId).await
                            }
                        };
            
                        Json(models::ApiTreeDagResponse {
                            ok: true,
                            message: "Got relationship".to_string(),
                            result: Some(result)
                        })
                    },
                    Err(err) => {
                        Json(models::ApiTreeDagResponse {
                            ok: false,
                            message: "Could not find tree".to_owned(),
                            result: None,
                        })
                    }
                }


            },
            Err(err) => Json(models::ApiTreeDagResponse {
                ok: false,
                message: "Could not connect to DB".to_owned(),
                result: None,
            })
        }
    
    }

}

#[get("/projects/<projectId>/configs")]
async fn projects_configs_list(projectId: String, key: auth::ApiKey) -> Json<models::ApiProjectConfigListResponse> {
    if key.tenant.is_none() {
        Json(models::ApiProjectConfigListResponse {
            ok: false,
            message: "Could not find a tenant".to_owned(),
            result: None,
        })
    } else {
        let db_client = database::get_instance().await;

        match db_client {
            Ok(client) => {
                let matching_configs = database::get_configs_for_project(&client, key.tenant.expect("checked"), &projectId).await;
    
                Json(models::ApiProjectConfigListResponse {
                    ok: true,
                    message: "Got configs".to_string(),
                    result: Some(ApiProjectConfigListResponseResult {
                        ids: matching_configs
                    })
                })
            },
            Err(err) => Json(models::ApiProjectConfigListResponse {
                ok: false,
                message: "Could not connect to DB".to_owned(),
                result: None,
            })
        }    
    }

}

#[post("/projects/<projectId>/configs", data = "<body>")]
async fn projects_configs_post(projectId: String, body: Json<models::ApiProjectConfigPayload>, key: auth::ApiKey) -> Json<models::ApiProjectConfigResponse> {
    if key.tenant.is_none() {
        Json(models::ApiProjectConfigResponse {
            ok: false,
            message: "Could not find a tenant".to_owned(),
            result: None,
        })
    } else {
        let db_client = database::get_instance().await;

        match db_client {
            Ok(client) => {
                let thing = body.into_inner();
    
                let new_config_id: Result<String, errors::DatabaseError> = database::new_config(&client, key.tenant.expect("checked"), &projectId, &thing).await;
                match new_config_id {
                    Ok(new_config_id) => {
                        Json(models::ApiProjectConfigResponse {
                            ok: true,
                            message: "Created config".to_owned(),
                            result: Some(models::ApiProjectConfigResponseResult {
                                id: new_config_id,
                                attributes: serde_json::json!(thing)
                            })
                        })
                    },
                    Err(err) => Json(models::ApiProjectConfigResponse {
                        ok: false,
                        message: "Creation of config failed".to_owned(),
                        result: None,
                    })
                }
            },
            Err(err) => Json(models::ApiProjectConfigResponse {
                ok: false,
                message: "Could not connect to DB".to_owned(),
                result: None,
            })
        }    
    }

}

#[put("/projects/<projectId>/configs/<configId>", data = "<body>")]
async fn projects_configs_put(projectId: String, configId: String, body: Json<models::ApiProjectConfigPayload>, key: auth::ApiKey) -> Json<models::ApiProjectConfigResponse> {
    if key.tenant.is_none() {
        Json(models::ApiProjectConfigResponse {
            ok: false,
            message: "Could not find a tenant".to_owned(),
            result: None,
        })
    } else {
        let db_client = database::get_instance().await;

        match db_client {
            Ok(client) => {
                let thing = body.into_inner();
                let new_config = database::update_config(&client, key.tenant.expect("checked"), &projectId, &configId, &thing).await;
    
                match new_config {
                    Ok(updated_id) => {
                        Json(models::ApiProjectConfigResponse {
                            ok: true,
                            message: "Updated config".to_owned(),
                            result: Some(models::ApiProjectConfigResponseResult {
                                id: updated_id,
                                attributes: serde_json::json!(thing)
                            }),
                        })
                    },
                    Err(err) => {
                        Json(models::ApiProjectConfigResponse {
                            ok: false,
                            message: "Update config failed".to_owned(),
                            result: None,
                        })
                    }
                }
            },
            Err(err) => Json(models::ApiProjectConfigResponse {
                ok: false,
                message: "Could not connect to DB".to_owned(),
                result: None,
            })
        }    
    }

}

#[get("/projects/<projectId>/configs/<configId>")]
async fn projects_configs_get(projectId: String, configId: String, key: auth::ApiKey) -> Json<models::ApiProjectConfigResponse> {
    if key.tenant.is_none() {
        Json(models::ApiProjectConfigResponse {
            ok: false,
            message: "Could not find a tenant".to_owned(),
            result: None,
        })
    } else {
        let db_client = database::get_instance().await;

        match db_client {
            Ok(client) => {
                let config = database::get_config(&client, key.tenant.expect("checked"), &projectId, &configId).await;
    
                match config {
                    Ok(resp) => {
                        Json(models::ApiProjectConfigResponse {
                            ok: true,
                            message: "Got config".to_owned(),
                            result: Some(resp),
                        })
                    },
                    Err(err) => {
                        Json(models::ApiProjectConfigResponse {
                            ok: false,
                            message: "Get config failed".to_owned(),
                            result: None,
                        })
                    }
                }
            },
            Err(err) => Json(models::ApiProjectConfigResponse {
                ok: false,
                message: "Could not connect to DB".to_owned(),
                result: None,
            })
        }    
    }

}

#[get("/projects/<projectId>/config")]
async fn projects_config_get(projectId: String, key: auth::ApiKey) -> Json<models::ApiProjectConfigResponse> {
    if key.tenant.is_none() {
        Json(models::ApiProjectConfigResponse {
            ok: false,
            message: "Could not find a tenant".to_owned(),
            result: None,
        })
    } else {
        let db_client = database::get_instance().await;

        match db_client {
            Ok(client) => {
                let config = database::get_selected_config(&client, key.tenant.expect("checked"), &projectId).await;
    
                match config {
                    Ok(config) => {
                        Json(models::ApiProjectConfigResponse {
                            ok: true,
                            message: "Found selected config".to_owned(),
                            result: Some(config),
                        })
                    },
                    Err(err) => Json(models::ApiProjectConfigResponse {
                        ok: false,
                        message: "Error finding selected config".to_owned(),
                        result: None,
                    })
                }
            },
            Err(err) => Json(models::ApiProjectConfigResponse {
                ok: false,
                message: "Could not connect to DB".to_owned(),
                result: None,
            })
        }
    }
}

#[put("/projects/<projectId>/config", data = "<body>")]
async fn projects_config_put(projectId: String, body: Json<models::ApiProjectConfigIdPayload>, key: auth::ApiKey) -> Json<models::ApiProjectConfigResponse> {
    if key.tenant.is_none() {
        Json(models::ApiProjectConfigResponse {
            ok: false,
            message: "Could not find a tenant".to_owned(),
            result: None,
        })
    } else {
        let db_client = database::get_instance().await;

        match db_client {
            Ok(client) => {
                let thing: models::ApiProjectConfigIdPayload = body.into_inner();
                let res = database::update_project_selected_config( &client, key.tenant.expect("checked"), &projectId, &thing).await;
    
                match res {
                    Ok(res) => {
                        Json(models::ApiProjectConfigResponse {
                            ok: true,
                            message: "Found selected config".to_owned(),
                            result: Some(res),
                        })
                    },
                    Err(err) => {
                        eprintln!("{}", err);
                        Json(models::ApiProjectConfigResponse {
                            ok: false,
                            message: "Error updating config".to_owned(),
                            result: None,
                        })
                    }
                }
            },
            Err(err) => Json(models::ApiProjectConfigResponse {
                ok: false,
                message: "Could not connect to DB".to_owned(),
                result: None,
            })
        }    
    }

}

#[post("/orgs", data = "<body>")]
async fn orgs_post(body: Json<models::ApiOrgMetadataBase>, key: auth::ApiKey) -> Json<models::ApiOrgResponse> {
    if key.tenant.is_none() {
        Json(models::ApiOrgResponse {
            ok: false,
            message: "Could not find a tenant".to_owned(),
            result: None,
        })
    } else {
        let db_client = database::get_instance().await;

        match db_client {
            Ok(client) => {
                let thing = body.into_inner();
                let res = database::create_org( &client, key.tenant.expect("checked"), &thing).await;

            },
            Err(err) => Json(models::ApiOrgResponse {
                ok: false,
                message: "Could not connect to DB".to_owned(),
                result: None,
            })
        }    
    }

}

#[launch]
async fn rocket() -> _ {
    rocket::build()
        .mount(
            "/",
            routes![
                index,
                auth_login_get,
                auth_login_post,
                auth_logout,
                projects_get,
                projects_post,
                projects_put,
                projects_trees_post,
                projects_trees_get,
                projects_trees_tree_get,
                projects_trees_tree_put,
                projects_trees_tree_undo_put,
                projects_trees_tree_dag_down_get,
                projects_model_get,
                projects_model_put,
                projects_configs_list,
                projects_configs_post,
                projects_configs_put,
                projects_configs_get,
                projects_config_get,
                projects_config_put,
                models_get,
                node_get
            ],
        )
        .attach(CORS)
}
