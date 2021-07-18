use mongodb::{
    bson::{doc, Document},
    sync::Client,
};

use crate::constants;
use crate::errors;
use crate::helpers;
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
                    Ok(res) => match res {
                        Some(doc) => Some(models::User {
                            email: doc.get_str("email").ok()?.to_string(),
                        }),
                        None => None,
                    },
                    Err(err) => None,
                };
            }

            None
        }
        Err(err) => None,
    }
}

pub fn new_user(client: mongodb::sync::Client, email: String) -> bool {
    let database = client.database(constants::DATABASE_NAME);
    let collection = database.collection::<Document>("users");

    collection.insert_one(doc! { "email": email }, None);

    true
}

// Checks if a project already exists in the databse. If it does, it is returned.
pub fn get_project_by_title(
    client: mongodb::sync::Client,
    title: String,
) -> Option<models::Project> {
    let database = client.database(constants::DATABASE_NAME);
    let collection = database.collection::<Document>("projects");

    match collection.count_documents(doc! {"title": title.to_owned()}, None) {
        Ok(count) => {
            if count > 0 {
                return match collection.find_one(doc! {"title": title.to_owned()}, None) {
                    Ok(res) => match res {
                        Some(doc) => Some(models::Project {
                            title: doc.get_str("title").ok()?.to_string(),
                            id: doc.get_str("_id").ok()?.to_string(),
                            related_tree_ids: helpers::convert_bson_objectid_array_to_str_array(
                                doc.get_array("related_tree_ids").ok()?.clone(),
                            ),
                        }),
                        None => None,
                    },
                    Err(err) => None,
                };
            }

            None
        }
        Err(err) => None,
    }
}

pub fn get_project_by_id(client: mongodb::sync::Client, id: String) -> Option<models::Project> {
    let database = client.database(constants::DATABASE_NAME);
    let collection = database.collection::<Document>("projects");

    match collection.find_one(
        doc! {"_id":  bson::oid::ObjectId::with_string(&id.to_owned()).expect("infallible")},
        None,
    ) {
        Ok(res) => match res {
            Some(doc) => {
                let title = match doc.get_str("title").ok() {
                    Some(val) => val,
                    None => {
                        println!("Found record does not have title field.");
                        ""
                    }
                };

                let id = match doc.get_object_id("_id").ok() {
                    Some(val) => val.to_hex(),
                    None => {
                        println!("Found record does not have id field.");
                        "".to_string()
                    }
                };

                let tree_ids = match doc.get_array("related_tree_ids").ok() {
                    Some(val) => val.clone(),
                    None => {
                        println!("Found record does not have related_tree_ids");
                        Vec::new()
                    }
                };

                let returnres = Some(models::Project {
                    title: title.to_string(),
                    id: id.to_string(),
                    related_tree_ids: helpers::convert_bson_objectid_array_to_str_array(tree_ids),
                });
                returnres
            }
            None => {
                println!("Could not find project with _id = {}", id);
                None
            }
        },
        Err(err) => {
            println!("find one failed with project with _id = {}", id);
            None
        }
    }
}

pub fn new_project(
    client: mongodb::sync::Client,
    title: String,
) -> Result<String, errors::DatabaseError> {
    let database = client.database(constants::DATABASE_NAME);
    let collection = database.collection::<Document>("projects");

    let insert_result =
        collection.insert_one(doc! { "title": title, "related_tree_ids": [] }, None)?;
    let inserted_id = insert_result.inserted_id;

    match inserted_id.as_object_id().clone() {
        Some(oid) => Ok(oid.to_string()),
        None => Err(errors::DatabaseError {
            message: "No object ID found.".to_string(),
        }),
    }
}

pub fn create_project_tree(
    client: mongodb::sync::Client,
    title: String,
    project_id: String,
) -> Result<String, errors::DatabaseError> {
    let database = client.database(constants::DATABASE_NAME);
    let trees_collection = database.collection::<Document>("trees");
    let project_collection = database.collection::<Document>("projects");

    let insert_result = trees_collection.insert_one(doc! { "title": title }, None)?;
    let inserted_id = insert_result.inserted_id;

    project_collection.find_one_and_update(
        doc! {
            "_id": project_id.to_owned()
        },
        doc! {
            "$push": {
                "related_tree_ids": inserted_id.clone()
            }
        },
        None,
    )?;

    match inserted_id.as_object_id().clone() {
        Some(oid) => Ok(oid.to_string()),
        None => Err(errors::DatabaseError {
            message: "No object ID found.".to_string(),
        }),
    }
}

fn get_tree_items_from_tree_ids(client: mongodb::sync::Client, tree_ids: Vec<String>) {
    let database = client.database(constants::DATABASE_NAME);
    let trees_collection = database.collection::<Document>("trees");
    let mut result = Vec::new();

    for tree_id in tree_ids {
        let matched_records = trees_collection.find(
            doc! {
                "related_tree_ids": tree_id
            },
            None,
        );

        match matched_records {
            Ok(mut records) => {
                while let Some(record) = records.next() {
                    println!("Found a match");

                    let _ = match record {
                        Ok(record) => result.push(models::ListTreeResponseItem {
                            title: record
                                .get_str("title")
                                .expect("Title should always exist")
                                .to_string(),
                            id: record
                                .get_object_id("_id")
                                .expect("_id should always exist")
                                .to_string(),
                        }),
                        Err(err) => eprintln!("MongoDB returned an error: {}", err),
                    };
                }
            }
            Err(err) => eprintln!("Getting matched records failed: {}", err),
        }
    };

    result
}

pub fn get_trees_by_project_id(
    client: mongodb::sync::Client,
    project_id: String,
) -> Result<Vec<models::ListTreeResponseItem>, errors::DatabaseError> {
    let database = client.database(constants::DATABASE_NAME);
    let project_collection = database.collection::<Document>("projects");
    let mut result = Vec::new();

    let matched_project = get_project_by_id(client, project_id.to_owned());

    match matched_project {
        Some(project) => {
            let tree_ids = project.related_tree_ids;

            Ok(result)
        }
        None => Err(errors::DatabaseError {
            message: "Failed to search for matching trees".to_string(),
        }),
    }
}
