use std::env;

pub const DATABASE_NAME: &str = "riskytrees";

pub fn get_database_host() -> String {
    match env::var("DATABASE_HOST") {
        Ok(val) => val,
        Err(err) => "mongodb://localhost:27017".to_owned()
    }
}
