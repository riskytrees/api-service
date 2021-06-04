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

// Checks if user already exists in the databse
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
