use serde::{Serialize, Serializer, Deserialize};

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

// Responses

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
