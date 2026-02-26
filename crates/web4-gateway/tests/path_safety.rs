use std::fs;
use std::path::PathBuf;

use web4_gateway::{
    app::build_state,
    config::{
        DebugConfig, DocumentConfig, GatewayConfig, RenderingConfig, RuntimeConfig, SecurityConfig,
        ServerConfig,
    },
};

fn temp_dir(name: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!(
        "web4-gateway-path-test-{name}-{}",
        std::process::id()
    ));
    let _ = fs::remove_dir_all(&dir);
    fs::create_dir_all(&dir).expect("create temp dir");
    dir
}

#[test]
fn rejects_service_binding_path_escape() {
    let root = temp_dir("escape");
    fs::create_dir_all(root.join("services")).expect("create services dir");

    let entry = root.join("entry.w4");
    fs::write(
        &entry,
        r#"<?xml version="1.0" encoding="UTF-8"?>
<w4 xmlns="urn:w4ml:0.1" version="0.1">
  <head>
    <title lang="en">Path safety</title>
  </head>
  <body>
    <schema>
      <type id="AddInput" kind="object">
        <property name="a" type="int" required="true"/>
      </type>
      <type id="AddOutput" kind="object">
        <property name="sum" type="int" required="true"/>
      </type>
    </schema>
    <service id="math.add" name="add" sourceRef="../outside.w4s">
      <intent lang="en">Add</intent>
      <input typeRef="AddInput"/>
      <output typeRef="AddOutput"/>
      <effects level="none"/>
      <consent mode="open"/>
    </service>
  </body>
</w4>"#,
    )
    .expect("write entry");
    fs::write(root.parent().expect("parent").join("outside.w4s"), "<w4s/>").expect("write outside");

    let result = build_state(&GatewayConfig {
        server: ServerConfig {
            bind_addr: "127.0.0.1:0".to_string(),
        },
        document: DocumentConfig {
            root: root.display().to_string(),
            entry_w4: "entry.w4".to_string(),
        },
        runtime: RuntimeConfig {
            http_base_url: Some("http://127.0.0.1:0".to_string()),
        },
        security: SecurityConfig {
            jwt_secret: "test-secret".to_string(),
            admin_token: "admin".to_string(),
        },
        debug: DebugConfig::default(),
        rendering: RenderingConfig::default(),
    });

    let err = match result {
        Ok(_) => panic!("path traversal must be rejected"),
        Err(err) => err,
    };
    assert_eq!(err.0.code, "INVALID_ARGUMENT");
    assert!(err.0.message.contains("disallowed segment"));
}
