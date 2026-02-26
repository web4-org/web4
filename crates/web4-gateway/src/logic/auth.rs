use crate::error::GatewayError;
use crate::logic::common::document_issuer;
use jsonwebtoken::{decode, encode, Algorithm, DecodingKey, EncodingKey, Header, Validation};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use web4_core::{ErrorCode, RuntimeError};

#[derive(Debug, Serialize, Deserialize)]
pub struct CapabilityClaims {
    pub iss: String,
    pub sub: String,
    pub exp: usize,
    pub scope: String,
}

pub fn issue_capability_token(
    ir: &Value,
    jwt_secret: &[u8],
    service_id: String,
    sub: Option<String>,
    ttl_seconds: Option<u64>,
) -> Result<(String, u64), GatewayError> {
    let now = crate::logic::common::unix_now();
    let ttl = ttl_seconds.unwrap_or(300).min(900);
    let exp = now + ttl;
    let claims = CapabilityClaims {
        iss: document_issuer(ir),
        sub: sub.unwrap_or_else(|| "anonymous".to_string()),
        exp: exp as usize,
        scope: service_id,
    };
    let token = encode(
        &Header::new(Algorithm::HS256),
        &claims,
        &EncodingKey::from_secret(jwt_secret),
    )
    .map_err(|err| {
        GatewayError(RuntimeError::new(
            ErrorCode::InternalError,
            err.to_string(),
            false,
        ))
    })?;

    Ok((token, exp))
}

pub fn validate_capability_token(
    token: &str,
    service_id: &str,
    issuer: &str,
    secret: &[u8],
) -> Result<(), GatewayError> {
    let mut validation = Validation::new(Algorithm::HS256);
    validation.validate_exp = true;
    let data = decode::<CapabilityClaims>(token, &DecodingKey::from_secret(secret), &validation)
        .map_err(|err| {
            GatewayError(RuntimeError::new(
                ErrorCode::Unauthorized,
                format!("invalid capability token: {err}"),
                false,
            ))
        })?;

    if data.claims.iss != issuer {
        return Err(GatewayError(RuntimeError::new(
            ErrorCode::Forbidden,
            "capability issuer mismatch",
            false,
        )));
    }

    let allowed = data
        .claims
        .scope
        .split_whitespace()
        .any(|scope| scope == service_id);
    if !allowed {
        return Err(GatewayError(RuntimeError::new(
            ErrorCode::Forbidden,
            "capability scope does not authorize this service",
            false,
        )));
    }

    Ok(())
}
