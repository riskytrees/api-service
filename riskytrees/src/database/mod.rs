use hmac::digest::core_api::UpdateCore;
use mongodb::{
    bson::{doc, Document}, options::UpdateOptions, Client
};
use openidconnect::Nonce;
use rand::{Rng, distributions::Alphanumeric};
use rocket::Data;

use std::{collections::{HashMap, HashSet}, hash::Hash, vec};

use bson::bson;
use crate::{constants, errors::DatabaseError, models::{ApiTreeDagItem, ApiProjectConfigResponseResult, ApiAddMemberPayload}, expression_evaluator};
use crate::errors;
use crate::helpers;
use crate::models;
use rocket::futures::stream::{StreamExt, TryStreamExt};
use async_recursion::async_recursion;

#[derive(Debug, Clone, PartialEq)]
pub struct Tenant {
    pub name: String
}

pub async fn get_instance() -> Result<mongodb::Client, mongodb::error::Error> {
    let client = Client::with_uri_str(constants::get_database_host().as_str()).await?;

    Ok(client)
}

// Checks if user already exists in the database. If it does, it is returned.
pub async fn get_user(client: &mongodb::Client, tenant: Tenant, email: String) -> Option<models::User> {
    let database = client.database(constants::DATABASE_NAME);
    let collection = database.collection::<Document>("users");

    match collection.count_documents(doc! {"email": email.to_owned(), "_tenant": tenant.name.to_owned()}, None).await {
        Ok(count) => {
            if count > 0 {
                return match collection.find_one(doc! {"email": email.to_owned(), "_tenant": tenant.name.to_owned()}, None).await {
                    Ok(res) => match res {
                        Some(doc) => Some(models::User {
                            email: doc.get_str("email").ok()?.to_string(),
                            id: doc.get_object_id("_id").expect("Should always exist").to_string()
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

// Only endpoint that doesn't need an input tenant
pub async fn new_user(client: &mongodb::Client, email: String) -> bool {
    let database = client.database(constants::DATABASE_NAME);
    let collection = database.collection::<Document>("users");
    let tenant_collection = database.collection::<Document>("tenants");

    
    match collection.insert_one(doc! { "email": email.clone(), "_tenant": email.clone() }, None).await {
        Ok(res) => {
            let user_id = res.inserted_id;
                // Create a tenant specific for this user.
                match tenant_collection.find_one(doc! {"name": email.to_owned()}, None).await {
                    Ok(res) => {
                        if res.is_some() {
                            // Tenant already exists. Abort
                            false
                        } else {
                            // Continue
                            match tenant_collection.insert_one(doc! { "name": email.clone(), "allowedUsers": [email] }, None).await {
                                Ok(res) => {
                                    let _tenant_id = res.inserted_id;

                                    true
                                },
                                Err(err) => {
                                    // TODO
                                    false
                                }
                            }

                        }
                    },
                    Err(_) => {
                        // TODO
                        false
                    }
                }
        },
        Err(err) => {
            // TODO
            false
        }
    }
}

// Checks if a project already exists in the databse. If it does, it is returned.
pub async fn get_project_by_title(
    client: mongodb::Client,
    tenant: Tenant,
    title: String,
) -> Option<models::Project> {
    let database = client.database(constants::DATABASE_NAME);
    let collection = database.collection::<Document>("projects");

    match collection.count_documents(doc! {"title": title.to_owned(), "_tenant": tenant.name.to_owned()}, None).await {
        Ok(count) => {
            if count > 0 {
                return match collection.find_one(doc! {"title": title.to_owned(), "_tenant": tenant.name.to_owned()}, None).await {
                    Ok(res) => match res {
                        Some(doc) => Some(models::Project {
                            title: doc.get_str("title").ok()?.to_string(),
                            id: doc.get_str("_id").ok()?.to_string(),
                            related_tree_ids: helpers::convert_bson_objectid_array_to_str_array(
                                doc.get_array("related_tree_ids").ok()?.clone(),
                            ),
                            related_config_ids: helpers::convert_bson_objectid_array_to_str_array(
                                doc.get_array("related_config_ids").ok()?.clone(),
                            ),
                            selected_model: match doc.get_str("selectedModel").ok() {
                                Some(val) => Some(val.to_string()),
                                None => {
                                    None
                                }
                            },
                            selected_config: match doc.get_str("selectedConfig").ok() {
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

pub async fn get_project_by_id(client: &mongodb::Client, tenant: Tenant, id: String) -> Option<models::Project> {
    let database = client.database(constants::DATABASE_NAME);
    let collection = database.collection::<Document>("projects");
    println!("Searching for {}", id);
    let mongo_id = mongodb::bson::oid::ObjectId::parse_str(&id).expect("Checked");
    match collection.find_one(
        doc! {"_id": mongo_id, "_tenant": tenant.name.to_owned()},
        None,
    ).await {
        Ok(res) => match res {
            Some(doc) => {
                println!("Operating on {}", id);
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

                let selected_config = match doc.get_str("selectedConfig").ok() {
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

                let config_ids = match doc.get_array("related_config_ids").ok() {
                    Some(val) => val.clone(),
                    None => {
                        println!("Found record does not have related_config_ids");
                        Vec::new()
                    }
                };
                let returnres = Some(models::Project {
                    title: title.to_string(),
                    id: id.to_string(),
                    related_tree_ids: helpers::convert_bson_objectid_array_to_str_array(tree_ids),
                    related_config_ids: helpers::convert_bson_objectid_array_to_str_array(config_ids),
                    selected_model: selected_model,
                    selected_config: selected_config
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
pub async fn get_available_project_ids(client: &mongodb::Client, tenants: Vec<Tenant>) -> Result<Vec<String>, errors::DatabaseError> {
    let database = client.database(constants::DATABASE_NAME);
    let collection = database.collection::<Document>("projects");
    let mut resulting_ids = Vec::new();

    println!("Tenants: {:?}", tenants);
    let matched_records = collection.find(doc!{"_tenant": doc! {
        "$in": helpers::tenant_names_from_vec(tenants)
    }}, None).await;

    match matched_records {
        Ok(mut records) => {
            while let Some(record) = records.next().await {
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

pub async fn new_project(
    client: mongodb::Client,
    user_email: String,
    tenants: Vec<Tenant>,
    title: String,
    org_id: Option<String>
) -> Result<String, errors::DatabaseError> {
    let database = client.database(constants::DATABASE_NAME);
    let collection = database.collection::<Document>("projects");

    let desired_tenant = match org_id {
        Some(org_id) => {
            // Find tenant for org
            match get_tenant_for_org(&client, &org_id).await {
                Ok(tenant) => Ok(tenant),
                Err(err) => Err(err)
            }
        },
        None => {
            Ok(Tenant { name: user_email })
        }
    }?;

    let insert_result =
        collection.insert_one(doc! { "title": title, "related_tree_ids": [], "selectedModel": null, "_tenant": desired_tenant.name.to_owned() }, None).await?;
    let inserted_id = insert_result.inserted_id;

    match inserted_id.as_object_id().clone() {
        Some(oid) => Ok(oid.to_string()),
        None => Err(errors::DatabaseError {
            message: "No object ID found.".to_string(),
        }),
    }
}

pub async fn update_project(client: mongodb::Client, tenant: Tenant, project_data: &models::Project) -> Result<models::Project, errors::DatabaseError> {
    let database = client.database(constants::DATABASE_NAME);
    let projects_collection = database.collection::<Document>("projects");


    // Need to convert related config ids and related tree ids to oids before updating.
    let mut proj_data_copy = project_data.clone();
    let mut doc = proj_data_copy.clone().to_bson_doc();

    doc.insert("related_tree_ids", helpers::convert_str_array_to_objectid_array(proj_data_copy.related_tree_ids));
    doc.insert("related_config_ids", helpers::convert_str_array_to_objectid_array(proj_data_copy.related_config_ids));

    println!("{}", doc);

    match projects_collection.find_one_and_update(doc! {
        "_id":  mongodb::bson::oid::ObjectId::parse_str(&project_data.id).expect("Checked"),
        "_tenant": tenant.name.to_owned()
    }, doc! {
        "$set": doc
    }, None).await {
        Ok(val) => {
            println!("{:?}", val);
        },
        Err(err) => eprintln!("Update project failed with: {}", err)
    }


    match get_project_by_id(&client, tenant, project_data.clone().id).await {
        Some(proj) => {
            Ok(proj)
        },
        None => Err(errors::DatabaseError {
            message: "No project matching ID found.".to_string(),
        })
    }
}


pub async fn update_project_model(client: mongodb::Client, tenant: Tenant, project_id: String, modelId: String) -> Result<bool, errors::DatabaseError> {
    let database = client.database(constants::DATABASE_NAME);
    let project_collection = database.collection::<Document>("projects");

    
    let matched_record = project_collection.find_one(
        doc! {
            "_id": mongodb::bson::oid::ObjectId::parse_str(&project_id).expect("Checked"),
            "_tenant": tenant.name.to_owned()
        },
        None,
    ).await?;

    match matched_record {
        Some(record) => {
            let title = record.get_str("title").expect("infalliable");
            let tree_ids = record.get_array("related_tree_ids").expect("infalliable");
            let new_doc = doc! {
                "title": title, "related_tree_ids": tree_ids, "selectedModel": modelId, "_tenant": tenant.name.to_owned()
            };

            let _result = project_collection.find_one_and_update(doc! {
                "_id": mongodb::bson::oid::ObjectId::parse_str(&project_id).expect("Checked"),
                "_tenant": tenant.name.to_owned()
            }, doc! {
                "$set": new_doc
            }, None).await;

            Ok(true)
        },
        None => {
            return Err(errors::DatabaseError {
                message: "Could not find project with _id = {}".to_string(),
            });
        }
    }

    
}

pub async fn create_project_tree(
    client: mongodb::Client,
    tenant: Tenant,
    title: String,
    project_id: String,
) -> Result<String, errors::DatabaseError> {
    let database = client.database(constants::DATABASE_NAME);
    let trees_collection = database.collection::<Document>("trees");
    let project_collection = database.collection::<Document>("projects");

    let insert_result = trees_collection.insert_one(doc! {
        "title": title,
        "rootNodeId": "", // Start with no root
        "_tenant": tenant.name.to_owned()
    }, None).await?;
    let inserted_id = insert_result.inserted_id.as_object_id();

    match inserted_id.clone() {
        Some(oid) => {
            project_collection.find_one_and_update(
                doc! {
                    "_id": mongodb::bson::oid::ObjectId::parse_str(&project_id).expect("Checked"),
                    "_tenant": tenant.name.to_owned()
                },
                doc! {
                    "$push": {
                        "related_tree_ids": oid.clone()
                    }
                },
                None,
            ).await?;

            Ok(oid.to_string().clone())
        },
        None => Err(errors::DatabaseError {
            message: "No object ID found.".to_string(),
        }),
    }
}

async fn get_tree_items_from_tree_ids(client: &mongodb::Client, tenant: Tenant, tree_ids: Vec<String>) -> Vec<models::ListTreeResponseItem> {
    let database = client.database(constants::DATABASE_NAME);
    let trees_collection = database.collection::<Document>("trees");
    let mut result = Vec::new();

    for tree_id in tree_ids {
        let matched_records = trees_collection.find(
            doc! {
                "_id": mongodb::bson::oid::ObjectId::parse_str(&tree_id).expect("Checked"),
                "_tenant": tenant.name.to_owned()
            },
            None,
        ).await;

        match matched_records {
            Ok(mut records) => {
                while let Some(record) = records.next().await {
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
async fn get_full_tree_data(client: &mongodb::Client, tenant: Tenant, tree_id: String, project_id: &String) -> Result<models::ApiFullComputedTreeData, errors::DatabaseError> {
    let database = client.database(constants::DATABASE_NAME);
    let trees_collection = database.collection::<Document>("trees");

    let matched_record = trees_collection.find_one(
        doc! {
            "_id": mongodb::bson::oid::ObjectId::parse_str(&tree_id).expect("Checked"),
            "_tenant": tenant.name.to_owned()
        },
        None,
    ).await?;

    match matched_record {
        Some(tree_record) => {
            let empty_bson_array = mongodb::bson::Array::new();
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

                        let mut condition_resolved = true; // Default to true
                        if condition_attribute.is_some() {
                            let config = get_selected_config(client, tenant.clone(), project_id).await;

                            match config {
                                Ok(config) => {
                                    condition_resolved = expression_evaluator::evaluate(condition_attribute.expect("Already checked"), &config);
                                },
                                Err(err) => {
                                    eprintln!("No config!");
                                    condition_resolved = false;
                                }
                            }
                        }


                        nodes_vec.push(models::ApiFullComputedNodeData {
                            id: id.to_owned(),
                            title: title.to_owned(),
                            description: description.to_owned(),
                            conditionAttribute: condition_attribute.unwrap_or("").to_owned(),
                            conditionResolved: condition_resolved,
                            children: children.unwrap_or(Vec::new()),
                            modelAttributes: model_attributes.unwrap_or(HashMap::new())
                        })
                    },
                    None => {
                        eprint!("nodes should be an array of objects, but isn't!")
                    }
                }

            }
            Ok(models::ApiFullComputedTreeData {
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

pub async fn get_trees_by_project_id(
    client: &mongodb::Client,
    tenant: Tenant,
    project_id: String,
) -> Result<Vec<models::ListTreeResponseItem>, errors::DatabaseError> {

    let matched_project = get_project_by_id(client, tenant.clone(), project_id.to_owned()).await;

    match matched_project {
        Some(project) => {
            let tree_ids = project.related_tree_ids;
            let trees = get_tree_items_from_tree_ids(client, tenant.clone(), tree_ids).await;

            Ok(trees)
        }
        None => Err(errors::DatabaseError {
            message: "Failed to search for matching trees".to_string(),
        }),
    }
}

pub async fn get_tree_by_id(
    client: &mongodb::Client,
    tenant: Tenant,
    tree_id: String,
    project_id: String
) -> Result<models::ApiFullComputedTreeData, errors::DatabaseError> {
    get_full_tree_data(client, tenant, tree_id, &project_id).await

}

pub async fn update_tree_by_id(
    client: &mongodb::Client,
    tenant: Tenant,
    tree_id: String,
    project_id: String,
    tree_data: models::ApiFullTreeData
) -> Result<models::ApiFullComputedTreeData, errors::DatabaseError> {
    let database = client.database(constants::DATABASE_NAME);
    let trees_collection = database.collection::<Document>("trees");

    let doc = tree_data.to_bson_doc();

    let _result = trees_collection.find_one_and_update(doc! {
        "_id": mongodb::bson::oid::ObjectId::parse_str(&tree_id).expect("Checked"),
        "_tenant": tenant.name.to_owned()
    }, doc!{
        "$set": doc
    }, None).await;


    get_full_tree_data(client, tenant, tree_id, &project_id).await
}

// DANGER: Not tenantized
pub async fn get_publicity_for_tree_by_id(
    client: &mongodb::Client,
    tree_id: String
) -> Result<bool, errors::DatabaseError> {
    let database = client.database(constants::DATABASE_NAME);
    let tree_access_collection = database.collection::<Document>("tree_access_control");

    let access_control_data = tree_access_collection.find_one(doc! {
        "treeId": mongodb::bson::oid::ObjectId::parse_str(&tree_id).expect("Checked")
    }, None).await;

    match access_control_data {
        Ok(access_control_data) => {
            match access_control_data {
                Some(access_control_data) => {
                    if access_control_data.contains_key("isPublic") && access_control_data.get_bool("isPublic").expect("Checked") {
                        return Ok(true);
                    }

                    return Ok(false);
                },
                None => {
                    return Ok(false);
                }
            }
        },
        Err(_err) => {
            return Err(DatabaseError{
                message: "Error looking up access_control_data".to_string()
            });
        }
    }
}

pub async fn set_publicity_for_tree_by_id(
    client: &mongodb::Client,
    tenants: Vec<Tenant>,
    tree_id: String,
    publicity: bool
) -> Result<bool, errors::DatabaseError> {
    let database = client.database(constants::DATABASE_NAME);
    let tree_access_collection = database.collection::<Document>("tree_access_control");

    let tree_tenant = get_tenant_for_tree(client, &tree_id).await.expect("Always exists");

    match tree_access_collection.update_one(doc! {
        "treeId": mongodb::bson::oid::ObjectId::parse_str(&tree_id).expect("Checked"),
        "_tenant": doc! {
            "$in": helpers::tenant_names_from_vec(tenants)
        }
    }, doc ! {
        "$set": {
            "treeId": mongodb::bson::oid::ObjectId::parse_str(&tree_id).expect("Checked"),
            "isPublic": publicity,
            "_tenant": tree_tenant.name.to_owned()
        }
    }, mongodb::options::UpdateOptions::builder()
    .upsert(Some(true))
    .build()).await {
        Ok(access_control_data) => {
            Ok(publicity)
        },
        Err(err) => {
            eprintln!("{}", err);
            return Err(DatabaseError{
                message: "Error looking up access_control_data".to_string()
            });
        }
    }
}


pub async fn get_projects_from_ids(client: &mongodb::Client, tenants: Vec<Tenant>, ids: Vec<String>) -> Vec<models::ApiProjectsListProjectItem> {
    let mut result = Vec::new();

    
    for id in ids.clone() {
        for tenant in tenants.clone() {
            let project_data = get_project_by_id(client, tenant.clone(), id.clone()).await;
            match project_data {
                Some(project_data) => {
                    result.push(models::ApiProjectsListProjectItem {
                        projectId: project_data.id,
                        name: project_data.title,
                        orgId: get_org_id_from_tenant(client, &tenant).await
                    });
                    break
                },
                None => { /* Skip */ }
            }
        }
    }

    result
}

pub async fn get_tree_from_node_id(client: &mongodb::Client, tenant: Tenant, node_id: String) -> Result<models::ApiGetNodeResponse, errors::DatabaseError> {
    let database = client.database(constants::DATABASE_NAME);
    let trees_collection = database.collection::<Document>("trees");

    match trees_collection.find_one(doc! {
        "nodes": {
            "$elemMatch": {
                "id": node_id.to_string()
            }
        },
        "_tenant": tenant.name.to_owned()
    }, None).await {
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

#[async_recursion]
pub async fn get_tree_relationships_down(client: &mongodb::Client, tenant: Tenant, startTreeId: &String, projectId: &String, mut seen_tree_ids: HashSet<String>) -> Vec<ApiTreeDagItem> {
    let mut result = vec![];

    if seen_tree_ids.contains(startTreeId) {
        return result;
    } else {
        seen_tree_ids.insert(startTreeId.to_string());
    }


    let childrenNodes = get_nodes_from_tree(client, tenant.clone(), startTreeId, projectId).await;

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
        let lookup = get_tree_from_node_id(client, tenant.clone(), node.to_string()).await;
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
        match get_tree_by_id(&client, tenant.clone(), childTree.to_string(), projectId.to_string()).await {
            Ok(tree_data) => {
                result.push(ApiTreeDagItem { id: childTree.to_string(), title: tree_data.title, children: get_tree_relationships_down(client, tenant.clone(), childTree, projectId, seen_tree_ids.clone()).await });
            },
            Err(err) => {
                // Ignore
            }
        }
        
    }

    result
}

pub async fn get_nodes_from_tree(client: &mongodb::Client, tenant: Tenant, treeId: &String, projectId: &String) -> Vec<models::ApiFullComputedNodeData> {
    let data = get_full_tree_data(client, tenant, treeId.to_string(), projectId).await;

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

pub async fn get_configs_for_project(client: &mongodb::Client, tenant: Tenant, project_id: &String,) -> Vec<String> {
    let project = get_project_by_id(client, tenant, project_id.to_string()).await;

    match project {
        Some(project) => {
            project.related_config_ids
        },
        None => vec![]
    }
}

pub async fn new_config(client: &mongodb::Client, tenant: Tenant, project_id: &String, body: &models::ApiProjectConfigPayload) -> Result<String, errors::DatabaseError> {
    let database = client.database(constants::DATABASE_NAME);
    let config_collection = database.collection::<Document>("configs");
    let project_collection = database.collection::<Document>("projects");

    let bson_attributes = mongodb::bson::to_bson(&body.attributes);

    match bson_attributes {
        Ok(bson_attributes) => {
            let insert_result = config_collection.insert_one(doc! {
                "attributes": bson_attributes,
                "_tenant": tenant.name.to_owned()
            }, None).await?;
        
            let inserted_id = insert_result.inserted_id.as_object_id();
        
            match inserted_id.clone() {
                Some(oid) => {
                    project_collection.find_one_and_update(
                        doc! {
                            "_id": mongodb::bson::oid::ObjectId::parse_str(&project_id).expect("Checked"),
                            "_tenant": tenant.name.to_owned()
                        },
                        doc! {
                            "$push": {
                                "related_config_ids": oid.clone()
                            }
                        },
                        None,
                    ).await?;
        
                    Ok(oid.to_string().clone())
                },
                None => Err(errors::DatabaseError {
                    message: "No object ID found.".to_string(),
                }),
            }
        },
        Err(err) => {
            Err(errors::DatabaseError {
                message: "New config body to json failed.".to_string(),
            })
        }
    }


}

pub async fn update_config(client: &mongodb::Client, tenant: Tenant, project_id: &String, config_id: &String, body: &models::ApiProjectConfigPayload) -> Result<String, errors::DatabaseError> {
    let database = client.database(constants::DATABASE_NAME);
    let config_collection = database.collection::<Document>("configs");
    let project_collection = database.collection::<Document>("projects");

    let bson_attributes = mongodb::bson::to_bson(&body.attributes);

    match bson_attributes {
        Ok(bson_attributes) => {
            let new_doc = doc! {
                "attributes": bson_attributes,
                "name": body.name.clone(),
                "_tenant": tenant.name.to_owned()
            };
        
            let doc = body;
        
            let _result = config_collection.find_one_and_replace(doc! {
                "_id": mongodb::bson::oid::ObjectId::parse_str(&config_id).expect("Checked"),
                "_tenant": tenant.name.to_owned()
            }, new_doc, None).await;
        
            Ok(config_id.to_owned())
        },
        Err(err) => {
            Err(DatabaseError { message: "Could not convert body attribute to JSON".to_owned() })
        }
    }
}

pub async fn get_selected_config(client: &mongodb::Client, tenant: Tenant, project_id: &String) -> Result<models::ApiProjectConfigResponseResult, errors::DatabaseError>  {
    let database = client.database(constants::DATABASE_NAME);
    let config_collection = database.collection::<Document>("configs");
    let project_collection = database.collection::<Document>("projects");

    let project = get_project_by_id(client, tenant.clone(), project_id.to_string()).await;

    match project {
        Some(project) => {
            let config_id = project.selected_config;

            match config_id {
                Some(config_id) => {
                    let matched_record = config_collection.find_one(
                        doc! {
                            "_id": mongodb::bson::oid::ObjectId::parse_str(&config_id).expect("Checked"),
                            "_tenant": tenant.clone().name.to_owned()
                        },
                        None,
                    ).await;

                    match matched_record {
                        Ok(matched_record) => {
                            match matched_record {
                                Some(matched_record) => {
                                    let attributes = matched_record.get_document("attributes").expect("Should always exist");
                                    let name = matched_record.get_str("name");

                                    Ok(ApiProjectConfigResponseResult {
                                        id: matched_record.get_object_id("_id").expect("Should always exist").to_string(),
                                        attributes: serde_json::json!(attributes),
                                        name: match name {
                                            Ok(_) => Some(name.expect("Checked").to_string()),
                                            Err(err) => None
                                        }
                                    })
                                },
                                None => Err(DatabaseError { message: "No matched record".to_string()})
                            }
                        },
                        Err(err) => Err(DatabaseError { message: format!("{}", err)})
                    }


                },
                None => Err(DatabaseError { message: "Could not find selected config".to_owned() })
            }
        },
        None => {
            Err(DatabaseError { message: "Could not find project".to_owned() })
        }
    }
}

pub async fn update_project_selected_config(client: &mongodb::Client, tenant: Tenant, project_id: &String, config: &models::ApiProjectConfigIdPayload) -> Result<models::ApiProjectConfigResponseResult, errors::DatabaseError> {
    let database = client.database(constants::DATABASE_NAME);
    let project_collection = database.collection::<Document>("projects");

    let new_doc = doc! {
        "$set": {
            "selectedConfig": config.desiredConfig.clone()
        }
    };

    let res = project_collection.find_one_and_update(doc! {
        "_id": mongodb::bson::oid::ObjectId::parse_str(&project_id).expect("Checked"),
        "_tenant": tenant.name.to_owned()
    }, new_doc, None).await;

    match res {
        Ok(res) => {
            match res {
                Some(res) => {
                    get_selected_config( client, tenant, project_id).await
                },
                None => Err(DatabaseError { message: "Could not find project to update".to_owned() })
            }
        },
        Err(err) => Err(DatabaseError { message: format!("{}", err)})
    }
}

pub async fn get_config(client: &mongodb::Client, tenant: Tenant, project_id: &String, config_id: &String) -> Result<models::ApiProjectConfigResponseResult, errors::DatabaseError> {
    let database = client.database(constants::DATABASE_NAME);
    let config_collection = database.collection::<Document>("configs");
    let project_collection = database.collection::<Document>("projects");


    let matched_record = config_collection.find_one(
        doc! {
            "_id": mongodb::bson::oid::ObjectId::parse_str(&config_id).expect("Checked"),
            "_tenant": tenant.name.to_owned()
        },
        None,
    ).await;

    match matched_record {
        Ok(matched_record) => {
            match matched_record {
                Some(matched_record) => {
                    let attributes = matched_record.get_document("attributes").expect("Should always exist");

                    Ok(ApiProjectConfigResponseResult {
                        id: matched_record.get_object_id("_id").expect("Should always exist").to_string(),
                        attributes: serde_json::json!(attributes),
                        name: match matched_record.get_str("name") {
                            Ok(res) => Some(res.to_string()),
                            Err(err) => None 
                        }
                    })
                },
                None => Err(DatabaseError { message: "No matched record".to_string()})
            }
        },
        Err(err) => Err(DatabaseError { message: format!("{}", err)})
    }
}

pub async fn store_csrf_token(token: &openidconnect::CsrfToken, nonce: &Nonce, client: &mongodb::Client) -> Result<bool, DatabaseError> {
    let database = client.database(constants::DATABASE_NAME);
    let csrf_collection = database.collection::<Document>("csrf_tokens");

    csrf_collection.insert_one(doc! {
        "csrf": token.secret(),
        "nonce": nonce.secret()
    }, None).await?;

    Ok(true)
}

// Deletes token at time of validation. i.e. good for one use
// Returns Nonce if validated, throws an error otherwise.
pub async fn validate_csrf_token(state_parameter: &String, client: &mongodb::Client) -> Result<Nonce, DatabaseError> {
    let database = client.database(constants::DATABASE_NAME);
    let csrf_collection = database.collection::<Document>("csrf_tokens");

    let result = csrf_collection.find_one_and_delete(doc! {
        "csrf": state_parameter
    }, None).await?;

    match result {
        Some(result) => {
            let nonce = result.get_str("nonce").map_err(|e| DatabaseError {
                message: "No nonce".to_owned()
            })?;
            Ok(Nonce::new(nonce.to_owned()))
        },
        None => {
            Err(DatabaseError {
                message: "No matching CSRF Token!".to_owned(),
            })
        }
    }
}

pub async fn store_history_record(client: &mongodb::Client, tenant: Tenant, id: String, data: models::ApiFullTreeData ) {
    let database = client.database(constants::DATABASE_NAME);
    let history_collection = database.collection::<Document>("history");

    let existing_records = history_collection.find(doc! {
        "record_id": id.clone(),
        "_tenant": tenant.name.to_owned()
    }, None).await;

    let mut highest_version_number = 0;

    match existing_records {
        Ok(mut records) => {
            while let Some(record) = records.next().await {
                match record {
                    Ok(record) => {
                        let version_number = record.get_i32("version_number").expect("Should exist");
                        if version_number > highest_version_number {
                            highest_version_number = version_number;
                        }
                    },
                    Err(err) => {
                        eprintln!("{}", err);
                    }
                }
            }
        },
        Err(err) => {
            eprintln!("{}", err)
        }
    }

    let next_version_number = highest_version_number + 1;


    history_collection.insert_one(doc! {
        "record_id": id,
        "version_number": next_version_number,
        "data": data.to_bson_doc(),
        "_tenant": tenant.name.to_owned()
    }, None).await.expect("To insert");

}

pub async fn move_backwards_in_history(client: &mongodb::Client, tenant: Tenant, id: String) -> Option<Document> {
    let database = client.database(constants::DATABASE_NAME);
    let history_collection = database.collection::<Document>("history");

    let existing_records = history_collection.find(doc! {
        "record_id": id.clone(),
        "_tenant": tenant.name.to_owned()
    }, None).await;

    let mut highest_version_number = 0;
    let mut relevant_record = None;
    let mut second_relevant_record = None;


    match existing_records {
        Ok(mut records) => {
            while let Some(record) = records.next().await {
                match record {
                    Ok(record) => {
                        let version_number = record.get_i32("version_number").expect("Should exist");
                        if version_number > highest_version_number {
                            highest_version_number = version_number;
                            second_relevant_record = relevant_record;
                            relevant_record = Some(record);
                        }
                    },
                    Err(err) => {
                        eprintln!("{}", err);
                    }
                }
            }
        },
        Err(err) => {
            eprintln!("{}", err)
        }
    }

    if highest_version_number <= 1 {
        None
    } else {

        match relevant_record {
            Some(record) => {
                match history_collection.delete_one(doc! {
                    "version_number": highest_version_number,
                    "record_id": id,
                    "_tenant": tenant.name.to_owned()
                }, None).await {
                    Ok(_) => {
                        match second_relevant_record {
                            Some(record) => {
                                Some(record.get_document("data").expect("Should always exist").clone())
                            },
                            None => {
                                eprintln!("No second record was found.");
                                None
                            }
                        }
                        
                    },
                    Err(err) => {
                        eprintln!("{}", err);
                        None
                    }
                }
            },
            None => None
        }
    }
}

// When creating an org we create a new tenant
pub async fn create_org(client: &mongodb::Client, tenant: Tenant, data: &models::ApiOrgMetadataBase ) -> Result<String, DatabaseError> {
    let database = client.database(constants::DATABASE_NAME);
    let org_collection = database.collection::<Document>("organizations");
    let tenant_collection = database.collection::<Document>("tenants");

    // Generate a new tenant id
    let new_tenant: String = rand::thread_rng()
    .sample_iter(&Alphanumeric)
    .take(7)
    .map(char::from)
    .collect();

    // Default plan to standard which will allow no additional users.
    let mut plan: String = "standard".to_owned();
    let plan_options = vec!["standard".to_owned(), "organization".to_owned(), "public-good".to_owned()];

    if (data.plan.is_some() && plan_options.contains(&(data.plan.clone().expect("Checked")))) {
        plan = data.plan.clone().expect("Checked");
    }

    let insert_result =
    org_collection.insert_one(doc! { "name": data.name.clone(), "plan": plan, "_tenant": new_tenant.clone() }, None).await?;
    let inserted_id = insert_result.inserted_id;

    // Give requesting user access to this new tenant...
    println!("Tenant (email): {}", tenant.name.clone());
    tenant_collection.insert_one(doc! { "name": new_tenant, "allowedUsers": [tenant.name] }, None).await?;

    match inserted_id.as_object_id().clone() {
        Some(oid) => Ok(oid.to_string()),
        None => Err(errors::DatabaseError {
            message: "No object ID found.".to_string(),
        }),
    }


}

pub async fn update_org(client: &mongodb::Client, tenants: Vec<Tenant>, org_id: String, data: &models::ApiOrgMetadataBase) -> Result<String, DatabaseError> {
    let database = client.database(constants::DATABASE_NAME);
    let org_collection = database.collection::<Document>("organizations");


    // Verify access to org
    let needed_tenant = get_tenant_for_org(client, &org_id).await;

    let mut new_doc = doc! {
        "$set": {
            "name": data.name.clone()
        }
    };

    if data.plan.is_some() {
        new_doc.get_document_mut("$set").expect("Just added").insert("plan", data.plan.clone().expect("Checked"));
    }

    match needed_tenant {
        Ok(needed_tenant) => {
            if tenants.contains(&needed_tenant) {
                // Update org
                match org_collection.update_one(doc! {
                    "_id": mongodb::bson::oid::ObjectId::parse_str(&org_id).expect("Checked"),
                    "_tenant": needed_tenant.name.to_owned()
                }, new_doc, None).await {
                    Ok(_) => {
                        Ok(org_id)
                    },
                    Err(err) => {
                        Err(errors::DatabaseError {
                            message: "Database error when trying to update org".to_string()
                        })
                    }
                }
            } else {
                Err(errors::DatabaseError {
                    message: "No access to org".to_string()
                })
            }
        },
        Err(err) => {
            Err(errors::DatabaseError {
                message: "Finding tenant for org failed.".to_string()
            })
        }
    }
}

pub async fn delete_project_by_id(client: &mongodb::Client, tenants: Vec<Tenant>, project_id: String) -> Result<bool, DatabaseError> {
    let database = client.database(constants::DATABASE_NAME);
    let project_collection = database.collection::<Document>("projects");

        // Verify access to org
        let needed_tenant = get_tenant_for_project(client, &project_id).await;

        match needed_tenant {
            Ok(needed_tenant) => {
                if tenants.contains(&needed_tenant) {
                    // Delete org
                    match project_collection.delete_one(doc! {
                        "_id": mongodb::bson::oid::ObjectId::parse_str(&project_id).expect("Checked"),
                        "_tenant": needed_tenant.name.to_owned()
                    }, None).await {
                        Ok(_) => {
                            Ok(true)
                        },
                        Err(err) => {
                            Err(errors::DatabaseError {
                                message: "Database error when trying to delete org".to_string()
                            })
                        }
                    }
                } else {
                    Err(errors::DatabaseError {
                        message: "No access to org".to_string()
                    })
                }
            },
            Err(err) => {
                Err(errors::DatabaseError {
                    message: "Finding tenant for org failed.".to_string()
                })
            }
        }
    
}

pub async fn delete_org(client: &mongodb::Client, tenants: Vec<Tenant>, org_id: String) -> Result<bool, DatabaseError> {
    let database = client.database(constants::DATABASE_NAME);
    let org_collection = database.collection::<Document>("organizations");


    // Verify access to org
    let needed_tenant = get_tenant_for_org(client, &org_id).await;

    match needed_tenant {
        Ok(needed_tenant) => {
            if tenants.contains(&needed_tenant) {
                // Delete org
                match org_collection.delete_one(doc! {
                    "_id": mongodb::bson::oid::ObjectId::parse_str(&org_id).expect("Checked"),
                    "_tenant": needed_tenant.name.to_owned()
                }, None).await {
                    Ok(_) => {
                        Ok(true)
                    },
                    Err(err) => {
                        Err(errors::DatabaseError {
                            message: "Database error when trying to delete org".to_string()
                        })
                    }
                }
            } else {
                Err(errors::DatabaseError {
                    message: "No access to org".to_string()
                })
            }
        },
        Err(err) => {
            Err(errors::DatabaseError {
                message: "Finding tenant for org failed.".to_string()
            })
        }
    }
}

pub async fn get_orgs(client: &mongodb::Client, tenants: Vec<Tenant>) -> Result<Vec<models::ApiOrgMetadata>, DatabaseError> {
    let database = client.database(constants::DATABASE_NAME);
    let org_collection = database.collection::<Document>("organizations");
    let mut result: Vec<models::ApiOrgMetadata> = vec![];

    for tenant in tenants {
        let relevant_orgs = org_collection.find(doc! {
            "_tenant": tenant.name.to_owned()
        }, None).await;

        match relevant_orgs {
            Ok(mut relevant_orgs) => {
                while let Some(record) = relevant_orgs.next().await {
                    if record.is_ok() {
                        result.push(models::ApiOrgMetadata {
                            name: record.clone().expect("checked").get_str("name").expect("Assert").to_owned(),
                            id: record.clone().expect("checked").get_object_id("_id").expect("Assert").to_string(),
                            plan: record.clone().expect("checked").get_str("plan").unwrap_or("standard").to_owned()
                        })
                    }
                }
            },
            Err(err) => {}
        }

    }

    Ok(result)
}

pub async fn get_tenants_for_user(client: &mongodb::Client, email: &String) -> Vec<Tenant> {
    let database = client.database(constants::DATABASE_NAME);
    let tenant_collection = database.collection::<Document>("tenants");
    let mut tenants = vec![];

    match tenant_collection.find(doc! {
        "allowedUsers": email.to_owned()
    }, None).await {
        Ok(mut res) => {
            while let Some(record) = res.next().await {
                if record.is_ok() {
                    tenants.push(Tenant { name: record.expect("checked").get_str("name").expect("Should always exist").to_string() })
                }
            }
        },
        Err(err) => {
            eprintln!("{}", err)
        }
    }

    tenants
}

pub async fn get_tenant_for_org(client: &mongodb::Client, org_id: &String) -> Result<Tenant, DatabaseError> {
    let database = client.database(constants::DATABASE_NAME);
    let org_collection = database.collection::<Document>("organizations");

    let org = org_collection.find_one(doc! {
        "_id": mongodb::bson::oid::ObjectId::parse_str(&org_id).expect("Checked"),
    }, None).await;

    match org {
        Ok(org) => {
            match org {
                Some(doc) => {
                    Ok(Tenant {
                        name: doc.get_str("_tenant").expect("To exist").to_owned()
                    })
                },
                None => Err(DatabaseError {
                    message: "Lookup failed for org".to_owned()
                })
            }
        },
        Err(err) => Err(DatabaseError {
            message: "Database connection error".to_owned()
        })
    }
}

pub async fn get_tenant_for_project(client: &mongodb::Client, project_id: &String) -> Result<Tenant, DatabaseError>  {
    let database = client.database(constants::DATABASE_NAME);
    let project_collection = database.collection::<Document>("projects");

    let org = project_collection.find_one(doc! {
        "_id": mongodb::bson::oid::ObjectId::parse_str(&project_id).expect("Checked"),
    }, None).await;

    match org {
        Ok(org) => {
            match org {
                Some(doc) => {
                    Ok(Tenant {
                        name: doc.get_str("_tenant").expect("To exist").to_owned()
                    })
                },
                None => Err(DatabaseError {
                    message: "Lookup failed for org".to_owned()
                })
            }
        },
        Err(err) => Err(DatabaseError {
            message: "Database connection error".to_owned()
        })
    }
}

pub async fn get_tenant_for_tree(client: &mongodb::Client, tree_id: &String) -> Result<Tenant, DatabaseError> {
    let database = client.database(constants::DATABASE_NAME);
    let tree_collection = database.collection::<Document>("trees");

    let tree = tree_collection.find_one(doc! {
        "_id": mongodb::bson::oid::ObjectId::parse_str(&tree_id).expect("Checked"),
    }, None).await;

    match tree {
        Ok(tree) => {
            match tree {
                Some(doc) => {
                    Ok(Tenant {
                        name: doc.get_str("_tenant").expect("To exist").to_owned()
                    })
                },
                None => Err(DatabaseError {
                    message: "Lookup failed for tree".to_owned()
                })
            }
        },
        Err(err) => Err(DatabaseError {
            message: "Database connection error".to_owned()
        })
    }
}

pub async fn get_org_id_from_tenant(client: &mongodb::Client, tenant: &Tenant) -> Option<String> {
    let database = client.database(constants::DATABASE_NAME);
    let org_collection = database.collection::<Document>("organizations");

    let org = org_collection.find_one(doc! {
        "_tenant": tenant.name.clone(),
    }, None).await;

    match org {
        Ok(org) => {
            match org {
                Some(org) => Some(org.get_object_id("_id").expect("always exists").to_string()),
                None => None
            }
        },
        Err(err) => {
            eprintln!("{}", err);
            None
        }
    }
}

pub async fn get_max_user_count_in_org(client: &mongodb::Client, tenants: Vec<Tenant>, org_id: String) -> Result<usize, DatabaseError> {
    let database = client.database(constants::DATABASE_NAME);
    let tenant_collection = database.collection::<Document>("tenants");
    let org_collection = database.collection::<Document>("organizations");

    let org_tenant = get_tenant_for_org(client, &org_id).await?;

    // Check to make sure user has access to this tenant
    let mut has_access = false;
    for tenant in tenants {
        if tenant.name == org_tenant.name {
            has_access = true;
        }
    }

    if has_access {
        let org = org_collection.find_one(doc! {
            "_id": mongodb::bson::oid::ObjectId::parse_str(&org_id).expect("Checked"),
        }, None).await;

        match org {
            Ok(res) => {
                match res {
                    Some(doc) => {
                        let plan = doc.get_str("plan").unwrap_or("standard");
                        match plan {
                            "organization" => Ok(5),
                            "public-good" => Ok(1000),
                            _ => Ok(1)
                        }
                    },
                    None => Err(DatabaseError {
                        message: "Could not find tenant".to_owned()
                    })
                }
            },
            Err(err) => Err(DatabaseError {
                message: "Error finding tenant".to_owned()
            })
        }
    } else {
        Err(DatabaseError {
            message: "User does not have access to this org.".to_owned()
        })
    }

}

pub async fn get_user_count_in_org(client: &mongodb::Client, tenants: Vec<Tenant>, org_id: String) -> Result<usize, DatabaseError> {
    let database = client.database(constants::DATABASE_NAME);
    let tenant_collection = database.collection::<Document>("tenants");

    let org_tenant = get_tenant_for_org(client, &org_id).await?;

    // Check to make sure user has access to this tenant
    let mut has_access = false;
    for tenant in tenants {
        if tenant.name == org_tenant.name {
            has_access = true;
        }
    }

    if has_access {
        let res = tenant_collection.find_one(doc! {
            "name": org_tenant.name
        }, None).await;

        match res {
            Ok(res) => {
                match res {
                    Some(doc) => {
                        let allowed_users = doc.get_array("allowedUsers").expect("To exist");
                        return Ok(allowed_users.len())
                    },
                    None => Err(DatabaseError {
                        message: "Could not find tenant".to_owned()
                    })
                }
            },
            Err(err) => Err(DatabaseError {
                message: "Error finding tenant".to_owned()
            })
        }
    } else {
        Err(DatabaseError {
            message: "User does not have access to this org.".to_owned()
        })
    }

}

pub async fn add_user_to_org(client: &mongodb::Client, tenants: Vec<Tenant>, org_id: String, email: String) -> Result<String, DatabaseError> {
    let database = client.database(constants::DATABASE_NAME);
    let tenant_collection = database.collection::<Document>("tenants");

    let org_tenant = get_tenant_for_org(client, &org_id).await?;

    // Check to make sure user has access to this tenant
    let mut has_access = false;
    for tenant in tenants {
        if tenant.name == org_tenant.name {
            has_access = true;
        }
    }

    if has_access {
        let new_doc = doc! {
            "$push": {
                "allowedUsers": email.clone()
            }
        };

        let res = tenant_collection.find_one_and_update(doc! {
            "name": org_tenant.name
        }, new_doc, None).await;

        Ok(email)
    } else {
        Err(DatabaseError {
            message: "User does not have access to this org.".to_owned()
        })
    }
}

pub async fn remove_user_from_org(client: &mongodb::Client, tenants: Vec<Tenant>, org_id: String, email: String) -> Result<(), DatabaseError> {
    let database = client.database(constants::DATABASE_NAME);
    let tenant_collection = database.collection::<Document>("tenants");

    let org_tenant = get_tenant_for_org(client, &org_id).await?;

    // Check to make sure user has access to this tenant
    let mut has_access = false;
    for tenant in tenants {
        if tenant.name == org_tenant.name {
            has_access = true;
        }
    }

    if has_access {
        let new_doc = doc! {
            "$pull": {
                "allowedUsers": email.clone()
            }
        };

        let res = tenant_collection.find_one_and_update(doc! {
            "name": org_tenant.name
        }, new_doc, None).await;

        match res {
            Ok(res) => {
                println!("Removed okay");
            },
            Err(err) => {
                eprintln!("Removal failed: {}", err);
            }
        }

        Ok(())
    } else {
        Err(DatabaseError {
            message: "User does not have access to this org.".to_owned()
        })
    }
}


pub async fn filter_tenant_for_project(client: &mongodb::Client, tenants: Vec<Tenant>, project_id: String) -> Option<Tenant> {
    let database = client.database(constants::DATABASE_NAME);
    let project_collection = database.collection::<Document>("projects");

    let project = project_collection.find_one(doc! {
        "_id": mongodb::bson::oid::ObjectId::parse_str(&project_id).expect("Checked"),
        "_tenant": doc! {
            "$in": helpers::tenant_names_from_vec(tenants)
        }

    }, None).await;

    match project {
        Ok(project) => {
            match project {
                Some(project) => {
                    Some(Tenant { name: project.get_str("_tenant").expect("Should exist").to_owned() })
                },
                None => None
            }
        },
        Err(err) => {
            eprintln!("{}", err);
            None
        }
    }
}

pub async fn filter_tenant_for_node(client: &mongodb::Client, tenants: Vec<Tenant>, node_id: String) -> Option<Tenant> {
    let database = client.database(constants::DATABASE_NAME);
    let node_collection = database.collection::<Document>("nodes");

    let database = client.database(constants::DATABASE_NAME);
    let trees_collection = database.collection::<Document>("trees");

    let tree = trees_collection.find_one(doc! {
        "nodes": {
            "$elemMatch": {
                "id": node_id.to_string()
            }
        },
        "_tenant": doc! {
            "$in": helpers::tenant_names_from_vec(tenants)
        }
    }, None).await;



    match tree {
        Ok(tree) => {
            match tree {
                Some(tree) => {
                    Some(Tenant { name: tree.get_str("_tenant").expect("Should exist").to_owned() })
                },
                None => None
            }
        },
        Err(err) => {
            eprintln!("{}", err);
            None
        }
    }
}

pub async fn get_members_for_org(client: &mongodb::Client, org_id: String, tenants: Vec<Tenant>) -> Result<Vec<ApiAddMemberPayload>, DatabaseError> {
    let database = client.database(constants::DATABASE_NAME);
    let tenant_collection = database.collection::<Document>("tenants");

    let org_tenant = get_tenant_for_org(client, &org_id).await?;

    // Check to make sure user has access to this tenant
    let mut has_access = false;
    for tenant in tenants {
        if tenant.name == org_tenant.name {
            has_access = true;
        }
    }

    if has_access {
        let tenant = tenant_collection.find_one(doc! {
            "name": org_tenant.name
        }, None).await;

        match tenant {
            Ok(tenant) => {
                match tenant {
                    Some(tenant) => {
                        let emails = tenant.get_array("allowedUsers").expect("Should always exist");

                        let mut result = vec![];

                        for email in emails {
                            result.push(ApiAddMemberPayload {
                                email: email.as_str().expect("Always the case").to_owned()
                            });
                        }

                        Ok(result)
                    },
                    None => Err(DatabaseError {
                        message: "Could not find matching tenant".to_owned()
                    })
                }
            },
            Err(err) => Err(DatabaseError {
                message: "Could not find tenant due to database failure".to_owned()
            })
        }

        
    } else {
        Err(DatabaseError {
            message: "User does not have access to this org.".to_owned()
        })
    }
}

pub async fn delete_tree_by_id(client: &mongodb::Client, tenants: Vec<Tenant>, id: String) -> Result<bool, DatabaseError> {
    let database = client.database(constants::DATABASE_NAME);
    let tree_collection = database.collection::<Document>("trees");


    // Verify access to org
    let needed_tenant = get_tenant_for_tree(client, &id).await;

    match needed_tenant {
        Ok(needed_tenant) => {
            if tenants.contains(&needed_tenant) {
                // Delete tree
                match tree_collection.delete_one(doc! {
                    "_id": mongodb::bson::oid::ObjectId::parse_str(&id).expect("Checked"),
                    "_tenant": needed_tenant.name.to_owned()
                }, None).await {
                    Ok(_) => {
                        Ok(true)
                    },
                    Err(err) => {
                        Err(errors::DatabaseError {
                            message: "Database error when trying to delete tree".to_string()
                        })
                    }
                }
            } else {
                Err(errors::DatabaseError {
                    message: "No access to tree".to_string()
                })
            }
        },
        Err(err) => {
            Err(errors::DatabaseError {
                message: "Finding tenant for tree failed:".to_string() + &err.to_string()
            })
        }
    }
}