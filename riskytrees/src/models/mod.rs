use serde::{Serialize, Serializer, Deserialize};
use std::collections::HashMap;
use bson::Bson;

use mongodb::{
    bson::{doc, Document}
};


#[derive(Serialize, Deserialize, Debug)]
pub struct ModelAttribute {
    pub value_string: Option<String>,
    pub value_int: Option<i32>,
    pub value_float: Option<f64>,
}

impl ModelAttribute {
    fn to_bson_doc(self) -> Document {
        if self.value_string.is_some() {
            doc! {
                "value_int": mongodb::bson::Bson::Null,
                "value_float": mongodb::bson::Bson::Null,
                "value_string": mongodb::bson::Bson::String(self.value_string.to_owned().unwrap_or("".to_owned()))
            }
        } else if self.value_int.is_some() {
            doc! {
                "value_int": self.value_int.unwrap_or(0),
                "value_float": mongodb::bson::Bson::Null,
                "value_string": mongodb::bson::Bson::Null
            }
        } else {
            doc! {
                "value_int": mongodb::bson::Bson::Null,
                "value_float": self.value_float.unwrap_or(0.0),
                "value_string": mongodb::bson::Bson::Null
            }
        }
    }
}

impl Clone for ModelAttribute {
    fn clone(&self) -> ModelAttribute {
        ModelAttribute {
            value_string: self.value_string.to_owned(),
            value_float: self.value_float,
            value_int: self.value_int
        }
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct User {
    pub email: String,
    pub id: String
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Project {
    pub title: String,
    pub id: String,
    pub related_tree_ids: Vec<String>,
    pub selected_model: Option<String>,
    pub related_config_ids: Vec<String>,
    pub selected_config: Option<String>
}

impl Clone for Project {
    fn clone(&self) -> Project {
        Project {
            title: self.title.to_owned(),
            id: self.id.to_owned(),
            related_tree_ids: self.related_tree_ids.to_owned(),
            selected_model: self.selected_model.to_owned(),
            related_config_ids: self.related_config_ids.to_owned(),
            selected_config: self.selected_config.to_owned()
        }
    }
}

impl Project {
    pub fn to_bson_doc(self) -> Document {
        let mut selectedModel = mongodb::bson::Bson::Null;
        let mut selectedConfig = mongodb::bson::Bson::Null;

        if (self.selected_model.is_some()) {
            selectedModel = mongodb::bson::Bson::String(self.selected_model.expect("Asserted"));
        }

        if (self.selected_config.is_some()) {
            selectedConfig = mongodb::bson::Bson::String(self.selected_config.expect("Asserted"));
        }

        let mut object_related_tree_ids = Vec::new();

        for id in self.related_tree_ids {
            object_related_tree_ids.push(id)
        }

        doc! {
            "title": self.title,
            "id": self.id,
            "related_tree_ids": object_related_tree_ids,
            "selectedModel": selectedModel,
            "related_config_ids": self.related_config_ids,
            "selectedConfig": selectedConfig
        }
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct AuthLoginResponseResult {
    pub sessionToken: String,
    pub loginRequest: String
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
    pub id: String,
    pub children: Vec<String>
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ListModelResult {
    pub models: Vec<ListModelResponseItem>
}

#[derive(Serialize, Deserialize, Debug)]

pub struct ListModelResponseItem {
    pub id: String,
    pub title: String
}

#[derive(Serialize, Deserialize, Debug)]

pub struct SelectedModelResult {
    pub modelId: String
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
    pub conditionAttribute: String
}

#[derive(Serialize, Deserialize)]
pub struct ApiFullNodeData {
    pub id: String,
    pub title: String,
    pub description: String,
    pub modelAttributes: HashMap<String, ModelAttribute>,
    pub conditionAttribute: String,
    pub children: Vec<String>,
}

impl Clone for ApiFullNodeData {
    fn clone(&self) -> ApiFullNodeData {
        ApiFullNodeData {
            id: self.id.to_owned(),
            title: self.title.to_owned(),
            description: self.description.to_owned(),
            modelAttributes: self.modelAttributes.clone(),
            conditionAttribute: self.conditionAttribute.to_owned(),
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
            "description": self.description,
            "modelAttributes": model_attributes,
            "conditionAttribute": self.conditionAttribute,
            "children": self.children
        }
    }
}

#[derive(Serialize, Deserialize)]
pub struct ApiFullComputedNodeData {
    pub id: String,
    pub title: String,
    pub description: String,
    pub modelAttributes: HashMap<String, ModelAttribute>,
    pub conditionAttribute: String,
    pub children: Vec<String>,
    pub conditionResolved: bool
}

impl Clone for ApiFullComputedNodeData {
    fn clone(&self) -> ApiFullComputedNodeData {
        ApiFullComputedNodeData {
            id: self.id.to_owned(),
            title: self.title.to_owned(),
            description: self.description.to_owned(),
            modelAttributes: self.modelAttributes.clone(),
            conditionAttribute: self.conditionAttribute.to_owned(),
            conditionResolved: self.conditionResolved.to_owned(),
            children: self.children.clone()
        }
    }
}

impl ApiFullComputedNodeData {
    fn into_bson_doc(self) -> Document {
        let mut model_attributes = doc! {};

        for (key, val) in self.modelAttributes.into_iter() {
            model_attributes.insert(key, val.to_bson_doc());
        }

        doc! {
            "id": self.id,
            "title": self.title,
            "description": self.description,
            "modelAttributes": model_attributes,
            "conditionAttribute": self.conditionAttribute,
            "conditionResolved": self.conditionResolved,
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
pub struct ApiFullComputedTreeData {
    pub title: String,
    pub rootNodeId: String,
    pub nodes: Vec<ApiFullComputedNodeData>
}

impl ApiFullComputedTreeData {
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
pub struct ApiProjectsListProjectItem {
    pub projectId: String,
    pub name: String
}

#[derive(Serialize, Deserialize)]
pub struct ApiProjectsListResponseResult {
    pub projects: Vec<ApiProjectsListProjectItem>
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
pub struct ApiTreeComputedResponse {
    pub ok: bool,
    pub message: String,
    pub result: Option<ApiFullComputedTreeData>
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

#[derive(Serialize, Deserialize, Debug)]
pub struct ApiListModelResponse {
    pub ok: bool,
    pub message: String,
    pub result: Option<ListModelResult>
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ApiSelectedModelResponse {
    pub ok: bool,
    pub message: String,
    pub result: Option<SelectedModelResult>
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ApiGetNodeResponse {
    pub ok: bool,
    pub message: String,
    pub result: Option<ApiGetNodeResponseResult>
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ApiGetNodeResponseResult {
    pub treeId: String
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ApiTreeDagResponse {
    pub ok: bool,
    pub message: String,
    pub result: Option<ApiTreeDagResponseResult>
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ApiTreeDagResponseResult {
    pub root: ApiTreeDagItem
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ApiTreeDagItem {
    pub id: String,
    pub title: String,
    pub children: Vec<ApiTreeDagItem>
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ApiProjectConfigListResponse {
    pub ok: bool,
    pub message: String,
    pub result: Option<ApiProjectConfigListResponseResult> 
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ApiProjectConfigListResponseResult {
    pub ids: Vec<String>
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ApiProjectConfigPayload {
    pub attributes: serde_json::Value
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ApiProjectConfigResponse {
    pub ok: bool,
    pub message: String,
    pub result: Option<ApiProjectConfigResponseResult> 
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ApiProjectConfigResponseResult {
    pub id: String,
    pub attributes: serde_json::Value
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ApiProjectConfigIdPayload {
    pub desiredConfig: String
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ApiOrgMetadataBase {
    pub name: String
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ApiOrgMetadata {
    pub name: String,
    pub id: String
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ApiOrgResponse {
    pub ok: bool,
    pub message: String,
    pub result: Option<ApiOrgMetadata> 
}