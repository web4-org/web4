use serde::{Deserialize, Serialize};
use serde_json::{json, Value};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolCompileInput {
    pub service_id: String,
    pub description: String,
    pub parameters: Value,
    pub canonical_uri: String,
    pub effects: String,
    pub consent_mode: String,
    pub bindings: Vec<Value>,
    pub load: String,
}

pub fn compile_to_w4_json(input: &ToolCompileInput) -> Value {
    json!({
        "type": "function",
        "function": {
            "name": input.service_id,
            "description": input.description,
            "parameters": input.parameters,
        },
        "x-w4": {
            "id": format!("{}#service={}", input.canonical_uri, input.service_id),
            "effects": input.effects,
            "consent": { "mode": input.consent_mode },
            "bindings": input.bindings,
            "load": input.load,
        }
    })
}
