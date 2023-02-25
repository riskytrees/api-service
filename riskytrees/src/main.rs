#![feature(proc_macro_hygiene, decl_macro)]

#[macro_use]
extern crate rocket;
extern crate rocket_cors;

use std::collections::HashSet;
use database::get_tree_from_node_id;
use models::ApiProjectConfigListResponseResult;
use mongodb::bson::doc;
use rocket_contrib::json::{Json, self};
use rocket::{http::Method, Config, config::Environment};

use rocket_cors::{
    AllowedHeaders, AllowedOrigins, Cors, CorsOptions
};

mod constants;
mod database;
mod errors;
mod helpers;
mod models;
mod auth;

#[cfg(test)]
mod tests;

fn make_cors() -> Cors {
    let mut origins = HashSet::new();

    origins.insert("*".to_string());

    /*let allowed_origins = AllowedOrigins::Some(rocket_cors::Origins{
        allow_null: true,
        exact: Some(origins),
        regex: None
    });*/

    let allowed_origins = AllowedOrigins::All;

    CorsOptions {
        allowed_origins,
        allowed_methods: vec![Method::Get, Method::Post, Method::Delete, Method::Put, Method::Options].into_iter().map(From::from).collect(),
        allowed_headers: AllowedHeaders::some(&[
            "Authorization",
            "Accept",
            "Access-Control-Allow-Origin",
        ]),
        allow_credentials: true,
        ..Default::default()
    }
    .to_cors()
    .expect("error while building CORS")
}


#[get("/")]
fn index() -> &'static str {
    let db_client = database::get_instance();
    match db_client {
        Ok(client) => {
            "Hello, world!"
        }
        Err(e) => "Failed to create user",
    }
}

#[get("/auth/login?<code>&<state>&<scope>")]
fn auth_login_get(code: Option<String>, state: Option<String>, scope: Option<String>) -> Json<models::ApiAuthLoginResponse> {
    let db_client = database::get_instance();
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
                match database::validate_csrf_token(&state.expect("Asserted"), &client) {
                    Ok(nonce) => {
                        let email = auth::trade_token(&code.as_ref().expect("Asserted"), nonce);
                        match email {
                            Ok(email) => {
                                // Create user if user does not exist
                                let user_exists = database::get_user(&client, email.clone());
                                match user_exists {
                                    Some(user) => {},
                                    None => {
                                        database::new_user(&client, email.clone());
                                        ()
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
fn auth_login_post(provider: Option<String>) -> Json<models::ApiAuthLoginResponse> {
    let db_client = database::get_instance();
    match db_client {
        Ok(client) => {
            // Start flow
            let start_data = auth::start_flow();
            match start_data {
                Ok(start_data) => {
                    // Store csrf_token for lookup later
                    match database::store_csrf_token(&start_data.csrf_token, &start_data.nonce, &client) {
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
fn projects_get(key: auth::ApiKey) -> Json<models::ApiProjectsListResponse> {
    let db_client = database::get_instance();
    match db_client {
        Ok(client) => {
            let ids = database::get_available_project_ids(&client);
            match ids {
                Ok(ids) => Json(models::ApiProjectsListResponse {
                    ok: true,
                    message: "Got projects".to_owned(),
                    result: Some(models::ApiProjectsListResponseResult {
                        projects: database::get_projects_from_ids(ids, &client)
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

#[post("/projects", data = "<body>")]
fn projects_post(body: Json<models::ApiCreateProject>, key: auth::ApiKey) -> Json<models::ApiCreateProjectResponse> {
    let db_client = database::get_instance();
    match db_client {
        Ok(client) => {
            let project = database::get_project_by_title(client.to_owned(), body.title.to_owned());
            match project {
                Some(project) => Json(models::ApiCreateProjectResponse {
                    ok: true,
                    message: "Tree already exists".to_owned(),
                    result: None,
                }),
                None => match database::new_project(client, body.title.to_owned()) {
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

#[post("/projects/<id>/trees", data = "<body>")]
fn projects_trees_post(
    id: String,
    body: Json<models::ApiCreateTree>,
    key: auth::ApiKey,
) -> Json<models::ApiCreateTreeResponse> {
    let db_client = database::get_instance();
    match db_client {
        Ok(client) => {
            let project = database::get_project_by_id(&client, id.to_owned());
            match project {
                Some(project) => {
                    // Create tree
                    match database::create_project_tree(
                        client.to_owned(),
                        body.title.to_owned(),
                        id,
                    ) {
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

#[get("/projects/<id>/trees")]
fn projects_trees_get(id: String, key: auth::ApiKey) -> Json<models::ApiListTreeResponse> {
    let db_client = database::get_instance();
    match db_client {
        Ok(client) => {
            let project: Option<models::Project> =
                database::get_project_by_id(&client, id.to_owned());
            match project {
                Some(project) => {
                    // Get trees
                    let trees = database::get_trees_by_project_id(&client, project.id);

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

#[get("/projects/<id>/trees/<tree_id>")]
fn projects_trees_tree_get(id: String, tree_id: String, key: auth::ApiKey) -> Json<models::ApiTreeResponse> {
    let db_client = database::get_instance();
    match db_client {
        Ok(client) => {
            let project: Option<models::Project> =
                database::get_project_by_id(&client, id.to_owned());
            match project {
                Some(project) => {
                    let tree = database::get_tree_by_id(&client, tree_id.to_owned());
                    match tree {
                        Ok(tree) => {
                            Json(models::ApiTreeResponse {
                                ok: true,
                                message: "Found tree".to_owned(),
                                result: Some(tree),
                            })
                        },
                        Err(err) => {
                            Json(models::ApiTreeResponse {
                                ok: false,
                                message: "Could not find tree using id".to_owned(),
                                result: None,
                            })
                        }
                    }

                }
                None => Json(models::ApiTreeResponse {
                    ok: false,
                    message: "Could not find project".to_owned(),
                    result: None,
                }),
            }
        }
        Err(e) => Json(models::ApiTreeResponse {
            ok: false,
            message: "Could not connect to DB".to_owned(),
            result: None,
        }),
    }
}

#[put("/projects/<id>/trees/<tree_id>", data = "<body>")]
fn projects_trees_tree_put(id: String, tree_id: String, body: Json<models::ApiFullTreeData>, key: auth::ApiKey) -> Json<models::ApiTreeResponse> {
    let db_client = database::get_instance();
    match db_client {
        Ok(client) => {
            let project: Option<models::Project> =
                database::get_project_by_id(&client, id.to_owned());
            match project {
                Some(project) => {
                    // Update tree and return
                    let tree = database::update_tree_by_id(&client, tree_id.to_owned(), models::ApiFullTreeData {
                        title: body.title.to_owned(),
                        rootNodeId: body.rootNodeId.to_owned(),
                        nodes: body.nodes.clone()
                    });
                    match tree {
                        Ok(tree) => {
                            Json(models::ApiTreeResponse {
                                ok: true,
                                message: "Found tree".to_owned(),
                                result: Some(tree),
                            })
                        },
                        Err(err) => {
                            Json(models::ApiTreeResponse {
                                ok: false,
                                message: "Could not find tree using id".to_owned(),
                                result: None,
                            })
                        }
                    }

                }
                None => Json(models::ApiTreeResponse {
                    ok: false,
                    message: "Could not find project".to_owned(),
                    result: None,
                }),
            }
        }
        Err(e) => Json(models::ApiTreeResponse {
            ok: false,
            message: "Could not connect to DB".to_owned(),
            result: None,
        }),
    }
}


#[get("/projects/<id>/model")]
fn projects_model_get(id: String, key: auth::ApiKey) -> Json<models::ApiSelectedModelResponse> {
    let db_client = database::get_instance();
    match db_client {
        Ok(client) => {
            let project: Option<models::Project> =
                database::get_project_by_id(&client, id.to_owned());
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

#[put("/projects/<id>/model", data = "<body>")]
fn projects_model_put(id: String, body: Json<models::SelectedModelResult>, key: auth::ApiKey) -> Json<models::ApiSelectedModelResponse> {
    let db_client = database::get_instance();
    match db_client {
        Ok(client) => {
            let project: Option<models::Project> =
                database::get_project_by_id(&client, id.to_owned());
            match project {
                Some(project) => {
                    match database::update_project_model(client, id.to_owned(), body.modelId.to_owned()) {
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

#[get("/models")]
fn models_get(key: auth::ApiKey) -> Json<models::ApiListModelResponse> {
    let db_client = database::get_instance();

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
                ok: false,
                message: "Could not connect to DB".to_owned(),
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

#[get("/nodes/<id>")]
fn node_get(id: String, key: auth::ApiKey) -> Json<models::ApiGetNodeResponse> {
    let db_client = database::get_instance();

    match db_client {
        Ok(client) => {
            match get_tree_from_node_id(id, &client) {
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

#[get("/projects/<projectId>/trees/<treeId>/dag/down")]
fn projects_trees_tree_dag_down_get(projectId: String, treeId: String, key: auth::ApiKey) -> Json<models::ApiTreeDagResponse> {
    let db_client = database::get_instance();

    match db_client {
        Ok(client) => {
            let result = models::ApiTreeDagResponseResult {
                root: models::ApiTreeDagItem {
                    id: treeId.clone(),
                    children: database::get_tree_relationships_down(&treeId, &client)
                }
            };

            Json(models::ApiTreeDagResponse {
                ok: true,
                message: "Got relationship".to_string(),
                result: Some(result)
            })
        },
        Err(err) => Json(models::ApiTreeDagResponse {
            ok: false,
            message: "Could not connect to DB".to_owned(),
            result: None,
        })
    }
}

#[get("/projects/<projectId>/configs")]
fn projects_configs_get(projectId: String, key: auth::ApiKey) -> Json<models::ApiProjectConfigListResponse> {
    let db_client = database::get_instance();

    match db_client {
        Ok(client) => {
            let matching_configs = database::get_configs_for_project(&projectId, &client);

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

#[post("/projects/<projectId>/configs", data = "<body>")]
fn projects_configs_post(projectId: String, body: Json<models::ApiProjectConfigPayload>, key: auth::ApiKey) -> Json<models::ApiProjectConfigResponse> {
    let db_client = database::get_instance();

    match db_client {
        Ok(client) => {
            let thing = body.into_inner();

            let new_config_id: Result<String, errors::DatabaseError> = database::new_config(&projectId, &thing, &client);
            match new_config_id {
                Ok(new_config_id) => {
                    Json(models::ApiProjectConfigResponse {
                        ok: true,
                        message: "Created config".to_owned(),
                        result: Some(models::ApiProjectConfigResponseResult {
                            id: new_config_id,
                            attributes: rocket_contrib::json!(thing)
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

#[post("/projects/<projectId>/configs/<configId>", data = "<body>")]
fn projects_configs_put(projectId: String, configId: String, body: Json<models::ApiProjectConfigPayload>, key: auth::ApiKey) -> Json<models::ApiProjectConfigResponse> {
    let db_client = database::get_instance();

    match db_client {
        Ok(client) => {
            let thing = body.into_inner();
            let new_config = database::update_config(&projectId, &configId, &thing, &client);

            match new_config {
                Ok(updated_id) => {
                    Json(models::ApiProjectConfigResponse {
                        ok: true,
                        message: "Updated config".to_owned(),
                        result: Some(models::ApiProjectConfigResponseResult {
                            id: updated_id,
                            attributes: rocket_contrib::json!(thing)
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

#[get("/projects/<projectId>/config")]
fn projects_config_get(projectId: String, key: auth::ApiKey) -> Json<models::ApiProjectConfigResponse> {
    let db_client = database::get_instance();

    match db_client {
        Ok(client) => {
            let config = database::get_selected_config(&projectId, &client);

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

#[put("/projects/<projectId>/config", data = "<body>")]
fn projects_config_put(projectId: String, body: Json<models::ApiProjectConfigIdPayload>, key: auth::ApiKey) -> Json<models::ApiProjectConfigResponse> {
    let db_client = database::get_instance();

    match db_client {
        Ok(client) => {
            let thing: models::ApiProjectConfigIdPayload = body.into_inner();
            let res = database::update_project_selected_config(&projectId, &thing, &client);

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

fn main() {
    let config = Config::build(Environment::Staging)
    .address("0.0.0.0")
    .unwrap();

    rocket::custom(config)
        .mount(
            "/",
            routes![
                index,
                auth_login_get,
                auth_login_post,
                auth_logout,
                projects_get,
                projects_post,
                projects_trees_post,
                projects_trees_get,
                projects_trees_tree_get,
                projects_trees_tree_put,
                projects_trees_tree_dag_down_get,
                projects_model_get,
                projects_model_put,
                projects_configs_get,
                projects_configs_post,
                projects_config_get,
                projects_config_put,
                models_get,
                node_get
            ],
        )
        .attach(make_cors())

        .launch();
}
