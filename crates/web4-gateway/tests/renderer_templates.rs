use serde_json::json;
use std::{
    fs,
    path::PathBuf,
    time::{SystemTime, UNIX_EPOCH},
};
use web4_gateway::{
    config::RenderingConfig,
    renderer::{render_html, RenderOptions},
};

fn unique_temp_dir(prefix: &str) -> PathBuf {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("clock before unix epoch")
        .as_nanos();
    std::env::temp_dir().join(format!("{prefix}-{}-{nanos}", std::process::id()))
}

fn sample_ir(template_href: &str) -> serde_json::Value {
    json!({
        "doc": {
            "id": "demo",
            "titles": [{"lang": "en", "text": "Demo"}],
            "meta": [{"name": "description", "content": "Desc"}],
            "links": [{"rel": "template", "href": template_href, "type": "text/html"}]
        },
        "sections": [
            {"id": "overview", "name": "Overview", "load": "eager", "class": "hero", "content": "Welcome"}
        ],
        "services": [
            {"id": "math.add", "name": "Add", "load": "lazy", "class": "tool", "effects": "none", "consent": {"mode": "open"}, "intents": [{"text": "Add numbers"}]}
        ],
        "body_order": [
            {"kind": "section", "id": "overview"},
            {"kind": "service", "id": "math.add"}
        ]
    })
}

#[tokio::test]
async fn renders_relative_custom_template() {
    let root = unique_temp_dir("web4-render-template");
    let _ = fs::remove_dir_all(&root);
    fs::create_dir_all(&root).expect("create temp root");
    let doc_dir = root.join("docs");
    fs::create_dir_all(&doc_dir).expect("create doc dir");
    fs::write(
        doc_dir.join("page.html"),
        "<!doctype html><html><body>{{ page.title }}|{{ stats.visible_count }}</body></html>",
    )
    .expect("write template");

    let rendering = RenderingConfig::default();
    let options = RenderOptions {
        fragment_selector: None,
        rendering: &rendering,
        document_root: &root,
        document_dir: &doc_dir,
    };
    let html = render_html(&sample_ir("page.html"), &options).await;
    assert!(html.contains("Demo|2"));
}

#[tokio::test]
async fn falls_back_when_remote_template_is_disabled() {
    let root = unique_temp_dir("web4-render-fallback");
    let _ = fs::remove_dir_all(&root);
    fs::create_dir_all(&root).expect("create temp root");

    let rendering = RenderingConfig::default();
    let options = RenderOptions {
        fragment_selector: None,
        rendering: &rendering,
        document_root: &root,
        document_dir: &root,
    };
    let html = render_html(&sample_ir("https://example.com/custom.html"), &options).await;
    assert!(html.contains("template builtin-default-fallback"));
}
