#![feature(proc_macro_hygiene, decl_macro)]

#[macro_use] extern crate rocket;

use mongodb::{
    bson::{doc},
};
use rocket_contrib::json::Json;
mod database;
mod constants;
mod models;

#[get("/")]
fn index() -> &'static str {
    let db_client = database::get_instance();
    match db_client {
        Ok(client) => {
            database::new_user(client, "Test".to_string());
            "Hello, world!"
        },
        Err(e) => "Failed to create user"
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
                        sessionToken: "testtoken".to_owned()
                    })
                })
            } else {
                Json(models::ApiAuthLoginResponse {
                    ok: true,
                    message: "User logged in succesfully".to_owned(),
                    result: None
                })

            }
        },
        Err(e) =>  Json(models::ApiAuthLoginResponse {
            ok: false,
            message: "Could not connect to DB".to_owned(),
            result: None
        })
    }
}

#[post("/auth/logout")]
fn auth_logout() -> &'static str {
    "todo"
}

fn main() {
    rocket::ignite().mount("/", routes![
        index,
        auth_login,
        auth_logout]).launch();

}
