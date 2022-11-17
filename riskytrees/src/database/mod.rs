use mongodb::{
    bson::{doc, Document},
    sync::Client,
};

use std::{collections::HashMap, hash::Hash};

use bson::bson;
use crate::{constants, errors::DatabaseError, models::ApiTreeDagItem};
use crate::errors;
use crate::helpers;
use crate::models;

pub fn get_instance() -> Result<mongodb::sync::Client, mongodb::error::Error> {
    let client = Client::with_uri_str(constants::get_database_host().as_str())?;

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
                            selected_model: match doc.get_str("selectedModel").ok() {
                                Some(val) => Some(val.to_string()),
                                None => {
                                    None
                                }
                            }
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

pub fn get_project_by_id(client: &mongodb::sync::Client, id: String) -> Option<models::Project> {
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

                let selected_model = match doc.get_str("selectedModel").ok() {
                    Some(val) => Some(val.to_string()),
                    None => {
                        None
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
                    selected_model: selected_model
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

// Gets a list of project ids that the current user can access
pub fn get_available_project_ids(client: &mongodb::sync::Client
) -> Result<Vec<String>, errors::DatabaseError> {
    let database = client.database(constants::DATABASE_NAME);
    let collection = database.collection::<Document>("projects");

    let matched_records = collection.find(doc!{}, None);

    let mut resulting_ids = Vec::new();

    match matched_records {
        Ok(mut records) => {
            while let Some(record) = records.next() {

                let _ = match record {
                    Ok(record) => {
                        let id = record.get_object_id("_id").expect("_id is always an oid");
                        resulting_ids.push(id.to_hex());
                    },
                    Err(err) => eprintln!("MongoDB returned an error: {}", err),
                };
            }
            Ok(resulting_ids)
        },
        Err(err) => Err(errors::DatabaseError {
            message: "Database failed to lookup projects!".to_string(),
        })
    }
}

pub fn new_project(
    client: mongodb::sync::Client,
    title: String,
) -> Result<String, errors::DatabaseError> {
    let database = client.database(constants::DATABASE_NAME);
    let collection = database.collection::<Document>("projects");

    let insert_result =
        collection.insert_one(doc! { "title": title, "related_tree_ids": [], "selectedModel": bson!(null) }, None)?;
    let inserted_id = insert_result.inserted_id;

    match inserted_id.as_object_id().clone() {
        Some(oid) => Ok(oid.to_string()),
        None => Err(errors::DatabaseError {
            message: "No object ID found.".to_string(),
        }),
    }
}

pub fn update_project_model(client: mongodb::sync::Client, project_id: String, modelId: String) -> Result<bool, errors::DatabaseError> {
    let database = client.database(constants::DATABASE_NAME);
    let project_collection = database.collection::<Document>("projects");

    
    let matched_record = project_collection.find_one(
        doc! {
            "_id": bson::oid::ObjectId::with_string(&project_id.to_owned()).expect("infallible")
        },
        None,
    )?;

    match matched_record {
        Some(record) => {
            let title = record.get_str("title").expect("infalliable");
            let tree_ids = record.get_array("related_tree_ids").expect("infalliable");
            let new_doc = doc! {
                "title": title, "related_tree_ids": tree_ids, "selectedModel": modelId
            };

            project_collection.find_one_and_replace(doc! {
                "_id": bson::oid::ObjectId::with_string(&project_id.to_owned()).expect("infallible")
            }, new_doc, None);

            Ok(true)
        },
        None => {
            return Err(errors::DatabaseError {
                message: "Could not find project with _id = {}".to_string(),
            });
        }
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

    let insert_result = trees_collection.insert_one(doc! {
        "title": title,
        "rootNodeId": "" // Start with no root
    }, None)?;
    let inserted_id = insert_result.inserted_id.as_object_id();

    match inserted_id.clone() {
        Some(oid) => {
            project_collection.find_one_and_update(
                doc! {
                    "_id": bson::oid::ObjectId::with_string(&project_id.to_owned()).expect("infallible")
                },
                doc! {
                    "$push": {
                        "related_tree_ids": oid.clone()
                    }
                },
                None,
            )?;

            Ok(oid.to_string().clone())
        },
        None => Err(errors::DatabaseError {
            message: "No object ID found.".to_string(),
        }),
    }
}

fn get_tree_items_from_tree_ids(client: &mongodb::sync::Client, tree_ids: Vec<String>) -> Vec<models::ListTreeResponseItem> {
    let database = client.database(constants::DATABASE_NAME);
    let trees_collection = database.collection::<Document>("trees");
    let mut result = Vec::new();

    for tree_id in tree_ids {
        let matched_records = trees_collection.find(
            doc! {
                "_id": bson::oid::ObjectId::with_string(&tree_id.to_owned()).expect("infallible")
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
    }

    result
}

fn convert_bson_document_to_ModelAttribute_map(bson_doc: &Document) -> HashMap<String, models::ModelAttribute> {
    let mut new_map: HashMap<String, models::ModelAttribute> = HashMap::new();

    for (key, val) in bson_doc.into_iter() {
        match val.as_document() {
            Some(val) => {
                if !val.is_null("value_string") {
                    new_map.insert(key.clone(), models::ModelAttribute {
                        value_string: Some(val.get_str("value_string").expect("Should match type field").to_owned()),
                        value_int: None,
                        value_float: None
                    });
                } else if !val.is_null("value_int") {
                    new_map.insert(key.clone(), models::ModelAttribute {
                        value_string: None,
                        value_int: Some(val.get_i32("value_int").expect("Should match type field")),
                        value_float: None
                    });
                } else if !val.is_null("value_float") {
                    new_map.insert(key.clone(), models::ModelAttribute {
                        value_string: None,
                        value_int: None,
                        value_float: Some(val.get_f64("value_float").expect("Should match type field"))
                    });
                } else {

                }
            },
            None => {
                eprintln!("Stored model attribute not a document!");
            }
        };
    }

    new_map
}

// Returns all the data contained in a single tree
fn get_full_tree_data(client: &mongodb::sync::Client, tree_id: String) -> Result<models::ApiFullTreeData, errors::DatabaseError> {
    let database = client.database(constants::DATABASE_NAME);
    let trees_collection = database.collection::<Document>("trees");

    let matched_record = trees_collection.find_one(
        doc! {
            "_id": bson::oid::ObjectId::with_string(&tree_id.to_owned()).expect("infallible")
        },
        None,
    )?;

    match matched_record {
        Some(tree_record) => {
            let empty_bson_array = bson::Array::new();
            let title = tree_record.get_str("title").expect("title should always exist");

            let root_node_id = tree_record.get_str("rootNodeId").expect("rootNodeId should always exist");
            let nodes = tree_record.get_array("nodes").unwrap_or(&empty_bson_array);
            let mut nodes_vec = Vec::new();

            for node in nodes.into_iter() {
                match node.as_document() {
                    Some(node) => {
                        let title = node.get_str("title").expect("title should always exist");
                        let description = node.get_str("description").expect("description should always exist");

                        let id = node.get_str("id").expect("id should always exist");

                        let condition_attribute = node.get_str("conditionAttribute").ok();
                        let children: Option<Vec<String>> = match node.get_array("children") {
                            Ok(val) => Some(helpers::convert_bson_str_array_to_str_array(val.clone())),
                            Err(err) => None
                        };

                        let model_attributes = match node.get_document("modelAttributes") {
                            Ok(val) => Some(convert_bson_document_to_ModelAttribute_map(val)),
                            Err(err) => None
                        };

                        println!("{:?}", model_attributes);

                        nodes_vec.push(models::ApiFullNodeData {
                            id: id.to_owned(),
                            title: title.to_owned(),
                            description: description.to_owned(),
                            conditionAttribute: condition_attribute.unwrap_or("").to_owned(),
                            children: children.unwrap_or(Vec::new()),
                            modelAttributes: model_attributes.unwrap_or(HashMap::new())
                        })
                    },
                    None => {
                        eprint!("nodes should be an array of objects, but isn't!")
                    }
                }

            }
            Ok(models::ApiFullTreeData {
                title: title.to_owned(),
                rootNodeId: root_node_id.to_owned(),
                nodes: nodes_vec
            })
        },
        None => {
            Err(errors::DatabaseError {
                message: "Couldn't find tree".to_owned()
            })
        }
    }
}

pub fn get_trees_by_project_id(
    client: &mongodb::sync::Client,
    project_id: String,
) -> Result<Vec<models::ListTreeResponseItem>, errors::DatabaseError> {

    let matched_project = get_project_by_id(client, project_id.to_owned());

    match matched_project {
        Some(project) => {
            let tree_ids = project.related_tree_ids;
            let trees = get_tree_items_from_tree_ids(client, tree_ids);

            Ok(trees)
        }
        None => Err(errors::DatabaseError {
            message: "Failed to search for matching trees".to_string(),
        }),
    }
}

pub fn get_tree_by_id(
    client: &mongodb::sync::Client,
    tree_id: String,
) -> Result<models::ApiFullTreeData, errors::DatabaseError> {

    get_full_tree_data(client, tree_id)

}

pub fn update_tree_by_id(
    client: &mongodb::sync::Client,
    tree_id: String,
    tree_data: models::ApiFullTreeData
) -> Result<models::ApiFullTreeData, errors::DatabaseError> {
    let database = client.database(constants::DATABASE_NAME);
    let trees_collection = database.collection::<Document>("trees");

    let doc = tree_data.to_bson_doc();

    trees_collection.find_one_and_replace(doc! {
        "_id": bson::oid::ObjectId::with_string(&tree_id.to_owned()).expect("infallible")
    }, doc, None);


    get_full_tree_data(client, tree_id)
}

pub fn get_projects_from_ids(ids: Vec<String>, client: &mongodb::sync::Client) -> Vec<models::ApiProjectsListProjectItem> {
    let mut result = Vec::new();

    for id in ids {
        let project_data = get_project_by_id(client, id);
        match project_data {
            Some(project_data) => {
                result.push(models::ApiProjectsListProjectItem {
                    projectId: project_data.id,
                    name: project_data.title
                })
            },
            None => { /* Skip */ }
        }
    }

    result
}

pub fn get_tree_from_node_id(node_id: String, client: &mongodb::sync::Client) -> Result<models::ApiGetNodeResponse, errors::DatabaseError> {
    let database = client.database(constants::DATABASE_NAME);
    let trees_collection = database.collection::<Document>("trees");

    match trees_collection.find_one(doc! {
        "nodes": {
            "$elemMatch": {
                "id": node_id.to_string()
            }
        }
    }, None) {
        Ok(res) => {
            match res {
                Some(found_doc) => {
                    Ok(models::ApiGetNodeResponse {
                        ok: true,
                        message: "Found node".to_string(),
                        result: Some(models::ApiGetNodeResponseResult {
                            treeId: found_doc.get_object_id("_id").expect("Should always exist").to_string()
                        })
                    })
                },
                None => {
                    Err(DatabaseError { message: "No matching node".to_string() })
                }
            }
        },
        Err(err) => {
            Err(DatabaseError {
                message: err.to_string()
            })
        }
    }
}

pub fn get_tree_relationships_down(startTreeId: &String, client: &mongodb::sync::Client) -> Vec<ApiTreeDagItem> {
    let mut result = vec![];

    let childrenNodes = get_nodes_from_tree(startTreeId, client);

    // Figure out which nodes have children that aren't included in this list of nodes
    let mut childrenOfConcern = std::collections::HashSet::new();
    let mut nodeInclusionMap = HashMap::new();

    for node in &childrenNodes {
        nodeInclusionMap.insert(node.id.clone(), true);
    }

    for node in &childrenNodes {
        for child in &(node.children) {
            if !nodeInclusionMap.contains_key(child) {
                childrenOfConcern.insert(child);
            }
        }
    }

    // Resolve children
    let mut childTrees = vec![];

    for node in childrenOfConcern {
        let lookup = get_tree_from_node_id(node.to_string(), client);
        match lookup {
            Ok(res) => {
                match res.result {
                    Some(res) => {
                        childTrees.push(res.treeId);
                    },
                    None => {}
                }
            }, Err(err) => {
                eprintln!("{}", err);
            }
        }
    }

    for childTree in &childTrees {
        result.push(ApiTreeDagItem { id: childTree.to_string(), children: get_tree_relationships_down(childTree, client) });
    }

    result
}

pub fn get_nodes_from_tree(treeId: &String, client: &mongodb::sync::Client) -> Vec<models::ApiFullNodeData> {
    let data = get_full_tree_data(client, treeId.to_string());

    match data {
        Ok(res) => {
            res.nodes
        },
        Err(err) => {
            eprintln!("{}", err);
            vec![]
        }
    }
}