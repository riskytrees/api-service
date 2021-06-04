#![feature(proc_macro_hygiene, decl_macro)]

#[macro_use] extern crate rocket;

use mongodb::{
    bson::{doc, Bson, Document},
    sync::Client,
};

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

#[post("/auth/login")]
fn auth_login() -> &'static str {
    "todo"
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
