use crate::error::GatewayError;
use crate::logic::{
    compile_view_json, consume_interactive_challenge, create_challenge_record, document_issuer,
    enforce_agent_policy, enforce_rate_limit, find_service, find_service_by_source_ref,
    issue_capability_token, negotiate_view, resolve_allow_origin, select_fragment,
    service_consent_mode, strip_reserved_policy_fields, update_challenge_status,
    validate_capability_token, View,
};
use crate::renderer::{render_html, RenderOptions};
use crate::state::{AppState, ChallengeStatus};
use axum::{
    extract::{Path, Query, State},
    http::{header, HeaderMap, HeaderValue, StatusCode},
    response::{IntoResponse, Response},
    Json,
};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use web4_core::{ErrorCode, RuntimeError};

#[derive(Debug, Deserialize)]
pub struct DocumentQuery {
    pub w4fragment: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct IssueRequest {
    pub service_id: String,
    pub sub: Option<String>,
    pub ttl_seconds: Option<u64>,
}

#[derive(Debug, Serialize)]
pub struct IssueResponse {
    pub token: String,
    pub expires_at: u64,
}

#[derive(Debug, Deserialize)]
pub struct ChallengeRequest {
    pub service_id: String,
    pub ttl_seconds: Option<u64>,
}

#[derive(Debug, Serialize)]
pub struct ChallengeResponse {
    pub challenge_id: String,
    pub expires_at: u64,
}

#[derive(Debug, Serialize)]
pub struct ChallengeUpdateResponse {
    pub challenge_id: String,
    pub status: &'static str,
}

pub async fn healthz() -> Json<Value> {
    Json(json!({"status": "ok"}))
}

pub async fn get_document(
    State(state): State<AppState>,
    Query(query): Query<DocumentQuery>,
    headers: HeaderMap,
) -> Result<Response, GatewayError> {
    render_document_response(state, query, headers).await
}

pub async fn view_document_by_path(
    State(state): State<AppState>,
    Path(source_ref): Path<String>,
    Query(query): Query<DocumentQuery>,
    headers: HeaderMap,
) -> Result<Response, GatewayError> {
    let normalized = source_ref.trim_start_matches('/');
    if normalized.is_empty() || normalized != state.document_entry.as_str() {
        return Err(GatewayError(RuntimeError::new(
            ErrorCode::NotFound,
            "document path not found",
            false,
        )));
    }

    render_document_response(state, query, headers).await
}

async fn render_document_response(
    state: AppState,
    query: DocumentQuery,
    headers: HeaderMap,
) -> Result<Response, GatewayError> {
    let view = negotiate_view(headers.get(header::ACCEPT).and_then(|h| h.to_str().ok()))?;

    let fragment_model = match query.w4fragment {
        Some(ref selector) => select_fragment(state.model.as_ref(), selector)?,
        None => state.model.as_ref().clone(),
    };

    let (content_type, body) = match view {
        View::W4ml => {
            let xml = if query.w4fragment.is_some() {
                crate::logic::render_w4ml_fragment(&fragment_model)
            } else {
                state.source.as_ref().clone()
            };
            ("application/w4ml+xml", xml)
        }
        View::Html => {
            let render_options = RenderOptions {
                fragment_selector: query.w4fragment.as_deref(),
                rendering: state.rendering.as_ref(),
                document_root: state.document_root.as_ref(),
                document_dir: state.document_dir.as_ref(),
            };
            (
                "text/html; charset=utf-8",
                render_html(&fragment_model, &render_options).await,
            )
        }
        View::W4Json => (
            "application/w4+json",
            serde_json::to_string_pretty(&compile_view_json(&fragment_model)).map_err(|err| {
                GatewayError(RuntimeError::new(
                    ErrorCode::InternalError,
                    format!("serialize w4+json failed: {err}"),
                    false,
                ))
            })?,
        ),
    };

    let mut response = Response::new(body.into());
    *response.status_mut() = StatusCode::OK;
    response.headers_mut().insert(
        header::CONTENT_TYPE,
        HeaderValue::from_str(content_type).map_err(|err| {
            GatewayError(RuntimeError::new(
                ErrorCode::InternalError,
                format!("invalid content-type header: {err}"),
                false,
            ))
        })?,
    );
    response
        .headers_mut()
        .insert(header::VARY, HeaderValue::from_static("Accept"));

    Ok(response)
}

pub async fn issue_capability(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(req): Json<IssueRequest>,
) -> Result<Json<IssueResponse>, GatewayError> {
    require_admin_token(&headers, state.admin_token.as_str())?;
    let service = find_service(state.model.as_ref(), &req.service_id)?;
    if service_consent_mode(service) != "capability" {
        return Err(GatewayError(RuntimeError::new(
            ErrorCode::InvalidArgument,
            "service is not in capability mode",
            false,
        )));
    }

    let (token, expires_at) = issue_capability_token(
        state.model.as_ref(),
        state.jwt_secret.as_bytes(),
        req.service_id,
        req.sub,
        req.ttl_seconds,
    )?;

    Ok(Json(IssueResponse { token, expires_at }))
}

pub async fn create_challenge(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(req): Json<ChallengeRequest>,
) -> Result<Json<ChallengeResponse>, GatewayError> {
    require_admin_token(&headers, state.admin_token.as_str())?;
    let service = find_service(state.model.as_ref(), &req.service_id)?;
    if service_consent_mode(service) != "interactive" {
        return Err(GatewayError(RuntimeError::new(
            ErrorCode::InvalidArgument,
            "service is not in interactive mode",
            false,
        )));
    }

    let (challenge_id, expires_at) =
        create_challenge_record(&state, req.service_id, req.ttl_seconds)?;

    Ok(Json(ChallengeResponse {
        challenge_id,
        expires_at,
    }))
}

pub async fn approve_challenge(
    State(state): State<AppState>,
    Path(challenge_id): Path<String>,
    headers: HeaderMap,
) -> Result<Json<ChallengeUpdateResponse>, GatewayError> {
    require_admin_token(&headers, state.admin_token.as_str())?;
    update_challenge_status(&state, &challenge_id, ChallengeStatus::Approved)?;
    Ok(Json(ChallengeUpdateResponse {
        challenge_id,
        status: "approved",
    }))
}

pub async fn deny_challenge(
    State(state): State<AppState>,
    Path(challenge_id): Path<String>,
    headers: HeaderMap,
) -> Result<Json<ChallengeUpdateResponse>, GatewayError> {
    require_admin_token(&headers, state.admin_token.as_str())?;
    update_challenge_status(&state, &challenge_id, ChallengeStatus::Denied)?;
    Ok(Json(ChallengeUpdateResponse {
        challenge_id,
        status: "denied",
    }))
}

pub async fn invoke_service_by_source_ref(
    State(state): State<AppState>,
    Path(source_ref): Path<String>,
    headers: HeaderMap,
    Json(input): Json<Value>,
) -> Result<Response, GatewayError> {
    if !source_ref.ends_with(".w4s") {
        return Err(GatewayError(RuntimeError::new(
            ErrorCode::NotFound,
            "service endpoint not found",
            false,
        )));
    }

    let normalized_source_ref = source_ref.trim_start_matches('/');
    let service = find_service_by_source_ref(state.model.as_ref(), normalized_source_ref)?;
    let service_id = service
        .get("id")
        .and_then(Value::as_str)
        .ok_or_else(|| {
            GatewayError(RuntimeError::new(
                ErrorCode::InternalError,
                "service id missing",
                false,
            ))
        })?
        .to_string();

    invoke_service_inner(state, service_id, headers, input).await
}

async fn invoke_service_inner(
    state: AppState,
    service_id: String,
    headers: HeaderMap,
    input: Value,
) -> Result<Response, GatewayError> {
    let service = find_service(state.model.as_ref(), &service_id)?;
    let consent_mode = service_consent_mode(service);
    let agent_id = headers.get("x-web4-agent-id").and_then(|v| v.to_str().ok());
    enforce_agent_policy(service, agent_id)?;
    enforce_rate_limit(&state, service, &service_id, agent_id)?;

    let capability_token = headers
        .get(header::AUTHORIZATION)
        .and_then(|v| v.to_str().ok())
        .and_then(|raw| raw.strip_prefix("W4-Capability "))
        .map(str::to_string);
    if consent_mode == "capability" {
        let token = capability_token.as_deref().ok_or_else(|| {
            GatewayError(RuntimeError::new(
                ErrorCode::Unauthorized,
                "capability token required",
                false,
            ))
        })?;
        validate_capability_token(
            token,
            &service_id,
            &document_issuer(state.model.as_ref()),
            state.jwt_secret.as_bytes(),
        )?;
    }

    let interactive_approved = if consent_mode == "interactive" {
        let challenge_id = headers
            .get("x-web4-challenge")
            .and_then(|v| v.to_str().ok())
            .ok_or_else(|| {
                GatewayError(RuntimeError::new(
                    ErrorCode::ConsentRequired,
                    "interactive challenge required",
                    false,
                ))
            })?;
        consume_interactive_challenge(&state, challenge_id, &service_id)?;
        true
    } else {
        false
    };

    let sanitized_input = strip_reserved_policy_fields(input);

    let output = state
        .runtime
        .invoke(
            state.model.as_ref(),
            &service_id,
            sanitized_input,
            web4_core::InvocationContext {
                capability_token,
                interactive_approved,
            },
        )
        .await
        .map_err(|e| GatewayError(e.into_runtime()))?;

    let mut response = Json(output).into_response();
    if let Some(origin) = headers.get(header::ORIGIN).and_then(|v| v.to_str().ok()) {
        if let Some(allowed) = resolve_allow_origin(service, origin) {
            response.headers_mut().insert(
                header::ACCESS_CONTROL_ALLOW_ORIGIN,
                HeaderValue::from_str(&allowed).map_err(|_| {
                    GatewayError(RuntimeError::new(
                        ErrorCode::InternalError,
                        "invalid allow origin header",
                        false,
                    ))
                })?,
            );
        }
    }

    Ok(response)
}

pub async fn sample_error(Path(code): Path<String>) -> Result<Json<Value>, GatewayError> {
    let err = RuntimeError::new(ErrorCode::from(code), "sample error", false);
    Err(GatewayError(err))
}

fn require_admin_token(headers: &HeaderMap, expected_token: &str) -> Result<(), GatewayError> {
    let provided = headers
        .get(header::AUTHORIZATION)
        .and_then(|v| v.to_str().ok())
        .and_then(|raw| raw.strip_prefix("Bearer "))
        .ok_or_else(|| {
            GatewayError(RuntimeError::new(
                ErrorCode::Unauthorized,
                "admin bearer token required",
                false,
            ))
        })?;

    if provided == expected_token {
        return Ok(());
    }

    Err(GatewayError(RuntimeError::new(
        ErrorCode::Forbidden,
        "invalid admin bearer token",
        false,
    )))
}
