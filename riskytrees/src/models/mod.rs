use serde::{Serialize, Serializer, Deserialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct User {
    pub email: String
}

#[derive(Serialize, Deserialize, Debug)]
pub struct AuthLoginResponseResult {
    pub sessionToken: String
}

#[derive(Deserialize)]
pub struct ApiRegisterUser {
    pub email: String
}

#[derive(Serialize, Deserialize)]
pub struct ApiAuthLoginResponse {
    pub ok: bool,
    pub message: String,
    pub result: Option<AuthLoginResponseResult>
}
