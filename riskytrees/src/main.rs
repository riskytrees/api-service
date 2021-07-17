#![feature(proc_macro_hygiene, decl_macro)]

#[macro_use]
extern crate rocket;

use mongodb::bson::doc;
use rocket_contrib::json::Json;

mod constants;
mod database;
mod errors;
mod helpers;
mod models;

#[cfg(test)]
mod tests;

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
            let project = database::get_project_by_id(client.to_owned(), id.to_owned());
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
                database::get_project_by_id(client.to_owned(), id.to_owned());
            match project {
                Some(project) => {
                    // Get trees
                    let trees = database::get_trees_by_project_id(client.to_owned(), project.id);

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
                    ok: true,
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

fn main() {
    rocket::ignite()
        .mount(
            "/",
            routes![
                index,
                auth_login,
                auth_logout,
                projects_post,
                projects_trees_post,
                projects_trees_get
            ],
        )
        .launch();
}
