use serde::{Serialize, Serializer, Deserialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct User {
    pub email: String
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Project {
    pub title: String,
    pub id: i32
}

#[derive(Serialize, Deserialize, Debug)]
pub struct AuthLoginResponseResult {
    pub sessionToken: String
}

#[derive(Serialize, Deserialize, Debug)]
pub struct CreateProjectResponseResult {
    pub id: i32,
    pub title: String
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
pub struct ApiCreateProjectResponse {
    pub ok: bool,
    pub message: String,
    pub result: Option<CreateProjectResponseResult>
}
