use serde_json::{json, Value};

pub fn w4_type_to_json_schema(typ: &Value) -> Value {
    let kind = typ.get("kind").and_then(Value::as_str).unwrap_or("object");
    match kind {
        "string" => json!({"type": "string"}),
        "int" => json!({"type": "integer"}),
        "float" => json!({"type": "number"}),
        "bool" => json!({"type": "boolean"}),
        "datetime" => json!({"type": "string", "format": "date-time"}),
        "uri" => json!({"type": "string", "format": "uri"}),
        "bytes" => json!({"type": "string", "contentEncoding": "base64"}),
        "array" => json!({"type": "array"}),
        _ => {
            let mut properties = serde_json::Map::new();
            let mut required = Vec::new();

            if let Some(props) = typ.get("properties").and_then(Value::as_array) {
                for prop in props {
                    let Some(name) = prop.get("name").and_then(Value::as_str) else {
                        continue;
                    };

                    let prop_kind = prop.get("type").and_then(Value::as_str).unwrap_or("string");
                    let mut schema = match prop_kind {
                        "string" => json!({"type": "string"}),
                        "int" => json!({"type": "integer"}),
                        "float" => json!({"type": "number"}),
                        "bool" => json!({"type": "boolean"}),
                        "datetime" => json!({"type": "string", "format": "date-time"}),
                        "uri" => json!({"type": "string", "format": "uri"}),
                        "bytes" => json!({"type": "string", "contentEncoding": "base64"}),
                        other => json!({"$ref": format!("#/types/{other}")}),
                    };

                    if let Some(desc) = prop.get("desc").and_then(Value::as_str) {
                        schema["description"] = Value::String(desc.to_string());
                    }
                    properties.insert(name.to_string(), schema);

                    if prop.get("required").and_then(Value::as_str) == Some("true") {
                        required.push(Value::String(name.to_string()));
                    }
                }
            }

            json!({
                "type": "object",
                "properties": properties,
                "required": required,
                "additionalProperties": typ.get("additionalProperties").and_then(Value::as_bool).unwrap_or(true),
            })
        }
    }
}
