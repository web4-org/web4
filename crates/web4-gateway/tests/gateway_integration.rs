use reqwest::header::{ACCEPT, CONTENT_TYPE};
use serde_json::json;
use web4_gateway::{
    app::{build_app, build_state},
    config::{
        DebugConfig, DocumentConfig, GatewayConfig, RenderingConfig, RuntimeConfig, SecurityConfig,
        ServerConfig,
    },
};

async fn start_gateway() -> (String, tokio::task::JoinHandle<()>) {
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0")
        .await
        .expect("bind test listener");
    let addr = listener.local_addr().expect("local addr");

    let state = build_state(&GatewayConfig {
        server: ServerConfig {
            bind_addr: addr.to_string(),
        },
        document: DocumentConfig {
            root: std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
                .join("../../examples/gateway")
                .display()
                .to_string(),
            entry_w4: "default.w4".to_string(),
        },
        runtime: RuntimeConfig {
            http_base_url: Some(format!("http://{addr}")),
        },
        security: SecurityConfig {
            jwt_secret: "test-secret".to_string(),
            admin_token: "test-admin-token".to_string(),
        },
        debug: DebugConfig::default(),
        rendering: RenderingConfig::default(),
    })
    .expect("build state");
    let app = build_app(state, false);
    let handle = tokio::spawn(async move {
        axum::serve(listener, app).await.expect("serve app");
    });

    (format!("http://{addr}"), handle)
}

#[tokio::test]
async fn returns_w4_json_view() {
    let (base, handle) = start_gateway().await;
    let client = reqwest::Client::new();

    let response = client
        .get(format!("{base}/"))
        .header(ACCEPT, "application/w4+json")
        .send()
        .await
        .expect("request success");

    assert!(response.status().is_success());
    assert_eq!(
        response
            .headers()
            .get(CONTENT_TYPE)
            .and_then(|v| v.to_str().ok()),
        Some("application/w4+json")
    );

    let body: serde_json::Value = response.json().await.expect("valid json");
    assert!(body["imports"].as_array().expect("imports array").len() > 0);
    assert!(body["peers"].as_array().expect("peers array").len() > 0);
    assert_eq!(
        body["tools"][0]["x-w4"]["bindings"][0]["endpoint"].as_str(),
        Some("/services/math.add.w4s")
    );

    handle.abort();
}

#[tokio::test]
async fn interactive_service_still_requires_challenge() {
    let (base, handle) = start_gateway().await;
    let client = reqwest::Client::new();

    let response = client
        .post(format!("{base}/services/math.interactive_add.w4s"))
        .header(CONTENT_TYPE, "application/json")
        .header("x-web4-agent-id", "trusted-agent")
        .json(&json!({
            "a": 1,
            "b": 2,
            "consent": {"mode": "open"},
            "effects": "none",
            "allowAgents": "*"
        }))
        .send()
        .await
        .expect("request success");

    assert_eq!(response.status(), reqwest::StatusCode::FORBIDDEN);
    let body: serde_json::Value = response.json().await.expect("valid error json");
    assert_eq!(body["error"]["code"], "CONSENT_REQUIRED");

    handle.abort();
}

#[tokio::test]
async fn capability_issue_rejects_open_service() {
    let (base, handle) = start_gateway().await;
    let client = reqwest::Client::new();

    let response = client
        .post(format!("{base}/consent/issue"))
        .header(CONTENT_TYPE, "application/json")
        .header("Authorization", "Bearer test-admin-token")
        .json(&json!({
            "service_id": "math.add",
            "sub": "tester"
        }))
        .send()
        .await
        .expect("request success");

    assert_eq!(response.status(), reqwest::StatusCode::BAD_REQUEST);
    let body: serde_json::Value = response.json().await.expect("valid error json");
    assert_eq!(body["error"]["code"], "INVALID_ARGUMENT");

    handle.abort();
}

#[tokio::test]
async fn capability_issue_requires_admin_token() {
    let (base, handle) = start_gateway().await;
    let client = reqwest::Client::new();

    let response = client
        .post(format!("{base}/consent/issue"))
        .header(CONTENT_TYPE, "application/json")
        .json(&json!({
            "service_id": "math.cap_add",
            "sub": "tester"
        }))
        .send()
        .await
        .expect("request success");

    assert_eq!(response.status(), reqwest::StatusCode::UNAUTHORIZED);
    let body: serde_json::Value = response.json().await.expect("valid error json");
    assert_eq!(body["error"]["code"], "UNAUTHORIZED");

    handle.abort();
}

#[tokio::test]
async fn debug_error_route_disabled_by_default() {
    let (base, handle) = start_gateway().await;
    let client = reqwest::Client::new();

    let response = client
        .get(format!("{base}/errors/INTERNAL_ERROR"))
        .send()
        .await
        .expect("request success");

    assert_eq!(response.status(), reqwest::StatusCode::METHOD_NOT_ALLOWED);
    handle.abort();
}
