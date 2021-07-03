use mongodb::{
    bson::{doc, Document},
    sync::Client,
};

use crate::constants;
use crate::models;
use crate::helpers;
use crate::errors;

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
                                id: doc.get_str("_id").ok()?.to_string(),
                                related_tree_ids: helpers::convert_bson_objectid_array_to_str_array(doc.get_array("related_tree_ids").ok()?.clone())
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

pub fn new_project(client: mongodb::sync::Client, title: String) -> Result<String, errors::DatabaseError> {
    let database = client.database(constants::DATABASE_NAME);
    let collection = database.collection::<Document>("projects");

    let insert_result = collection.insert_one(doc! { "title": title }, None)?;
    let inserted_id = insert_result.inserted_id;

    match inserted_id.as_object_id().clone() {
        Some(oid) => Ok(oid.to_string()),
        None => Err(errors::DatabaseError { message: "No object ID found.".to_string() })
    }
}


pub fn create_project_tree(client: mongodb::sync::Client, title: String, project_id: String) -> Result<String, errors::DatabaseError> {
    let database = client.database(constants::DATABASE_NAME);
    let trees_collection = database.collection::<Document>("trees");
    let project_collection = database.collection::<Document>("projects");

    let insert_result = trees_collection.insert_one(doc! { "title": title }, None)?;
    let inserted_id = insert_result.inserted_id;

    project_collection.find_one_and_update(doc! {
        "_id": project_id.to_owned()
    }, doc! {
        "$push": {
            "related_tree_ids": inserted_id.clone()
        }
    }, None)?;

    match inserted_id.as_object_id().clone() {
        Some(oid) => Ok(oid.to_string()),
        None => Err(errors::DatabaseError { message: "No object ID found.".to_string() })
    }
}
