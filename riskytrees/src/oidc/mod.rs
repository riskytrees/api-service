enum ProviderType {
    Google
}

struct AccessAndIdToken {
    pub access_token: String,
    pub id_token: String
}

pub fn create_csrf_token() -> String {
    todo!();
}

pub fn generate_oauth2_request(provider: ProviderType) {
    todo!();
}

pub fn verify_csrf_from_code(code: &String) {
    todo!();
}

pub fn get_tokens_from_code(code: &String) -> AccessAndIdToken {
    todo!();
}