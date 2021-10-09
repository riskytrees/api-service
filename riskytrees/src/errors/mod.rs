use std::fmt;

pub struct DatabaseError {
    pub message: String
}

impl fmt::Display for DatabaseError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Database error: {}", self.message)
    }
}


impl std::convert::From<mongodb::error::Error> for DatabaseError {
    fn from(mongo_error: mongodb::error::Error) -> DatabaseError {
        DatabaseError {
            message: mongo_error.to_string()
        }
    }
}
