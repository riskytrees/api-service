use rocket::serde::json::Json;

use crate::{database, models};

pub async fn record_tree_update(client: &mongodb::Client, tenant: database::Tenant, tree_id: String, tree_data: models::ApiFullTreeData) -> () {
    database::store_history_record(client, tenant, tree_id, tree_data).await
}

pub async fn move_back_tree_update(client: &mongodb::Client, tenant: database::Tenant, tree_id: String, project_id: String) -> Option<models::ApiFullComputedTreeData> {
    match database::move_backwards_in_history(client, tenant.clone(), tree_id.clone()).await {
        Some(history_doc) => {
            let tree_data: Result<models::ApiFullTreeData, mongodb::bson::de::Error>  = mongodb::bson::from_bson(mongodb::bson::Bson::Document(history_doc));

            match tree_data {
                Ok(res) => {
                    match database::update_tree_by_id(client, tenant, tree_id, project_id, res).await {
                        Ok(res) => {
                            Some(res)
                        },
                        Err(err) => {
                            eprintln!("{}", err);
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