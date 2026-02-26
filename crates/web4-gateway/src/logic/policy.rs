use crate::error::GatewayError;
use crate::logic::common::unix_now;
use crate::state::AppState;
use serde_json::Value;
use web4_core::{ErrorCode, RuntimeError};

pub fn policy_value<'a>(service: &'a Value, field: &str) -> Option<&'a str> {
    service
        .get("policy")
        .and_then(|p| p.get(field))
        .and_then(|f| f.get("value"))
        .and_then(Value::as_str)
}

pub fn split_policy_list(value: &str) -> Vec<String> {
    value
        .split(',')
        .map(str::trim)
        .filter(|v| !v.is_empty())
        .map(str::to_string)
        .collect()
}

pub fn enforce_agent_policy(service: &Value, agent_id: Option<&str>) -> Result<(), GatewayError> {
    let Some(allow_agents) = policy_value(service, "allowAgents") else {
        return Ok(());
    };
    if allow_agents == "*" {
        return Ok(());
    }

    let agent = agent_id.ok_or_else(|| {
        GatewayError(RuntimeError::new(
            ErrorCode::Forbidden,
            "agent identity required by allowAgents policy",
            false,
        ))
    })?;

    if split_policy_list(allow_agents)
        .iter()
        .any(|allowed| allowed == agent)
    {
        return Ok(());
    }

    Err(GatewayError(RuntimeError::new(
        ErrorCode::Forbidden,
        "agent not allowed by policy",
        false,
    )))
}

pub fn enforce_rate_limit(
    state: &AppState,
    service: &Value,
    service_id: &str,
    agent_id: Option<&str>,
) -> Result<(), GatewayError> {
    let Some(raw) = policy_value(service, "rateLimit") else {
        return Ok(());
    };
    let (limit, window_secs) = parse_rate_limit(raw)?;
    let agent = agent_id.unwrap_or("anonymous");
    let key = format!("{service_id}:{agent}");

    state
        .rate_limiter
        .enforce(key, limit, window_secs, unix_now())
        .map_err(GatewayError)
}

pub fn parse_rate_limit(value: &str) -> Result<(u64, u64), GatewayError> {
    let (limit_raw, unit) = value.split_once('/').ok_or_else(|| {
        GatewayError(RuntimeError::new(
            ErrorCode::InvalidArgument,
            "invalid rateLimit format",
            false,
        ))
    })?;
    let limit = limit_raw.parse::<u64>().map_err(|_| {
        GatewayError(RuntimeError::new(
            ErrorCode::InvalidArgument,
            "invalid rateLimit value",
            false,
        ))
    })?;
    let seconds = match unit {
        "s" => 1,
        "m" => 60,
        "h" => 3600,
        "d" => 86400,
        _ => {
            return Err(GatewayError(RuntimeError::new(
                ErrorCode::InvalidArgument,
                "invalid rateLimit unit",
                false,
            )))
        }
    };
    Ok((limit, seconds))
}

pub fn resolve_allow_origin(service: &Value, request_origin: &str) -> Option<String> {
    let allow_origins = policy_value(service, "allowOrigins")?;
    if allow_origins == "*" {
        return Some("*".to_string());
    }

    split_policy_list(allow_origins)
        .into_iter()
        .find(|origin| origin == request_origin)
}

pub fn strip_reserved_policy_fields(input: Value) -> Value {
    let Value::Object(mut map) = input else {
        return input;
    };
    for key in [
        "policy",
        "consent",
        "effects",
        "allowAgents",
        "allowOrigins",
        "rateLimit",
    ] {
        map.remove(key);
    }
    Value::Object(map)
}
