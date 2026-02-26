use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ParseResult {
    pub raw: String,
    pub model: Option<Value>,
    pub warnings: Vec<String>,
    pub index: Option<ModelIndex>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum ValidationTarget {
    JsonSchema { schema: Value, payload: Value },
    NormalizedModel { model: Value },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConsentRequest {
    pub service_id: String,
    pub effects: String,
    pub mode: String,
    pub capability_token: Option<String>,
    pub interactive_approved: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BindingInvocation {
    pub service_id: String,
    pub binding: Value,
    pub input: Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RenderRequest {
    pub document: Value,
    pub accept_language: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ModelIndex {
    pub section_ids: HashMap<String, usize>,
    pub service_ids: HashMap<String, usize>,
    pub type_ids: HashMap<String, usize>,
}
