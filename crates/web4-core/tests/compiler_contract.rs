use serde_json::json;
use web4_core::{compile_to_w4_json, ToolCompileInput};

#[test]
fn compiler_emits_x_w4_extension() {
    let value = compile_to_w4_json(&ToolCompileInput {
        service_id: "ping".into(),
        description: "Ping tool".into(),
        parameters: json!({"type": "object"}),
        canonical_uri: "https://example.com/w4.xml".into(),
        effects: "read".into(),
        consent_mode: "open".into(),
        bindings: vec![json!({"type": "http", "method": "POST"})],
        load: "eager".into(),
    });

    assert_eq!(
        value["x-w4"]["id"],
        "https://example.com/w4.xml#service=ping"
    );
}
