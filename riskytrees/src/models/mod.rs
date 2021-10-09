use serde::{Serialize, Serializer, Deserialize};
use std::collections::HashMap;

#[derive(Serialize, Deserialize, Debug)]
pub struct ModelAttribute {
    pub value_string: String,
    pub value_int: i32,
    pub value_float: f64,
    pub value_type: String // str, int, float
}

#[derive(Serialize, Deserialize, Debug)]
pub struct User {
    pub email: String
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Project {
    pub title: String,
    pub id: String,
    pub related_tree_ids: Vec<String>
}

#[derive(Serialize, Deserialize, Debug)]
pub struct AuthLoginResponseResult {
    pub sessionToken: String
}

#[derive(Serialize, Deserialize, Debug)]
pub struct CreateProjectResponseResult {
    pub id: String,
    pub title: String
}

#[derive(Serialize, Deserialize, Debug)]
pub struct CreateTreeResponseResult {
    pub title: String,
    pub id: String
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ListTreeResponseResult {
    pub trees: Vec<ListTreeResponseItem>
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ListTreeResponseItem {
    pub title: String,
    pub id: String
}

#[derive(Serialize, Deserialize, Debug)]
pub struct NodeResponseResult {
    pub title: String,
    pub description: String,
    pub modelAttributes: HashMap<String, ModelAttribute>,
    pub conditionAttribute: String,
    pub parents: Vec<String>,
    pub id: String,
    pub children: Vec<String>
}

// Everything below is an OpenAPI structure

#[derive(Serialize, Deserialize)]
pub struct ApiRegisterUser {
    pub email: String
}

#[derive(Serialize, Deserialize)]
pub struct ApiAuthLoginResponse {
    pub ok: bool,
    pub message: String,
    pub result: Option<AuthLoginResponseResult>
}

#[derive(Serialize, Deserialize)]
pub struct ApiCreateProject {
    pub title: String
}

#[derive(Serialize, Deserialize)]
pub struct ApiCreateTree {
    pub title: String
}

#[derive(Serialize, Deserialize)]
pub struct ApiCreateNode {
    pub title: String,
    pub description: String,
    pub modelAttributes: HashMap<String, ModelAttribute>,
    pub conditionAttribute: String,
    pub parents: Vec<String>
}

#[derive(Serialize, Deserialize)]
pub struct ApiFullTreeData {
    pub title: String,
    pub modelAttributes: HashMap<String, ModelAttribute>,
    pub conditionAttribute: String,
    pub parents: Vec<String>,
    pub children: Vec<String>
}

// Responses

#[derive(Serialize, Deserialize)]
pub struct ApiListTreeResponse {
    pub ok: bool,
    pub message: String,
    pub result: Option<ListTreeResponseResult>
}

#[derive(Serialize, Deserialize)]
pub struct ApiTreeResponse {
    pub ok: bool,
    pub message: String,
    pub result: Option<ApiFullTreeData>
}

#[derive(Serialize, Deserialize)]
pub struct ApiCreateProjectResponse {
    pub ok: bool,
    pub message: String,
    pub result: Option<CreateProjectResponseResult>
}

#[derive(Serialize, Deserialize)]
pub struct ApiCreateTreeResponse {
    pub ok: bool,
    pub message: String,
    pub result: Option<CreateTreeResponseResult>
}

#[derive(Serialize, Deserialize)]
pub struct ApiNodeResponse {
    pub ok: bool,
    pub message: String,
    pub result: Option<NodeResponseResult>
}
