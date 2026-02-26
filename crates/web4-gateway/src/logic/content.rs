use crate::error::GatewayError;
use serde_json::{json, Value};
use std::cmp::Ordering;
use web4_core::{
    compile_to_w4_json, w4_type_to_json_schema, ErrorCode, RuntimeError, ToolCompileInput,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum View {
    W4ml,
    Html,
    W4Json,
}

pub fn negotiate_view(accept: Option<&str>) -> Result<View, GatewayError> {
    let Some(accept) = accept else {
        return Ok(View::Html);
    };

    let mut best: Option<(f32, u8, usize, View)> = None;

    for (order, raw_part) in accept.split(',').enumerate() {
        let part = raw_part.trim();
        if part.is_empty() {
            continue;
        }

        let (media_type, q) = parse_accept_part(part);
        if q <= 0.0 {
            continue;
        }

        for (view, supported) in [
            (View::W4ml, "application/w4ml+xml"),
            (View::Html, "text/html"),
            (View::W4Json, "application/w4+json"),
        ] {
            if let Some(specificity) = media_match_score(media_type, supported) {
                let candidate = (q, specificity, usize::MAX - order, view);
                if best
                    .as_ref()
                    .map(|current| compare_choice(&candidate, current) == Ordering::Greater)
                    .unwrap_or(true)
                {
                    best = Some(candidate);
                }
            }
        }
    }

    best.map(|(_, _, _, view)| view).ok_or_else(|| {
        GatewayError(RuntimeError::new(
            ErrorCode::InvalidArgument,
            "no acceptable media type",
            false,
        ))
    })
}

fn parse_accept_part(part: &str) -> (&str, f32) {
    let mut q = 1.0;
    let mut media = part;

    for (idx, section) in part.split(';').enumerate() {
        if idx == 0 {
            media = section.trim();
            continue;
        }
        let trimmed = section.trim();
        if let Some(value) = trimmed.strip_prefix("q=") {
            q = value.parse::<f32>().unwrap_or(0.0);
        }
    }

    (media, q)
}

fn media_match_score(requested: &str, supported: &str) -> Option<u8> {
    if requested == "*/*" {
        return Some(0);
    }
    if requested == supported {
        return Some(2);
    }

    let (req_type, req_subtype) = requested.split_once('/')?;
    let (sup_type, _) = supported.split_once('/')?;
    if req_subtype == "*" && req_type == sup_type {
        return Some(1);
    }

    None
}

fn compare_choice(a: &(f32, u8, usize, View), b: &(f32, u8, usize, View)) -> Ordering {
    a.0.partial_cmp(&b.0)
        .unwrap_or(Ordering::Equal)
        .then_with(|| a.1.cmp(&b.1))
        .then_with(|| a.2.cmp(&b.2))
}

pub fn select_fragment(ir: &Value, selector: &str) -> Result<Value, GatewayError> {
    if let Some(id) = selector.strip_prefix("section:") {
        let section = ir
            .get("sections")
            .and_then(Value::as_array)
            .and_then(|arr| {
                arr.iter()
                    .find(|item| item.get("id").and_then(Value::as_str) == Some(id))
                    .cloned()
            })
            .ok_or_else(|| {
                GatewayError(RuntimeError::new(
                    ErrorCode::NotFound,
                    format!("section not found: {id}"),
                    false,
                ))
            })?;

        return Ok(json!({
            "doc": ir.get("doc").cloned().unwrap_or_else(|| json!({})),
            "types": [],
            "sections": [section],
            "services": [],
        }));
    }

    if let Some(id) = selector.strip_prefix("service:") {
        let service = ir
            .get("services")
            .and_then(Value::as_array)
            .and_then(|arr| {
                arr.iter()
                    .find(|item| item.get("id").and_then(Value::as_str) == Some(id))
                    .cloned()
            })
            .ok_or_else(|| {
                GatewayError(RuntimeError::new(
                    ErrorCode::NotFound,
                    format!("service not found: {id}"),
                    false,
                ))
            })?;

        let mut type_ids = Vec::new();
        for key in ["input", "output"] {
            if let Some(type_id) = service
                .get(key)
                .and_then(|v| v.get("id"))
                .and_then(Value::as_str)
            {
                type_ids.push(type_id.to_string());
            }
        }

        let types = ir
            .get("types")
            .and_then(Value::as_array)
            .map(|arr| {
                arr.iter()
                    .filter(|typ| {
                        typ.get("id")
                            .and_then(Value::as_str)
                            .map(|id| type_ids.iter().any(|expected| expected == id))
                            .unwrap_or(false)
                    })
                    .cloned()
                    .collect::<Vec<_>>()
            })
            .unwrap_or_default();

        return Ok(json!({
            "doc": ir.get("doc").cloned().unwrap_or_else(|| json!({})),
            "types": types,
            "sections": [],
            "services": [service],
        }));
    }

    Err(GatewayError(RuntimeError::new(
        ErrorCode::InvalidArgument,
        "invalid w4fragment selector; expected section:<id> or service:<id>",
        false,
    )))
}

pub fn compile_view_json(ir: &Value) -> Value {
    let canonical_uri = ir
        .get("doc")
        .and_then(|d| d.get("id"))
        .and_then(Value::as_str)
        .unwrap_or("https://example.com/index.w4");

    let tools = ir
        .get("services")
        .and_then(Value::as_array)
        .map(|services| {
            services
                .iter()
                .map(|service| {
                    let service_id = service
                        .get("id")
                        .and_then(Value::as_str)
                        .unwrap_or("service.unknown")
                        .to_string();
                    let description = service
                        .get("intents")
                        .and_then(Value::as_array)
                        .and_then(|intents| intents.first())
                        .and_then(|i| i.get("text"))
                        .and_then(Value::as_str)
                        .unwrap_or("No intent")
                        .to_string();
                    let parameters = service
                        .get("input")
                        .map(w4_type_to_json_schema)
                        .unwrap_or_else(|| json!({"type": "object"}));

                    let effects = service
                        .get("effects")
                        .and_then(Value::as_str)
                        .unwrap_or("unknown")
                        .to_string();
                    let consent_mode = service
                        .get("consent")
                        .and_then(|c| c.get("mode"))
                        .and_then(Value::as_str)
                        .unwrap_or("capability")
                        .to_string();
                    let bindings = service
                        .get("source_ref")
                        .and_then(Value::as_str)
                        .map(|source_ref| {
                            vec![json!({
                                "type": "http",
                                "endpoint": format!("/{}", source_ref.trim_start_matches('/')),
                            })]
                        })
                        .unwrap_or_default();
                    let load = service
                        .get("load")
                        .and_then(Value::as_str)
                        .unwrap_or("eager")
                        .to_string();

                    compile_to_w4_json(&ToolCompileInput {
                        service_id,
                        description,
                        parameters,
                        canonical_uri: canonical_uri.to_string(),
                        effects,
                        consent_mode,
                        bindings,
                        load,
                    })
                })
                .collect::<Vec<_>>()
        })
        .unwrap_or_default();

    let links = ir
        .get("doc")
        .and_then(|d| d.get("links"))
        .and_then(Value::as_array)
        .cloned()
        .unwrap_or_default();
    let imports = links
        .iter()
        .filter(|link| link.get("rel").and_then(Value::as_str) == Some("import"))
        .cloned()
        .collect::<Vec<_>>();
    let peers = links
        .iter()
        .filter(|link| link.get("rel").and_then(Value::as_str) == Some("peer"))
        .cloned()
        .collect::<Vec<_>>();

    json!({
        "doc": ir.get("doc").cloned().unwrap_or_else(|| json!({})),
        "imports": imports,
        "peers": peers,
        "tools": tools,
    })
}

pub fn render_w4ml_fragment(ir: &Value) -> String {
    let doc_id = ir
        .get("doc")
        .and_then(|d| d.get("id"))
        .and_then(Value::as_str)
        .unwrap_or("https://example.com/index.w4");

    let mut out = String::new();
    out.push_str("<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n");
    out.push_str(&format!(
        "<w4 xmlns=\"urn:w4ml:0.1\" version=\"0.1\" id=\"{doc_id}\">\n"
    ));
    out.push_str("  <head/>\n  <body>\n");

    if let Some(sections) = ir.get("sections").and_then(Value::as_array) {
        for section in sections {
            let id = section
                .get("id")
                .and_then(Value::as_str)
                .unwrap_or("section");
            out.push_str(&format!("    <section id=\"{id}\"/>\n"));
        }
    }

    if let Some(types) = ir.get("types").and_then(Value::as_array) {
        if !types.is_empty() {
            out.push_str("    <schema>\n");
            for typ in types {
                let id = typ.get("id").and_then(Value::as_str).unwrap_or("Type");
                let kind = typ.get("kind").and_then(Value::as_str).unwrap_or("object");
                out.push_str(&format!("      <type id=\"{id}\" kind=\"{kind}\"/>\n"));
            }
            out.push_str("    </schema>\n");
        }
    }

    if let Some(services) = ir.get("services").and_then(Value::as_array) {
        for service in services {
            let id = service
                .get("id")
                .and_then(Value::as_str)
                .unwrap_or("service");
            out.push_str(&format!("    <service id=\"{id}\"/>\n"));
        }
    }

    out.push_str("  </body>\n</w4>");
    out
}
