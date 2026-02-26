use async_trait::async_trait;
use axum::{routing::post, Json, Router};
use serde_json::{json, Value};
use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::sync::Arc;
use web4_core::{
    DefaultConsentEngine, GatewayBindingExecutor, HttpBindingExecutor, InvocationContext,
    JsonSchemaValidator, LocalBindingExecutor, McpInvoker, ServiceRuntime,
};

fn sample_ir_with(binding: Value, effects: &str, mode: &str) -> Value {
    json!({
        "services": [{
            "id": "math.add",
            "effects": effects,
            "consent": {"mode": mode},
            "bindings": [binding],
            "input": {
                "id": "AddInput",
                "kind": "object",
                "properties": [
                    {"name": "a", "type": "int", "required": "true"},
                    {"name": "b", "type": "int", "required": "true"}
                ],
                "additionalProperties": false
            },
            "output": {
                "id": "AddOutput",
                "kind": "object",
                "properties": [
                    {"name": "sum", "type": "int", "required": "true"}
                ],
                "additionalProperties": false
            }
        }]
    })
}

#[tokio::test]
async fn effects_blocked_without_consent() {
    let runtime = ServiceRuntime {
        validator: JsonSchemaValidator,
        consent_engine: DefaultConsentEngine,
        binding_executor: GatewayBindingExecutor::default(),
    };
    let ir = sample_ir_with(
        json!({"type": "http", "method": "POST", "endpoint": "http://localhost/any", "contentType":"application/json"}),
        "write",
        "open",
    );
    let err = runtime
        .invoke(
            &ir,
            "math.add",
            json!({"a": 1, "b": 2}),
            InvocationContext::default(),
        )
        .await
        .expect_err("should block high effects open mode");

    assert_eq!(err.into_runtime().code, "EFFECTS_BLOCKED");
}

#[tokio::test]
async fn capability_requires_token() {
    let runtime = ServiceRuntime {
        validator: JsonSchemaValidator,
        consent_engine: DefaultConsentEngine,
        binding_executor: GatewayBindingExecutor::default(),
    };
    let ir = sample_ir_with(
        json!({"type": "http", "method": "POST", "endpoint": "http://localhost/any", "contentType":"application/json"}),
        "read",
        "capability",
    );
    let err = runtime
        .invoke(
            &ir,
            "math.add",
            json!({"a": 1, "b": 2}),
            InvocationContext::default(),
        )
        .await
        .expect_err("should reject missing token");

    assert_eq!(err.into_runtime().code, "UNAUTHORIZED");
}

#[tokio::test]
async fn http_binding_happy_path() {
    async fn handler(Json(payload): Json<Value>) -> Json<Value> {
        let a = payload.get("a").and_then(Value::as_i64).unwrap_or(0);
        let b = payload.get("b").and_then(Value::as_i64).unwrap_or(0);
        Json(json!({"sum": a + b}))
    }

    let app = Router::new().route("/rpc/math.add", post(handler));
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0")
        .await
        .expect("bind");
    let addr = listener.local_addr().expect("addr");
    tokio::spawn(async move {
        axum::serve(listener, app).await.expect("serve test app");
    });

    let runtime = ServiceRuntime {
        validator: JsonSchemaValidator,
        consent_engine: DefaultConsentEngine,
        binding_executor: GatewayBindingExecutor {
            http: HttpBindingExecutor {
                base_url: Some(format!("http://{addr}")),
            },
            local: LocalBindingExecutor::default(),
        },
    };

    let ir = sample_ir_with(
        json!({"type": "http", "method": "POST", "endpoint": "/rpc/math.add", "contentType":"application/json"}),
        "none",
        "open",
    );

    let result = runtime
        .invoke(
            &ir,
            "math.add",
            json!({"a": 1, "b": 2}),
            InvocationContext::default(),
        )
        .await
        .expect("invoke should succeed");

    assert_eq!(result["sum"], 3);
}

#[tokio::test]
async fn local_bin_binding_works() {
    let script_path = std::env::temp_dir().join("web4-bin-echo.sh");
    fs::write(
        &script_path,
        r#"#!/usr/bin/env bash
python3 -c "import json,sys;d=json.load(sys.stdin);print(json.dumps({'sum': d['a'] + d['b']}))"
"#,
    )
    .expect("write script");
    let mut perms = fs::metadata(&script_path).expect("meta").permissions();
    perms.set_mode(0o755);
    fs::set_permissions(&script_path, perms).expect("chmod");

    let runtime = ServiceRuntime {
        validator: JsonSchemaValidator,
        consent_engine: DefaultConsentEngine,
        binding_executor: GatewayBindingExecutor {
            http: HttpBindingExecutor::default(),
            local: LocalBindingExecutor::default(),
        },
    };

    let ir = sample_ir_with(
        json!({"type": "local", "exec": format!("bin:{}", script_path.display())}),
        "none",
        "open",
    );

    let result = runtime
        .invoke(
            &ir,
            "math.add",
            json!({"a": 1, "b": 2}),
            InvocationContext::default(),
        )
        .await
        .expect("invoke should succeed");

    assert_eq!(result["sum"], 3);
}

#[tokio::test]
async fn local_bin_binding_with_working_dir_works() {
    let mod_dir = std::env::temp_dir().join("web4_cmd_mod");
    fs::create_dir_all(&mod_dir).expect("mkdir");
    fs::write(
        mod_dir.join("adder.py"),
        "import json, sys\npayload = json.load(sys.stdin)\nprint(json.dumps({'sum': payload['a'] + payload['b']}))\n",
    )
    .expect("write module");

    let runtime = ServiceRuntime {
        validator: JsonSchemaValidator,
        consent_engine: DefaultConsentEngine,
        binding_executor: GatewayBindingExecutor {
            http: HttpBindingExecutor::default(),
            local: LocalBindingExecutor {
                working_dir: Some(mod_dir.display().to_string()),
                mcp: None,
            },
        },
    };

    let ir = sample_ir_with(
        json!({"type": "local", "exec": "bin:python3 adder.py"}),
        "none",
        "open",
    );

    let result = runtime
        .invoke(
            &ir,
            "math.add",
            json!({"a": 1, "b": 2}),
            InvocationContext::default(),
        )
        .await
        .expect("invoke should succeed");

    assert_eq!(result["sum"], 3);
}

struct MockMcp;

#[async_trait]
impl McpInvoker for MockMcp {
    async fn call_tool(
        &self,
        _server: &str,
        _tool: &str,
        args: Value,
    ) -> Result<Value, web4_core::Web4Error> {
        Ok(json!({"sum": args["a"].as_i64().unwrap_or(0) + args["b"].as_i64().unwrap_or(0)}))
    }
}

#[tokio::test]
async fn local_mcp_binding_works() {
    let runtime = ServiceRuntime {
        validator: JsonSchemaValidator,
        consent_engine: DefaultConsentEngine,
        binding_executor: GatewayBindingExecutor {
            http: HttpBindingExecutor::default(),
            local: LocalBindingExecutor {
                working_dir: None,
                mcp: Some(Arc::new(MockMcp)),
            },
        },
    };

    let ir = sample_ir_with(
        json!({"type": "local", "exec": "mcp:calc/add"}),
        "none",
        "open",
    );

    let result = runtime
        .invoke(
            &ir,
            "math.add",
            json!({"a": 2, "b": 3}),
            InvocationContext::default(),
        )
        .await
        .expect("invoke should succeed");

    assert_eq!(result["sum"], 5);
}
