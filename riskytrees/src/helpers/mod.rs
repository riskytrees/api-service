use mongodb::{
    bson::{doc, Document},
    sync::Client,
};

use bson::Bson;

pub fn convert_bson_str_array_to_str_array(bson_array: Vec<Bson>) -> Vec<String> {
    let mut new_vec: Vec<String> = Vec::new();

    for doc in bson_array {
        match doc.as_str() {
            Some(val) => {
                new_vec.push(val.to_string())
            },
            None => ()
        }
    }

    new_vec
}

pub fn convert_bson_objectid_array_to_str_array(bson_array: Vec<Bson>) -> Vec<String> {
    let mut new_vec: Vec<String> = Vec::new();

    for doc in bson_array {
        match doc.as_object_id() {
            Some(val) => {
                new_vec.push(val.to_string())
            },
            None => ()
        }
    }

    new_vec
}
