use serde::Deserialize;

pub struct User {
    pub email: String
}

#[derive(Deserialize)]
pub struct ApiRegisterUser {
    pub email: String
}
