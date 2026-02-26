use web4_core::{build_default_runtime, RuntimeOptions};

#[test]
fn builds_runtime_from_options() {
    let runtime = build_default_runtime(RuntimeOptions {
        http_base_url: Some("http://127.0.0.1:8080".to_string()),
        local_working_dir: Some("/tmp".to_string()),
    });
    assert_eq!(
        runtime.binding_executor.http.base_url.as_deref(),
        Some("http://127.0.0.1:8080")
    );
}
