#![feature(proc_macro_hygiene, decl_macro)]

#[macro_use]
extern crate rocket;
extern crate rocket_cors;

use std::collections::HashSet;
use mongodb::bson::doc;
use rocket_contrib::json::Json;

use rocket::http::Method;

use rocket_cors::{
    AllowedHeaders, AllowedOrigins, Error, Cors, CorsOptions
};

mod constants;
mod database;
mod errors;
mod helpers;
mod models;

#[cfg(test)]
mod tests;

fn make_cors() -> Cors {
    let mut origins = HashSet::new();

    origins.insert("http://localhost:8080".to_string());

    let allowed_origins = AllowedOrigins::Some(rocket_cors::Origins{
        allow_null: true,
        exact: Some(origins),
        regex: None
    });

    CorsOptions {
        allowed_origins,
        allowed_methods: vec![Method::Get].into_iter().map(From::from).collect(),
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
            database::new_user(client, "Test".to_string());
            "Hello, world!"
        }
        Err(e) => "Failed to create user",
    }
}

#[post("/auth/login", data = "<body>")]
fn auth_login(body: Json<models::ApiRegisterUser>) -> Json<models::ApiAuthLoginResponse> {
    let db_client = database::get_instance();
    match db_client {
        Ok(client) => {
            if database::get_user(client.to_owned(), body.email.to_owned()).is_none() {
                database::new_user(client, body.email.to_owned());

                Json(models::ApiAuthLoginResponse {
                    ok: true,
                    message: "User created and logged in succesfully".to_owned(),
                    result: Some(models::AuthLoginResponseResult {
                        sessionToken: "testtoken".to_owned(),
                    }),
                })
            } else {
                Json(models::ApiAuthLoginResponse {
                    ok: true,
                    message: "User logged in succesfully".to_owned(),
                    result: None,
                })
            }
        }
        Err(e) => Json(models::ApiAuthLoginResponse {
            ok: false,
            message: "Could not connect to DB".to_owned(),
            result: None,
        }),
    }
}

#[post("/auth/logout")]
fn auth_logout() -> &'static str {
    "todo"
}

#[get("/projects")]
fn projects_get() -> Json<models::ApiProjectsListResponse> {
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
fn projects_post(body: Json<models::ApiCreateProject>) -> Json<models::ApiCreateProjectResponse> {
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
fn projects_trees_get(id: String) -> Json<models::ApiListTreeResponse> {
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
fn projects_trees_tree_get(id: String, tree_id: String) -> Json<models::ApiTreeResponse> {
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
fn projects_trees_tree_put(id: String, tree_id: String, body: Json<models::ApiFullTreeData>) -> Json<models::ApiTreeResponse> {
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

fn main() {
    rocket::ignite()
        .mount(
            "/",
            routes![
                index,
                auth_login,
                auth_logout,
                projects_get,
                projects_post,
                projects_trees_post,
                projects_trees_get,
                projects_trees_tree_get,
                projects_trees_tree_put
            ],
        )
        .attach(make_cors())
        .launch();
}
