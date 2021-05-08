#![feature(proc_macro_hygiene, decl_macro)]

#[macro_use] extern crate rocket;

mod database;

#[get("/")]
fn index() -> &'static str {
    database::new_user();
    "Hello, world!"
}

fn main() {
    rocket::ignite().mount("/", routes![index]).launch();
}
