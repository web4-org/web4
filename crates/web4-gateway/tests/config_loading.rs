use std::{
    fs,
    path::PathBuf,
    time::{SystemTime, UNIX_EPOCH},
};
use web4_gateway::config::GatewayConfig;

fn unique_temp_dir(prefix: &str) -> PathBuf {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("clock before unix epoch")
        .as_nanos();
    std::env::temp_dir().join(format!("{prefix}-{}-{nanos}", std::process::id()))
}

#[test]
fn parses_with_defaults() {
    let dir = unique_temp_dir("web4-gateway-config-test");
    let _ = fs::remove_dir_all(&dir);
    fs::create_dir_all(&dir).expect("create temp dir");
    let path = dir.join("config.json");
    fs::write(
        &path,
        r#"{
  "server": {"bind_addr": "127.0.0.1:8080"},
  "document": {"root": "examples/gateway", "entry_w4": "default.w4"},
  "runtime": {},
  "security": {"jwt_secret": "secret", "admin_token": "admin"}
}"#,
    )
    .expect("write config");

    let cfg = GatewayConfig::from_file(&path).expect("config parses");
    assert_eq!(cfg.http_base_url(), "http://127.0.0.1:8080");
    assert!(!cfg.debug.enable_error_route);
    assert!(!cfg.rendering.template_loader.allow_remote);
}

#[test]
fn rejects_empty_secret() {
    let dir = unique_temp_dir("web4-gateway-config-test-invalid");
    let _ = fs::remove_dir_all(&dir);
    fs::create_dir_all(&dir).expect("create temp dir");
    let path = dir.join("config.json");
    fs::write(
        &path,
        r#"{
  "server": {"bind_addr": "127.0.0.1:8080"},
  "document": {"root": "examples/gateway", "entry_w4": "default.w4"},
  "runtime": {},
  "security": {"jwt_secret": "", "admin_token": "admin"}
}"#,
    )
    .expect("write config");

    let err = GatewayConfig::from_file(&path).expect_err("config must fail");
    assert_eq!(err.0.code, "INVALID_ARGUMENT");
}
