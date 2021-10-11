use serde::{Serialize, Serializer, Deserialize};
use std::collections::HashMap;
use bson::Bson;

use mongodb::{
    bson::{doc, Document},
    sync::Client,
};


#[derive(Serialize, Deserialize, Debug)]
pub struct ModelAttribute {
    pub value_string: String,
    pub value_int: i32,
    pub value_float: f64,
    pub value_type: String // str, int, float
}

impl ModelAttribute {
    fn to_bson_doc(self) -> Document {
        doc! {
            "value_int": self.value_int,
            "value_type": self.value_type.to_owned(),
            "value_float": self.value_float,
            "value_string": self.value_string.to_owned()
        }
    }
}

impl Clone for ModelAttribute {
    fn clone(&self) -> ModelAttribute {
        ModelAttribute {
            value_string: self.value_string.to_owned(),
            value_type: self.value_type.to_owned(),
            value_float: self.value_float,
            value_int: self.value_int
        }
    }
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

// Everything below is an OpenAPI structure or part of one

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
pub struct ApiFullNodeData {
    pub id: String,
    pub title: String,
    pub modelAttributes: HashMap<String, ModelAttribute>,
    pub conditionAttribute: String,
    pub parents: Vec<String>,
    pub children: Vec<String>
}

impl Clone for ApiFullNodeData {
    fn clone(&self) -> ApiFullNodeData {
        ApiFullNodeData {
            id: self.id.to_owned(),
            title: self.title.to_owned(),
            modelAttributes: self.modelAttributes.clone(),
            conditionAttribute: self.conditionAttribute.to_owned(),
            parents: self.parents.clone(),
            children: self.children.clone()
        }
    }
}

impl ApiFullNodeData {
    fn into_bson_doc(self) -> Document {
        let mut model_attributes = doc! {};

        for (key, val) in self.modelAttributes.into_iter() {
            model_attributes.insert(key, val.to_bson_doc());
        }


        doc! {
            "id": self.id,
            "title": self.title,
            "modelAttributes": model_attributes,
            "conditionAttribute": self.conditionAttribute,
            "parents": self.parents,
            "children": self.children
        }
    }
}

#[derive(Serialize, Deserialize)]
pub struct ApiFullTreeData {
    pub title: String,
    pub rootNodeId: String,
    pub nodes: Vec<ApiFullNodeData>
}

impl ApiFullTreeData {
    pub fn to_bson_doc(self) -> Document {
        let mut nodes_as_docs = Vec::new();

        for node in self.nodes {
            nodes_as_docs.push(node.into_bson_doc());
        }

        doc! {
            "title": self.title,
            "rootNodeId": self.rootNodeId,
            "nodes": nodes_as_docs
        }
    }
}

#[derive(Serialize, Deserialize)]
pub struct ApiProjectsListResponseResult {
    pub projects: Vec<String>
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
pub struct ApiProjectsListResponse {
    pub ok: bool,
    pub message: String,
    pub result: Option<ApiProjectsListResponseResult>
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
