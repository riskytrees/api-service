use rocket::serde::json::Json;

use crate::{database, models};

pub async fn record_tree_update(client: &mongodb::Client, tree_id: String, tree_data: models::ApiFullTreeData) -> () {
    database::store_history_record(client, tree_id, tree_data).await
}