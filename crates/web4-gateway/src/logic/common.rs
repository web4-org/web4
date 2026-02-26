use crate::error::GatewayError;
use serde_json::Value;
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use web4_core::{ErrorCode, RuntimeError};

pub fn unix_now() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or(Duration::from_secs(0))
        .as_secs()
}

pub fn document_issuer(ir: &Value) -> String {
    ir.get("doc")
        .and_then(|doc| doc.get("id"))
        .and_then(Value::as_str)
        .unwrap_or("https://example.com/index.w4")
        .to_string()
}

pub fn find_service<'a>(ir: &'a Value, service_id: &str) -> Result<&'a Value, GatewayError> {
    ir.get("services")
        .and_then(Value::as_array)
        .and_then(|services| {
            services
                .iter()
                .find(|svc| svc.get("id").and_then(Value::as_str) == Some(service_id))
        })
        .ok_or_else(|| {
            GatewayError(RuntimeError::new(
                ErrorCode::NotFound,
                format!("service not found: {service_id}"),
                false,
            ))
        })
}

pub fn find_service_by_source_ref<'a>(
    ir: &'a Value,
    source_ref: &str,
) -> Result<&'a Value, GatewayError> {
    ir.get("services")
        .and_then(Value::as_array)
        .and_then(|services| {
            services
                .iter()
                .find(|svc| svc.get("source_ref").and_then(Value::as_str) == Some(source_ref))
        })
        .ok_or_else(|| {
            GatewayError(RuntimeError::new(
                ErrorCode::NotFound,
                format!("service not found for sourceRef: {source_ref}"),
                false,
            ))
        })
}

pub fn service_consent_mode(service: &Value) -> &str {
    let effects = service
        .get("effects")
        .and_then(Value::as_str)
        .unwrap_or("unknown");
    service
        .get("consent")
        .and_then(|c| c.get("mode"))
        .and_then(Value::as_str)
        .unwrap_or(if effects == "none" {
            "open"
        } else {
            "capability"
        })
}
