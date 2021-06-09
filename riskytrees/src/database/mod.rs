use mongodb::{
    bson::{doc, Document},
    sync::Client,
};

use crate::constants;
use crate::models;

pub fn get_instance() -> Result<mongodb::sync::Client, mongodb::error::Error> {
    let client = Client::with_uri_str(constants::DATABASE_HOST)?;

    Ok(client)
}

// Checks if user already exists in the databse. If it does, it is returned.
pub fn get_user(client: mongodb::sync::Client, email: String) -> Option<models::User> {
    let database = client.database(constants::DATABASE_NAME);
    let collection = database.collection::<Document>("users");

    match collection.count_documents(doc! {"email": email.to_owned()}, None) {
        Ok(count) => {
            if count > 0 {
                return match collection.find_one(doc! {"email": email.to_owned()}, None) {
                    Ok(res) => {
                        match res {
                            Some(doc) => Some(models::User {
                                email: doc.get_str("email").ok()?.to_string()
                            }),
                            None => None
                        }
                    },
                    Err(err) => None
                }
            }

            None
        },
        Err(err) => {
            None
        }
    }
}

pub fn new_user(client: mongodb::sync::Client, email: String) -> bool {
    let database = client.database(constants::DATABASE_NAME);
    let collection = database.collection::<Document>("users");

    collection.insert_one(doc! { "email": email }, None);

    true
}


// Checks if a project already exists in the databse. If it does, it is returned.
pub fn get_project(client: mongodb::sync::Client, title: String) -> Option<models::Project> {
    let database = client.database(constants::DATABASE_NAME);
    let collection = database.collection::<Document>("projects");

    match collection.count_documents(doc! {"title": title.to_owned()}, None) {
        Ok(count) => {
            if count > 0 {
                return match collection.find_one(doc! {"title": title.to_owned()}, None) {
                    Ok(res) => {
                        match res {
                            Some(doc) => Some(models::Project {
                                title: doc.get_str("title").ok()?.to_string(),
                                id: doc.get_i32("id").ok()?
                            }),
                            None => None
                        }
                    },
                    Err(err) => None
                }
            }

            None
        },
        Err(err) => {
            None
        }
    }
}
