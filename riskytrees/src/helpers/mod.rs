use mongodb::{
    bson::{doc, Document, Bson, oid::ObjectId}
};
use std::str::FromStr;

use crate::database::Tenant;

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

pub fn convert_str_array_to_objectid_array(str_array: Vec<String>) -> Vec<ObjectId> {
    let mut new_vec: Vec<ObjectId> = Vec::new();

    for val in str_array {
        let id = mongodb::bson::oid::ObjectId::from_str(&val).expect("Checked");
        new_vec.push(id);
    }

    return new_vec;
}
    

pub fn tenant_names_from_vec(tenants: Vec<Tenant>) -> Vec<String> {
    let mut result = vec![];

    for tenant in tenants {
        result.push(tenant.name);
    }

    result
}