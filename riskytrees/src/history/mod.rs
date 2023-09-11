use rocket::serde::json::Json;

use crate::{database, models};

pub async fn record_tree_update(client: &mongodb::Client, tenant: database::Tenant, tree_id: String, tree_data: models::ApiFullTreeData) -> () {
    database::store_history_record(client, tenant, tree_id, tree_data).await
}