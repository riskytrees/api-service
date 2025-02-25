
#[macro_use]
extern crate rocket;

use std::collections::HashSet;
use database::{get_org_id_from_tenant, get_project_by_id, get_publicity_for_tree_by_id, get_tree_from_node_id, set_publicity_for_tree_by_id};
use models::{ApiProjectConfigListResponseResult, AuthPersonalTokenResponseResult};
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
                    Ok(validation_result) => {
                        println!("Start trade");
                        let email = auth::trade_token(&code.as_ref().expect("Asserted"), validation_result).await;
                        println!("End trade");
                        match email {
                            Ok(email) => {
                                // Generate JWT
                                let start = std::time::SystemTime::now();
                                let since_the_epoch = start
                                    .duration_since(std::time::UNIX_EPOCH)
                                    .expect("Time went backwards").as_secs() + 604800;
                                let session_token = auth::generate_user_jwt(&email, since_the_epoch, None);
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
async fn auth_login_post(mut provider: Option<String>) -> Json<models::ApiAuthLoginResponse> {
    if provider.is_none() {
        provider = Some("google".to_string());
    }
    
    let db_client = database::get_instance().await;
    match db_client {
        Ok(client) => {
            // Start flow
            let start_data = auth::start_flow(provider.clone().expect("Asserted"));
            match start_data {
                Ok(start_data) => {
                    // Store csrf_token for lookup later
                    match database::store_csrf_token(&start_data.csrf_token, &start_data.nonce, &client, provider.clone().expect("Condition handled above")).await {
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
    if key.email == "" {
        Json(models::ApiProjectsListResponse {
            ok: false,
            message: "Could not find a tenant".to_owned(),
            result: None,
        })
    } else {
        let db_client = database::get_instance().await;
        match db_client {
            Ok(client) => {
                let ids = database::get_available_project_ids(&client, key.tenants.clone()).await;
                println!("Project IDs: {:?}", ids);
                match ids {
                    Ok(ids) => Json(models::ApiProjectsListResponse {
                        ok: true,
                        message: "Got projects".to_owned(),
                        result: Some(models::ApiProjectsListResponseResult {
                            projects: database::get_projects_from_ids(&client, key.tenants, ids).await
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

#[post("/auth/personal/tokens", data = "<body>")]
async fn auth_personal_tokens_post(body: Json<models::ApiCreateAuthPersonalToken>, key: auth::ApiKey) -> Json<models::ApiAuthPersonalTokenResponse> {
    if key.email == "" {
        Json(models::ApiAuthPersonalTokenResponse {
            ok: false,
            message: "Could not find a tenant".to_owned(),
            result: None,
        })
    } else {
        let db_client = database::get_instance().await;
        match db_client {
            Ok(client) => {
                let token = database::generate_token_for_user(&client, &key.email, body.expiresInDays).await;

                match token {
                    Ok(token) => {
                        Json(models::ApiAuthPersonalTokenResponse {
                            ok: true,
                            message: "Token created succesfully.".to_owned(),
                            result: Some(token)
                        })
                    },
                    Err(err) => Json(models::ApiAuthPersonalTokenResponse {
                        ok: false,
                        message: "Generation of token failed".to_owned(),
                        result: None,
                    })
                }


            }
            Err(e) => Json(models::ApiAuthPersonalTokenResponse {
                ok: false,
                message: "Could not connect to DB".to_owned(),
                result: None,
            }),
        }
    }

}

#[post("/projects", data = "<body>")]
async fn projects_post(body: Json<models::ApiCreateProject>, key: auth::ApiKey) -> Json<models::ApiCreateProjectResponse> {
    if key.email == "" {
        Json(models::ApiCreateProjectResponse {
            ok: false,
            message: "Could not find a tenant".to_owned(),
            result: None,
        })
    } else {
        let db_client = database::get_instance().await;
        match db_client {
            Ok(client) => {
                let project = database::get_project_by_title(client.to_owned(), crate::database::Tenant { name: key.email.clone() }, body.title.to_owned()).await;
                match project {
                    Some(project) => Json(models::ApiCreateProjectResponse {
                        ok: true,
                        message: "Project already exists".to_owned(),
                        result: None,
                    }),
                    None => match database::new_project(client, key.email, key.tenants, body.title.to_owned(), body.orgId.to_owned()).await {
                        Ok(new_project_id) => Json(models::ApiCreateProjectResponse {
                            ok: true,
                            message: "Project created successfully".to_owned(),
                            result: Some(models::CreateProjectResponseResult {
                                title: body.title.to_owned(),
                                id: new_project_id,
                                orgId: body.orgId.to_owned()
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

#[get("/projects/<id>")]
async fn project_get(id: String, key: auth::ApiKey) -> Json<models::ApiGetProjectResponse> {
    if key.email == "" {
        Json(models::ApiGetProjectResponse {
            ok: false,
            message: "Could not find a tenant".to_owned(),
            result: None,
        })
    } else {
        let db_client = database::get_instance().await;
        match db_client {
            Ok(client) => {
                let ids = database::get_available_project_ids(&client, key.tenants.clone()).await;
                println!("Project IDs: {:?}", ids);
                match ids {
                    Ok(ids) => {
                        if ids.contains(&id) {
                            let project_tenant = database::filter_tenant_for_project(&client, key.tenants.clone(), id.clone()).await.unwrap_or(database::Tenant {name: key.email.clone( )});
                            let project_data = get_project_by_id(&client, project_tenant.clone(), id.clone()).await.expect("Already checked");
                            Json(models::ApiGetProjectResponse {
                                ok: true,
                                message: "Found project".to_owned(),
                                result: Some(models::ApiProjectsListProjectItem {
                                    projectId: id,
                                    name: project_data.title,
                                    orgId:  get_org_id_from_tenant(&client, &project_tenant).await
                                }),
                            })
                        } else {
                            Json(models::ApiGetProjectResponse {
                                ok: false,
                                message: "Could not find project".to_owned(),
                                result: None,
                            })
                        }
                    },
                    Err(err) => Json(models::ApiGetProjectResponse {
                        ok: false,
                        message: "Failed to find project ids".to_owned(),
                        result: None,
                    })
                }
            }
            Err(e) => Json(models::ApiGetProjectResponse {
                ok: false,
                message: "Could not connect to DB".to_owned(),
                result: None,
            }),
        }
    }
}

#[put("/projects/<id>", data = "<body>")]
async fn projects_put(id: String, body: Json<models::ApiCreateProject>, key: auth::ApiKey) -> Json<models::ApiCreateProjectResponse> {
    if key.email == "" {
        Json(models::ApiCreateProjectResponse {
            ok: false,
            message: "Could not find a tenant".to_owned(),
            result: None,
        })
    } else {
        let db_client = database::get_instance().await;
        match db_client {
            Ok(client) => {
                let mut project = database::get_project_by_id(&client, database::filter_tenant_for_project(&client, key.tenants.clone(), id.clone()).await.unwrap_or(database::Tenant {name: key.email.clone( )}), id.clone()).await;
    
                match project {
                    Some(mut project) => {
                        project.title = body.title.clone();
    
                        let updated_project = database::update_project(client.clone(), database::filter_tenant_for_project(&client, key.tenants.clone(), id.clone()).await.unwrap_or(database::Tenant {name: key.email.clone( )}), &project).await;
                        match updated_project {
                            Ok(proj) => {
                                Json(models::ApiCreateProjectResponse {
                                    ok: true,
                                    message: "Project updated".to_owned(),
                                    result: Some(models::CreateProjectResponseResult {
                                        title: body.title.to_owned(),
                                        id: proj.id,
                                        orgId: get_org_id_from_tenant(&client, &database::filter_tenant_for_project(&client, key.tenants.clone(), id.clone()).await.unwrap_or(database::Tenant {name: key.email.clone( )})).await
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

#[delete("/projects/<id>")]
async fn projects_delete(id: String, key: auth::ApiKey) -> Json<models::ApiResponse> {
    if key.email == "" {
        Json(models::ApiResponse {
            ok: false,
            message: "Could not find a tenant".to_owned(),
            result: None,
        })
    } else {
        let db_client = database::get_instance().await;
        match db_client {
            Ok(client) => {
                let tenant = database::filter_tenant_for_project(&client, key.tenants.clone(), id.clone()).await.unwrap_or(database::Tenant {name: key.email.clone( )});
                let mut project = database::get_project_by_id(&client, database::filter_tenant_for_project(&client, key.tenants.clone(), id.clone()).await.unwrap_or(database::Tenant {name: key.email.clone( )}), id.clone()).await;
    
                match project {
                    Some(mut project) => {
                        match database::get_trees_by_project_id(&client, tenant, project.id.clone()).await {
                            Ok(res) => {
                                // Delete trees before deleting project
                                for tree in res {
                                    database::delete_tree_by_id(&client, key.tenants.clone(), tree.id).await;
                                }

                                // Delete project
                                match database::delete_project_by_id(&client, key.tenants.clone(), project.id.clone()).await {
                                    Ok(res) => {
                                        Json(models::ApiResponse {
                                            ok: true,
                                            message: "Deleted project and trees".to_owned(),
                                            result: None,
                                        })
                                    },
                                    Err(err) => {
                                        Json(models::ApiResponse {
                                            ok: false,
                                            message: "Error deleting project".to_owned(),
                                            result: None,
                                        })
                                    }
                                }
                            },
                            Err(err) => {
                                Json(models::ApiResponse {
                                    ok: false,
                                    message: "Could not lookup trees but should have been able to".to_owned(),
                                    result: None,
                                })
                            }
                        }
                    },
                    None => Json(models::ApiResponse {
                        ok: false,
                        message: "Project not found".to_owned(),
                        result: None,
                    })
                }
            }
            Err(e) => Json(models::ApiResponse {
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
    if key.email == "" {
        Json(models::ApiCreateTreeResponse {
            ok: false,
            message: "Could not find a tenant".to_owned(),
            result: None,
        })
    } else {
        let db_client = database::get_instance().await;
        match db_client {
            Ok(client) => {
                let project = database::get_project_by_id(&client, database::filter_tenant_for_project(&client, key.tenants.clone(), id.clone()).await.unwrap_or(database::Tenant {name: key.email.clone( )}), id.clone()).await;
                match project {
                    Some(project) => {
                        // Create tree
                        match database::create_project_tree(
                            client.to_owned(),
                            database::filter_tenant_for_project(&client, key.tenants.clone(), id.clone()).await.unwrap_or(database::Tenant {name: key.email.clone( )}),
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
    if key.email == "" {
        Json(models::ApiListTreeResponse {
            ok: false,
            message: "Could not find a tenant".to_owned(),
            result: None,
        })
    } else {
        let db_client = database::get_instance().await;
        match db_client {
            Ok(client) => {
                let project = database::get_project_by_id(&client, database::filter_tenant_for_project(&client, key.tenants.clone(), id.clone()).await.unwrap_or(database::Tenant {name: key.email.clone( )}), id.clone()).await;

                match project {
                    Some(project) => {
                        // Get trees
                        let trees = database::get_trees_by_project_id(&client, database::filter_tenant_for_project(&client, key.tenants.clone(), id.clone()).await.unwrap_or(database::Tenant {name: key.email.clone( )}), project.id).await;
    
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

#[get("/projects/<id>/trees/<tree_id>?<config_id>")]
async fn projects_trees_tree_get(id: String, tree_id: String, key: auth::ApiKey, config_id: Option<String>) -> Json<models::ApiTreeComputedResponse> {
    if key.email == "" {
        Json(models::ApiTreeComputedResponse {
            ok: false,
            message: "Could not find a tenant".to_owned(),
            result: None,
        })
    } else {
        let db_client = database::get_instance().await;
        match db_client {
            Ok(client) => {
                // Tenancy applies by default, but if this tree is public, override the tenant
                let tree: Result<models::ApiFullComputedTreeData, errors::DatabaseError> = match get_publicity_for_tree_by_id(&client, tree_id.clone()).await {
                    Ok(res) => {
                        match res {
                            true => database::get_tree_by_id_with_config(&client, database::get_tenant_for_tree(&client, &tree_id.clone()).await.expect("Always exists"), tree_id, id.clone(), config_id).await,
                            false => database::get_tree_by_id_with_config(&client, database::filter_tenant_for_project(&client, key.tenants.clone(), id.clone()).await.unwrap_or(database::Tenant {name: key.email.clone( )}), tree_id.to_owned(), id.to_owned(), config_id).await        
                        }
                    },
                    Err(err) => {
                        Err(err)
                    }
                };

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
    if key.email.clone() == "" {
        Json(models::ApiTreeComputedResponse {
            ok: false,
            message: "Could not find a tenant".to_owned(),
            result: None,
        })
    } else {
        let db_client = database::get_instance().await;
        match db_client {
            Ok(client) => {
                let project = database::get_project_by_id(&client, database::filter_tenant_for_project(&client, key.tenants.clone(), id.clone()).await.unwrap_or(database::Tenant {name: key.email.clone( )}), id.clone()).await;

                match project {
                    Some(project) => {
                        let tenant = database::filter_tenant_for_project(&client, key.tenants.clone(), id.clone()).await.unwrap_or(database::Tenant {name: key.email.clone( )});
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

#[delete("/projects/<id>/trees/<tree_id>")]
async fn projects_trees_tree_delete(id: String, tree_id: String, key: auth::ApiKey) -> Json<models::ApiResponse> {
    if key.email.clone() == "" {
        Json(models::ApiResponse {
            ok: false,
            message: "Could not find a tenant".to_owned(),
            result: None,
        })
    } else {
        let db_client = database::get_instance().await;
        match db_client {
            Ok(client) => {
                let project = database::get_project_by_id(&client, database::filter_tenant_for_project(&client, key.tenants.clone(), id.clone()).await.unwrap_or(database::Tenant {name: key.email.clone( )}), id.clone()).await;

                match project {
                    Some(project) => {
                        match database::delete_tree_by_id(&client, key.tenants.clone(), tree_id).await {
                            Ok(res) => {
                                Json(models::ApiResponse {
                                    ok: true,
                                    message: "Deleted tree".to_owned(),
                                    result: None
                                })
                            },
                            Err(err) => {
                                eprintln!("{err}");
                                Json(models::ApiResponse {
                                    ok: false,
                                    message: "Error deleting tree".to_owned(),
                                    result: None
                                })
                            }
                        }
    
                    }
                    None => Json(models::ApiResponse {
                        ok: false,
                        message: "Could not find project".to_owned(),
                        result: None,
                    }),
                }
            }
            Err(e) => Json(models::ApiResponse {
                ok: false,
                message: "Could not connect to DB".to_owned(),
                result: None,
            }),
        }    
    }
}



#[put("/projects/<id>/trees/<tree_id>/undo")]
async fn projects_trees_tree_undo_put(id: String, tree_id: String, key: auth::ApiKey) -> Json<models::ApiTreeComputedResponse> {
    if key.email == "" {
        Json(models::ApiTreeComputedResponse {
            ok: false,
            message: "Could not find a tenant".to_owned(),
            result: None,
        })
    } else {
        let db_client = database::get_instance().await;
        match db_client {
            Ok(client) => {
                match history::move_back_tree_update(&client, database::filter_tenant_for_project(&client, key.tenants.clone(), id.clone()).await.unwrap_or(database::Tenant {name: key.email.clone( )}), tree_id, id).await {
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

#[get("/projects/<id>/trees/<tree_id>/public")]
async fn projects_trees_tree_public_get(id: String, tree_id: String, key: auth::ApiKey) -> Json<models::ApiTreePublicityResponse> {
    if key.email == "" {
        Json(models::ApiTreePublicityResponse {
            ok: false,
            message: "Could not find a tenant".to_owned(),
            result: None,
        })
    } else {
        let db_client = database::get_instance().await;
        match db_client {
            Ok(client) => {
                match database::get_publicity_for_tree_by_id(&client, tree_id).await {
                    Ok(res) => {
                        Json(models::ApiTreePublicityResponse {
                            ok: true,
                            message: "Got publicity successfully.".to_owned(),
                            result: Some(models::ApiTreePublicity {
                                isPublic: res
                            })
                        })
                    },
                    Err(err) => {
                        Json(models::ApiTreePublicityResponse {
                            ok: false,
                            message: "Getting publicity failed.".to_owned(),
                            result: None,
                        })
                    }
                }


            }
            Err(e) => Json(models::ApiTreePublicityResponse {
                ok: false,
                message: "Could not connect to DB".to_owned(),
                result: None,
            }),
        }    
    }
}

#[put("/projects/<id>/trees/<tree_id>/public", data = "<body>")]
async fn projects_trees_tree_public_put(id: String, body: Json<models::ApiTreePublicity>, tree_id: String, key: auth::ApiKey) -> Json<models::ApiTreePublicityResponse> {
    if key.email == "" {
        Json(models::ApiTreePublicityResponse {
            ok: false,
            message: "Could not find a tenant".to_owned(),
            result: None,
        })
    } else {
        let db_client = database::get_instance().await;
        match db_client {
            Ok(client) => {
                match set_publicity_for_tree_by_id(&client, key.tenants.clone(), tree_id, body.isPublic).await {
                    Ok(res) => {
                        Json(models::ApiTreePublicityResponse {
                            ok: true,
                            message: "Setting publicity succeeded.".to_owned(),
                            result: Some(models::ApiTreePublicity {
                                isPublic: res
                            })
                        })
                    },
                    Err(err) => {
                        Json(models::ApiTreePublicityResponse {
                            ok: false,
                            message: "Setting publicity failed.".to_owned(),
                            result: None,
                        })
                    }
                }


            }
            Err(e) => Json(models::ApiTreePublicityResponse {
                ok: false,
                message: "Could not connect to DB".to_owned(),
                result: None,
            }),
        }    
    }
}

#[get("/projects/<id>/model")]
async fn projects_model_get(id: String, key: auth::ApiKey) -> Json<models::ApiSelectedModelResponse> {
    if key.email == "" {
        Json(models::ApiSelectedModelResponse {
            ok: false,
            message: "Could not find a tenant".to_owned(),
            result: None,
        })
    } else {
        let db_client = database::get_instance().await;
        match db_client {
            Ok(client) => {
                let project = database::get_project_by_id(&client, database::filter_tenant_for_project(&client, key.tenants.clone(), id.clone()).await.unwrap_or(database::Tenant {name: key.email.clone( )}), id.clone()).await;
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
    if key.email == "" {
        Json(models::ApiSelectedModelResponse {
            ok: false,
            message: "Could not find a tenant".to_owned(),
            result: None,
        })
    } else {
        let db_client = database::get_instance().await;
        match db_client {
            Ok(client) => {
                let project = database::get_project_by_id(&client, database::filter_tenant_for_project(&client, key.tenants.clone(), id.clone()).await.unwrap_or(database::Tenant {name: key.email.clone( )}), id.clone()).await;

                match project {
                    Some(project) => {
                        match database::update_project_model(client.clone(), database::filter_tenant_for_project(&client, key.tenants.clone(), id.clone()).await.unwrap_or(database::Tenant {name: key.email.clone( )}), id.to_owned(), body.modelId.to_owned()).await {
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
    if key.email == "" {
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
    if key.email == "" {
        Json(models::ApiGetNodeResponse {
            ok: false,
            message: "Could not find a tenant".to_owned(),
            result: None,
        })
    } else {
        let db_client = database::get_instance().await;

        match db_client {
            Ok(client) => {
                match get_tree_from_node_id(&client, database::filter_tenant_for_node(&client, key.tenants.clone(), id.clone()).await.unwrap_or(database::Tenant {name: key.email.clone( )}), id).await {
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
    if key.email == "" {
        Json(models::ApiTreeDagResponse {
            ok: false,
            message: "Could not find a tenant".to_owned(),
            result: None,
        })
    } else {
        let db_client = database::get_instance().await;

        match db_client {
            Ok(client) => {
                match database::get_tree_by_id(&client, database::filter_tenant_for_project(&client, key.tenants.clone(), projectId.clone()).await.unwrap_or(database::Tenant {name: key.email.clone( )}), treeId.clone(), projectId.clone()).await {
                    Ok(tree_data) => {
                        let result = models::ApiTreeDagResponseResult {
                            root: models::ApiTreeDagItem {
                                id: treeId.clone(),
                                title: tree_data.title,
                                children: database::get_tree_relationships_down(&client, database::filter_tenant_for_project(&client, key.tenants.clone(), projectId.clone()).await.unwrap_or(database::Tenant {name: key.email.clone( )}), &treeId, &projectId, HashSet::new()).await
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
    if key.email == "" {
        Json(models::ApiProjectConfigListResponse {
            ok: false,
            message: "Could not find a tenant".to_owned(),
            result: None,
        })
    } else {
        let db_client = database::get_instance().await;

        match db_client {
            Ok(client) => {
                let matching_configs = database::get_configs_for_project(&client, database::filter_tenant_for_project(&client, key.tenants.clone(), projectId.clone()).await.unwrap_or(database::Tenant {name: key.email.clone( )}), &projectId).await;
    
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
    if key.email == "" {
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
    
                let new_config_id: Result<String, errors::DatabaseError> = database::new_config(&client, database::filter_tenant_for_project(&client, key.tenants.clone(), projectId.clone()).await.unwrap_or(database::Tenant {name: key.email.clone( )}), &projectId, &thing).await;
                match new_config_id {
                    Ok(new_config_id) => {
                        Json(models::ApiProjectConfigResponse {
                            ok: true,
                            message: "Created config".to_owned(),
                            result: Some(models::ApiProjectConfigResponseResult {
                                id: new_config_id,
                                attributes: serde_json::json!(thing.attributes),
                                name: None
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
    if key.email == "" {
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
                let new_config = database::update_config(&client, database::filter_tenant_for_project(&client, key.tenants.clone(), projectId.clone()).await.unwrap_or(database::Tenant {name: key.email.clone( )}), &projectId, &configId, &thing).await;
    
                match new_config {
                    Ok(updated_id) => {
                        Json(models::ApiProjectConfigResponse {
                            ok: true,
                            message: "Updated config".to_owned(),
                            result: Some(models::ApiProjectConfigResponseResult {
                                id: updated_id,
                                attributes: serde_json::json!(thing.attributes),
                                name: thing.name
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
    if key.email == "" {
        Json(models::ApiProjectConfigResponse {
            ok: false,
            message: "Could not find a tenant".to_owned(),
            result: None,
        })
    } else {
        let db_client = database::get_instance().await;

        match db_client {
            Ok(client) => {
                let config = database::get_config(&client, database::filter_tenant_for_project(&client, key.tenants.clone(), projectId.clone()).await.unwrap_or(database::Tenant {name: key.email.clone( )}), &projectId, &configId).await;
    
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
    if key.email == "" {
        Json(models::ApiProjectConfigResponse {
            ok: false,
            message: "Could not find a tenant".to_owned(),
            result: None,
        })
    } else {
        let db_client = database::get_instance().await;

        match db_client {
            Ok(client) => {
                let config = database::get_selected_config(&client, database::filter_tenant_for_project(&client, key.tenants.clone(), projectId.clone()).await.unwrap_or(database::Tenant {name: key.email.clone( )}), &projectId).await;
    
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
    if key.email == "" {
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
                let res = database::update_project_selected_config( &client, database::filter_tenant_for_project(&client, key.tenants.clone(), projectId.clone()).await.unwrap_or(database::Tenant {name: key.email.clone( )}), &projectId, &thing).await;
    
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
    if key.email == "" {
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
                let res = database::create_org( &client, crate::database::Tenant { name: key.email }, &thing).await;

                match res {
                    Ok(res) => {
                        Json(models::ApiOrgResponse {
                            ok: true,
                            message: "Created org successfully".to_owned(),
                            result: Some(models::ApiOrgMetadata {
                                name: thing.name,
                                id: res,
                                plan: thing.plan.unwrap_or("standard".to_owned())
                            }),
                        })
                    },
                    Err(err) => {
                        eprintln!("{}", err);
                        Json(models::ApiOrgResponse {
                            ok: false,
                            message: "Error creating org".to_owned(),
                            result: None,
                        })
                    }
                }
            },
            Err(err) => Json(models::ApiOrgResponse {
                ok: false,
                message: "Could not connect to DB".to_owned(),
                result: None,
            })
        }    
    }
}

#[put("/orgs/<org_id>", data = "<body>")]
async fn orgs_put(org_id: String, body: Json<models::ApiOrgMetadataBase>, key: auth::ApiKey) -> Json<models::ApiOrgResponse> {
    if key.email == "" {
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
                let res = database::update_org( &client, key.tenants, org_id, &thing).await;

                match res {
                    Ok(res) => {
                        Json(models::ApiOrgResponse {
                            ok: true,
                            message: "Updated org successfully".to_owned(),
                            result: Some(models::ApiOrgMetadata {
                                name: thing.name,
                                id: res,
                                plan: thing.plan.unwrap_or("standard".to_owned())
                            }),
                        })
                    },
                    Err(err) => {
                        eprintln!("{}", err);
                        Json(models::ApiOrgResponse {
                            ok: false,
                            message: "Error creating org".to_owned(),
                            result: None,
                        })
                    }
                }
            },
            Err(err) => Json(models::ApiOrgResponse {
                ok: false,
                message: "Could not connect to DB".to_owned(),
                result: None,
            })
        }    
    }
}



#[get("/orgs")]
async fn orgs_get(key: auth::ApiKey) -> Json<models::ApiGetOrgsResponse> {
    if key.email == "" {
        Json(models::ApiGetOrgsResponse {
            ok: false,
            message: "Could not find a tenant".to_owned(),
            result: None,
        })
    } else {
        let db_client = database::get_instance().await;
        match db_client {
            Ok(client) => {
                let res = database::get_orgs( &client, key.tenants).await;
                match res {
                    Ok(res) => {
                        Json(models::ApiGetOrgsResponse {
                            ok: true,
                            message: "Created org successfully".to_owned(),
                            result: Some(models::OrgMetadataList {
                                orgs: res
                            })
                        })
                    },
                    Err(err) => {
                        eprintln!("{}", err);
                        Json(models::ApiGetOrgsResponse {
                            ok: false,
                            message: "Getting orgs failed".to_owned(),
                            result: None,
                        })
                    }
                }
            }
            Err(e) => Json(models::ApiGetOrgsResponse {
                ok: false,
                message: "Could not connect to DB".to_owned(),
                result: None,
            }),
        }
    
    }
    
}

#[post("/orgs/<org_id>/members", data = "<body>")]
async fn orgs_members_post(org_id: String, body: Json<models::ApiAddMemberPayload>, key: auth::ApiKey) -> Json<models::ApiAddMemberResponse> {
    if key.email == "" {
        Json(models::ApiAddMemberResponse {
            ok: false,
            message: "Could not find a tenant".to_owned(),
            result: None,
        })
    } else {
        let db_client = database::get_instance().await;

        match db_client {
            Ok(client) => {
                let thing = body.into_inner();

                let existing_user_count = database::get_user_count_in_org(&client, key.tenants.clone(), org_id.clone()).await;
                let org_max_count = database::get_max_user_count_in_org(&client, key.tenants.clone(), org_id.clone()).await;

                if (org_max_count.is_err()) {
                    return Json(models::ApiAddMemberResponse {
                        ok: false,
                        message: "Error getting max user org count".to_owned(),
                        result: None,
                    })
                }

                // Only allow 5 users per org.
                match existing_user_count {
                    Ok(user_count) => {
                        if user_count >= org_max_count.expect("Checked") {
                            Json(models::ApiAddMemberResponse {
                                ok: false,
                                message: "Too many users. Please upgrade.".to_owned(),
                                result: None,
                            })
                        } else {
                            let res = database::add_user_to_org(&client, key.tenants, org_id, thing.email.clone()).await;
        
                            match res {
                                Ok(res) => Json(models::ApiAddMemberResponse {
                                    ok: true,
                                    message: "Created org successfully".to_owned(),
                                    result: Some(models::ApiAddMemberPayload {
                                        email: thing.email
                                    })
                                }),
                                Err(err) => Json(models::ApiAddMemberResponse {
                                    ok: false,
                                    message: "Adding user to org failed".to_owned(),
                                    result: None,
                                })
                            }
                        }
                    },
                    Err(err) => {
                        Json(models::ApiAddMemberResponse {
                            ok: false,
                            message: "Failed to count users".to_owned(),
                            result: None,
                        })
                    }
                }
            }
            Err(err) => Json(models::ApiAddMemberResponse {
                ok: false,
                message: "Could not connect to DB".to_owned(),
                result: None,
            })
        }    
    }
}

#[get("/orgs/<org_id>/members")]
async fn org_members_get(org_id: String, key: auth::ApiKey) -> Json<models::ApiGetMembersResponse> {
    if key.email == "" {
        Json(models::ApiGetMembersResponse {
            ok: false,
            message: "Could not find a tenant".to_owned(),
            result: None,
        })
    } else {
        let db_client = database::get_instance().await;
        match db_client {
            Ok(client) => {
                let res = database::get_members_for_org( &client, org_id, key.tenants).await;
                
                match res {
                    Ok(res) => {
                        Json(models::ApiGetMembersResponse {
                            ok: true,
                            message: "Got members".to_owned(),
                            result: Some(models::ApiGetMembersResult {
                                members: res
                            })
                        })
                    },
                    Err(err) => Json(models::ApiGetMembersResponse {
                        ok: false,
                        message: "Getting members failed".to_owned(),
                        result: None,
                    })
                }
            }
            Err(e) => Json(models::ApiGetMembersResponse {
                ok: false,
                message: "Could not connect to DB".to_owned(),
                result: None,
            }),
        }
    
    }
    
}

#[delete("/orgs/<org_id>/members", data = "<body>")]
async fn orgs_members_delete(org_id: String, body: Json<models::ApiAddMemberPayload>, key: auth::ApiKey) -> Json<models::ApiGetMembersResponse> {
    if key.email == "" {
        Json(models::ApiGetMembersResponse {
            ok: false,
            message: "Could not find a tenant".to_owned(),
            result: None,
        })
    } else {
        let db_client = database::get_instance().await;

        match db_client {
            Ok(client) => {
                let thing = body.into_inner();

                if thing.email == key.email {
                    Json(models::ApiGetMembersResponse {
                        ok: false,
                        message: "You can not remove yourself".to_owned(),
                        result: None,
                    })
                } else {
                    let existing_user_count = database::get_user_count_in_org(&client, key.tenants.clone(), org_id.clone()).await;

                    match existing_user_count {
                        Ok(user_count) => {

                            if user_count > 1 {
                                let res = database::remove_user_from_org(&client, key.tenants.clone(), org_id.clone(), thing.email.clone()).await;

                                match res {
                                    Ok(res) => {
                                        let res = database::get_members_for_org( &client, org_id, key.tenants).await;
                                
                                        match res {
                                            Ok(res) => {
                                                Json(models::ApiGetMembersResponse {
                                                    ok: true,
                                                    message: "Got members".to_owned(),
                                                    result: Some(models::ApiGetMembersResult {
                                                        members: res
                                                    })
                                                })
                                            },
                                            Err(err) => Json(models::ApiGetMembersResponse {
                                                ok: false,
                                                message: "Getting members failed".to_owned(),
                                                result: None,
                                            })
                                        }
                        
                                    },
                                    Err(err) => Json(models::ApiGetMembersResponse {
                                        ok: false,
                                        message: "Removal of user failed".to_owned(),
                                        result: None,
                                    })
                                }
                            } else {
                                Json(models::ApiGetMembersResponse {
                                    ok: false,
                                    message: "Can not delete last member.".to_owned(),
                                    result: None,
                                })
                            }

                        },
                        Err(err) => {
                            Json(models::ApiGetMembersResponse {
                                ok: false,
                                message: "Failed to get user count".to_owned(),
                                result: None,
                            })
                        }
                    }


                }
            }
            Err(err) => Json(models::ApiGetMembersResponse {
                ok: false,
                message: "Could not connect to DB".to_owned(),
                result: None,
            })
        }    
    }
}


#[delete("/orgs/<org_id>")]
async fn org_delete(org_id: String, key: auth::ApiKey) -> Json<models::ApiDeleteOrgResponse> {
    if key.email == "" {
        Json(models::ApiDeleteOrgResponse {
            ok: false,
            message: "Could not find a tenant".to_owned(),
            result: None,
        })
    } else {
        let db_client = database::get_instance().await;

        match db_client {
            Ok(client) => {
                let res = database::delete_org(&client, key.tenants.clone(), org_id.clone()).await;

                match res {
                    Ok(_) => {
                        Json(models::ApiDeleteOrgResponse {
                            ok: true,
                            message: "Got members".to_owned(),
                            result: Some(models::DeleteOrgResponseResult {
                                
                            })
                        })
                    },
                    Err(err) => {
                        Json(models::ApiDeleteOrgResponse {
                            ok: false,
                            message: "Error trying to delete org".to_owned(),
                            result: None,
                        })
                    }
                }
            }
            Err(err) => Json(models::ApiDeleteOrgResponse {
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
                auth_personal_tokens_post,
                project_get,
                projects_get,
                projects_post,
                projects_put,
                projects_delete,
                projects_trees_post,
                projects_trees_get,
                projects_trees_tree_get,
                projects_trees_tree_put,
                projects_trees_tree_delete,
                projects_trees_tree_undo_put,
                projects_trees_tree_public_get,
                projects_trees_tree_public_put,
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
                node_get,
                orgs_post,
                orgs_get,
                org_delete,
                orgs_put,
                org_members_get,
                orgs_members_post,
                orgs_members_delete
            ],
        )
        .attach(CORS)
}
