use crate::config::GatewayConfig;
use crate::error::GatewayError;
use crate::handlers::{
    approve_challenge, create_challenge, deny_challenge, get_document, healthz,
    invoke_service_by_source_ref, issue_capability, sample_error, view_document_by_path,
};
use crate::state::{AppState, InMemoryChallengeStore, InMemoryRateLimiter};
use axum::{
    routing::{get, post},
    Router,
};
use std::{
    net::SocketAddr,
    path::{Component, Path, PathBuf},
    sync::{atomic::AtomicU64, Arc},
};
use web4_core::{
    DefaultConsentEngine, GatewayBindingExecutor, HttpBindingExecutor, JsonSchemaValidator,
    LocalBindingExecutor, ServiceRuntime, W4mlParser,
};
use web4_core::{ErrorCode, RuntimeError};

pub async fn run(config: GatewayConfig) -> Result<(), GatewayError> {
    let state = build_state(&config)?;
    let app = build_app(state, config.debug.enable_error_route);

    let addr: SocketAddr = config.server.bind_addr.parse().map_err(|err| {
        GatewayError(RuntimeError::new(
            ErrorCode::InvalidArgument,
            format!("invalid server.bind_addr: {err}"),
            false,
        ))
    })?;
    let listener = tokio::net::TcpListener::bind(addr).await.map_err(|err| {
        GatewayError(RuntimeError::new(
            ErrorCode::InternalError,
            format!("bind gateway listener failed: {err}"),
            false,
        ))
    })?;

    tracing::info!(%addr, "web4 gateway listening");
    axum::serve(listener, app).await.map_err(|err| {
        GatewayError(RuntimeError::new(
            ErrorCode::InternalError,
            format!("serve gateway failed: {err}"),
            false,
        ))
    })?;
    Ok(())
}

pub fn build_app(state: AppState, enable_debug_routes: bool) -> Router {
    let mut router = Router::new()
        .route("/healthz", get(healthz))
        .route("/", get(get_document))
        .route(
            "/{*source_ref}",
            get(view_document_by_path).post(invoke_service_by_source_ref),
        )
        .route("/consent/issue", post(issue_capability))
        .route("/consent/challenge", post(create_challenge))
        .route(
            "/consent/challenge/{challenge_id}/approve",
            post(approve_challenge),
        )
        .route(
            "/consent/challenge/{challenge_id}/deny",
            post(deny_challenge),
        )
        .with_state(state);
    if enable_debug_routes {
        router = router.route("/errors/{code}", post(sample_error));
    }
    router
}

pub fn build_state(config: &GatewayConfig) -> Result<AppState, GatewayError> {
    let root = canonicalize_root(&config.document.root)?;
    let source_path = resolve_path_under_root(&root, &config.document.entry_w4).map_err(invalid)?;
    let source = std::fs::read_to_string(&source_path).map_err(|err| {
        GatewayError(RuntimeError::new(
            ErrorCode::InvalidArgument,
            format!("read document entry source failed: {err}"),
            false,
        ))
    })?;
    let parser = W4mlParser;
    let parsed = parser
        .parse(&source)
        .map_err(|e| GatewayError(e.into_runtime()))?;
    let mut model = parsed.model.ok_or_else(|| {
        GatewayError(RuntimeError::new(
            ErrorCode::InvalidArgument,
            "W4 source did not produce normalized model",
            false,
        ))
    })?;
    resolve_w4s_bindings(&mut model, &root).map_err(invalid)?;

    let runtime = ServiceRuntime {
        validator: JsonSchemaValidator,
        consent_engine: DefaultConsentEngine,
        binding_executor: GatewayBindingExecutor {
            http: HttpBindingExecutor {
                base_url: Some(config.http_base_url()),
            },
            local: LocalBindingExecutor {
                working_dir: Some(root.display().to_string()),
                mcp: None,
            },
        },
    };

    Ok(AppState {
        source: Arc::new(source),
        model: Arc::new(model),
        document_entry: Arc::new(config.document.entry_w4.clone()),
        runtime: Arc::new(runtime),
        jwt_secret: Arc::new(config.security.jwt_secret.clone()),
        challenge_counter: Arc::new(AtomicU64::new(1)),
        challenges: Arc::new(InMemoryChallengeStore::default()),
        rate_limiter: Arc::new(InMemoryRateLimiter::default()),
        admin_token: Arc::new(config.security.admin_token.clone()),
        rendering: Arc::new(config.rendering.clone()),
        document_root: Arc::new(root.clone()),
        document_dir: Arc::new(source_path.parent().unwrap_or(&root).to_path_buf()),
    })
}

fn resolve_w4s_bindings(model: &mut serde_json::Value, root: &Path) -> Result<(), String> {
    let services = model
        .get_mut("services")
        .and_then(serde_json::Value::as_array_mut)
        .ok_or_else(|| "model.services must be an array".to_string())?;

    for service in services.iter_mut() {
        let service_id = service
            .get("id")
            .and_then(serde_json::Value::as_str)
            .ok_or_else(|| "service.id missing".to_string())?;
        let binding_ref = service
            .get("source_ref")
            .and_then(serde_json::Value::as_str)
            .ok_or_else(|| format!("service {service_id} missing source_ref"))?;
        let w4s_path = resolve_path_under_root(root, binding_ref)?;
        let w4s_source = std::fs::read_to_string(&w4s_path)
            .map_err(|err| format!("read binding file {binding_ref} failed: {err}"))?;
        let bindings = parse_w4s_bindings(&w4s_source, binding_ref, service_id)?;
        if bindings.is_empty() {
            return Err(format!("binding file {binding_ref} has no binding element"));
        }
        service["bindings"] = serde_json::Value::Array(bindings);
    }

    Ok(())
}

fn parse_w4s_bindings(
    source: &str,
    binding_ref: &str,
    expected_service: &str,
) -> Result<Vec<serde_json::Value>, String> {
    const W4_NS: &str = "urn:w4ml:0.1";

    let doc = roxmltree::Document::parse(source)
        .map_err(|err| format!("parse binding file {binding_ref} failed: {err}"))?;
    let root = doc.root_element();
    if root.tag_name().namespace() != Some(W4_NS) || root.tag_name().name() != "w4s" {
        return Err(format!(
            "binding file {binding_ref} must have root <w4s xmlns=\"{W4_NS}\">"
        ));
    }
    if let Some(service_id) = root.attribute("service") {
        if service_id != expected_service {
            return Err(format!(
                "binding file {binding_ref} declares service {service_id}, expected {expected_service}"
            ));
        }
    }

    let bindings_node = root
        .children()
        .find(|n| {
            n.is_element()
                && n.tag_name().namespace() == Some(W4_NS)
                && n.tag_name().name() == "bindings"
        })
        .ok_or_else(|| format!("binding file {binding_ref} missing <bindings>"))?;

    let bindings = bindings_node
        .children()
        .filter(|n| {
            n.is_element()
                && n.tag_name().namespace() == Some(W4_NS)
                && n.tag_name().name() == "binding"
        })
        .map(|binding| {
            let mut map = serde_json::Map::new();
            for attr in binding.attributes() {
                map.insert(
                    attr.name().to_string(),
                    serde_json::Value::String(attr.value().to_string()),
                );
            }
            serde_json::Value::Object(map)
        })
        .collect::<Vec<_>>();

    Ok(bindings)
}

fn canonicalize_root(root: &str) -> Result<PathBuf, GatewayError> {
    std::fs::canonicalize(root).map_err(|err| {
        GatewayError(RuntimeError::new(
            ErrorCode::InvalidArgument,
            format!("invalid document.root: {err}"),
            false,
        ))
    })
}

fn invalid(message: String) -> GatewayError {
    GatewayError(RuntimeError::new(
        ErrorCode::InvalidArgument,
        message,
        false,
    ))
}

fn resolve_path_under_root(root: &Path, relative_path: &str) -> Result<PathBuf, String> {
    if relative_path.is_empty() {
        return Err("path must not be empty".to_string());
    }
    let rel = Path::new(relative_path);
    if rel.is_absolute() {
        return Err(format!(
            "path must be relative to document.root: {relative_path}"
        ));
    }
    for component in rel.components() {
        match component {
            Component::Normal(_) => {}
            Component::CurDir => {}
            _ => return Err(format!("path contains disallowed segment: {relative_path}")),
        }
    }

    let joined = root.join(rel);
    let resolved = std::fs::canonicalize(&joined)
        .map_err(|err| format!("resolve path {relative_path} failed: {err}"))?;
    if !resolved.starts_with(root) {
        return Err(format!("path escapes document.root: {relative_path}"));
    }
    Ok(resolved)
}
