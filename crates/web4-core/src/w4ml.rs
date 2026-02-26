use crate::error::{ErrorCode, RuntimeError, Web4Error};
use crate::model::{ModelIndex, ParseResult, ValidationTarget};
use crate::traits::Validator;
use regex::Regex;
use roxmltree::{Document, Node};
use serde_json::{json, Map, Value};
use std::collections::{HashMap, HashSet};
use std::sync::OnceLock;

const W4_NS: &str = "urn:w4ml:0.1";

fn section_id_re() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| Regex::new(r"^[a-z][a-z0-9_-]*$").expect("valid regex"))
}

fn service_id_re() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| Regex::new(r"^[a-z][a-z0-9._-]*$").expect("valid regex"))
}

fn type_id_re() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| Regex::new(r"^[A-Za-z][A-Za-z0-9_]*$").expect("valid regex"))
}

fn property_name_re() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| Regex::new(r"^[a-z_][a-z0-9_]*$").expect("valid regex"))
}

fn binding_ref_re() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| Regex::new(r"^[A-Za-z0-9._/-]+\.w4s$").expect("valid regex"))
}

#[derive(Default)]
pub struct W4mlParser;

pub struct W4mlValidator;

impl W4mlParser {
    pub fn parse(&self, source: &str) -> Result<ParseResult, Web4Error> {
        parse_w4ml(source)
    }
}

impl Validator for W4mlValidator {
    fn validate(&self, target: &ValidationTarget) -> Result<(), Web4Error> {
        match target {
            ValidationTarget::NormalizedModel { model } => validate_normalized_model(model),
            ValidationTarget::JsonSchema { .. } => Ok(()),
        }
    }
}

fn parse_w4ml(source: &str) -> Result<ParseResult, Web4Error> {
    let doc = Document::parse(source).map_err(|err| {
        let detail = json!({
            "line": err.pos().row,
            "column": err.pos().col,
            "source": err.to_string(),
        });
        Web4Error::Runtime(
            RuntimeError::new(ErrorCode::InvalidArgument, "fatal parse error", false)
                .with_details(detail),
        )
    })?;

    let root = doc.root_element();
    if root.tag_name().name() != "w4" {
        return Err(fatal("root element must be w4"));
    }

    if root.tag_name().namespace() != Some(W4_NS) {
        return Err(fatal("root xmlns must be urn:w4ml:0.1"));
    }

    let mut warnings = Vec::new();
    if let Some(version) = root.attribute("version") {
        if version != "0.1" {
            warnings.push(format!("unknown w4 version: {version}"));
        }
    } else {
        return Err(fatal("missing required root attribute version"));
    }

    let mut head = None;
    let mut body = None;

    for node in root.children().filter(|n| n.is_element()) {
        if !is_core(node) {
            continue;
        }
        match node.tag_name().name() {
            "head" => head = Some(node),
            "body" => body = Some(node),
            _ => {}
        }
    }

    let head = head.ok_or_else(|| fatal("missing head element"))?;
    let body = body.ok_or_else(|| fatal("missing body element"))?;

    if head.range().start > body.range().start {
        return Err(fatal("head must appear before body"));
    }

    let doc_id = root.attribute("id").map(str::to_string);
    let titles = parse_titles(head);
    let metas = parse_metas(head);
    let links = parse_links(head);

    let mut sections = Vec::new();
    let mut services = Vec::new();
    let mut types = Vec::new();
    let mut extensions = Vec::new();
    let mut body_order = Vec::new();

    let mut schema_seen = false;

    for node in body.children().filter(|n| n.is_element()) {
        if is_extension(node) {
            extensions.push(parse_extension(node));
            continue;
        }

        if !is_core(node) {
            continue;
        }

        match node.tag_name().name() {
            "section" => {
                let parsed = parse_section(node)?;
                if let Some(id) = parsed.get("id").and_then(Value::as_str) {
                    body_order.push(json!({"kind": "section", "id": id}));
                }
                sections.push(parsed);
            }
            "schema" => {
                if schema_seen {
                    return Err(fatal("multiple schema elements are not allowed"));
                }
                schema_seen = true;
                types = parse_schema(node)?;
            }
            "service" => {
                let parsed = parse_service(node)?;
                if let Some(id) = parsed.get("id").and_then(Value::as_str) {
                    body_order.push(json!({"kind": "service", "id": id}));
                }
                services.push(parsed);
            }
            _ => {}
        }
    }

    semantic_validate(&sections, &services, &types)?;

    let types_by_id: HashMap<String, Value> = types
        .iter()
        .map(|t| (t["id"].as_str().unwrap_or_default().to_string(), t.clone()))
        .collect();

    for service in &mut services {
        if let Some(type_ref) = service
            .get("input_type_ref")
            .and_then(Value::as_str)
            .map(str::to_string)
        {
            let resolved = types_by_id
                .get(&type_ref)
                .ok_or_else(|| invalid(format!("unresolved input schema: {type_ref}")))?;
            service["input"] = resolved.clone();
        }

        if let Some(type_ref) = service
            .get("output_type_ref")
            .and_then(Value::as_str)
            .map(str::to_string)
        {
            let resolved = types_by_id
                .get(&type_ref)
                .ok_or_else(|| invalid(format!("unresolved output schema: {type_ref}")))?;
            service["output"] = resolved.clone();
        }

        if let Some(obj) = service.as_object_mut() {
            obj.remove("input_type_ref");
            obj.remove("output_type_ref");
        }
    }

    let model = json!({
        "doc": {
            "id": doc_id,
            "titles": titles,
            "meta": metas,
            "links": links,
        },
        "types": types,
        "sections": sections,
        "services": services,
        "body_order": body_order,
        "extensions": extensions,
    });

    let index = build_index(&model);
    let validator = W4mlValidator;
    validator.validate(&ValidationTarget::NormalizedModel {
        model: model.clone(),
    })?;

    Ok(ParseResult {
        raw: source.to_string(),
        model: Some(model),
        warnings,
        index: Some(index),
    })
}

fn validate_normalized_model(model: &Value) -> Result<(), Web4Error> {
    let sections = model
        .get("sections")
        .and_then(Value::as_array)
        .ok_or_else(|| invalid("model.sections must be an array"))?;
    let services = model
        .get("services")
        .and_then(Value::as_array)
        .ok_or_else(|| invalid("model.services must be an array"))?;
    let types = model
        .get("types")
        .and_then(Value::as_array)
        .ok_or_else(|| invalid("model.types must be an array"))?;

    let mut ids = HashSet::new();

    for section in sections {
        let id = section
            .get("id")
            .and_then(Value::as_str)
            .ok_or_else(|| invalid("section.id missing"))?;
        if !section_id_re().is_match(id) {
            return Err(invalid(format!("invalid section id: {id}")));
        }
        ids.insert(id.to_string());
    }

    for service in services {
        let id = service
            .get("id")
            .and_then(Value::as_str)
            .ok_or_else(|| invalid("service.id missing"))?;
        if !service_id_re().is_match(id) {
            return Err(invalid(format!("invalid service id: {id}")));
        }
        if !ids.insert(id.to_string()) {
            return Err(invalid(format!(
                "duplicate id across section/service: {id}"
            )));
        }
        let binding_ref = service
            .get("source_ref")
            .and_then(Value::as_str)
            .ok_or_else(|| invalid("service.source_ref missing"))?;
        if !binding_ref_re().is_match(binding_ref) {
            return Err(invalid(format!(
                "invalid service source_ref: {binding_ref}"
            )));
        }
    }

    let mut type_ids = HashSet::new();
    for typ in types {
        let id = typ
            .get("id")
            .and_then(Value::as_str)
            .ok_or_else(|| invalid("type.id missing"))?;
        if !type_id_re().is_match(id) {
            return Err(invalid(format!("invalid type id: {id}")));
        }
        if !type_ids.insert(id.to_string()) {
            return Err(invalid(format!("duplicate type id: {id}")));
        }
    }

    Ok(())
}

fn semantic_validate(
    sections: &[Value],
    services: &[Value],
    types: &[Value],
) -> Result<(), Web4Error> {
    let mut ids = HashSet::new();

    for section in sections {
        let id = value_str(section, "id")?;
        if !section_id_re().is_match(id) {
            return Err(invalid(format!("section id does not match pattern: {id}")));
        }
        if !ids.insert(id.to_string()) {
            return Err(invalid(format!("duplicate id: {id}")));
        }
    }

    for service in services {
        let id = value_str(service, "id")?;
        if !service_id_re().is_match(id) {
            return Err(invalid(format!("service id does not match pattern: {id}")));
        }
        if !ids.insert(id.to_string()) {
            return Err(invalid(format!("duplicate id: {id}")));
        }
        let binding_ref = value_str(service, "source_ref")?;
        if !binding_ref_re().is_match(binding_ref) {
            return Err(invalid(format!(
                "service source_ref does not match pattern: {binding_ref}"
            )));
        }
    }

    let mut type_ids = HashSet::new();
    for typ in types {
        let id = value_str(typ, "id")?;
        if !type_id_re().is_match(id) {
            return Err(invalid(format!("type id does not match pattern: {id}")));
        }
        if !type_ids.insert(id.to_string()) {
            return Err(invalid(format!("duplicate type id: {id}")));
        }

        if let Some(properties) = typ.get("properties").and_then(Value::as_array) {
            for prop in properties {
                let name = value_str(prop, "name")?;
                if !property_name_re().is_match(name) {
                    return Err(invalid(format!(
                        "property name does not match pattern: {name}"
                    )));
                }
            }
        }
    }

    Ok(())
}

fn parse_titles(head: Node<'_, '_>) -> Vec<Value> {
    head.children()
        .filter(|n| n.is_element() && is_core(*n) && n.tag_name().name() == "title")
        .map(|n| {
            json!({
                "lang": n.attribute("lang"),
                "text": normalized_text(n),
            })
        })
        .collect()
}

fn parse_metas(head: Node<'_, '_>) -> Vec<Value> {
    head.children()
        .filter(|n| n.is_element() && is_core(*n) && n.tag_name().name() == "meta")
        .map(|n| {
            json!({
                "name": n.attribute("name"),
                "content": n.attribute("content"),
            })
        })
        .collect()
}

fn parse_links(head: Node<'_, '_>) -> Vec<Value> {
    head.children()
        .filter(|n| n.is_element() && is_core(*n) && n.tag_name().name() == "link")
        .map(|n| {
            json!({
                "rel": n.attribute("rel"),
                "href": n.attribute("href"),
                "type": n.attribute("type"),
            })
        })
        .collect()
}

fn parse_section(node: Node<'_, '_>) -> Result<Value, Web4Error> {
    for child in node.children().filter(|n| n.is_element()) {
        if is_core(child) && child.tag_name().name() == "section" {
            return Err(fatal("section elements shall not be nested"));
        }
    }

    let id = required_attr(node, "id")?;
    let text = normalized_text(node);
    Ok(json!({
        "id": id,
        "name": node.attribute("name"),
        "load": node.attribute("load").unwrap_or("eager"),
        "class": node.attribute("class"),
        "content": text,
    }))
}

fn parse_schema(node: Node<'_, '_>) -> Result<Vec<Value>, Web4Error> {
    let mut types = Vec::new();
    for type_node in node.children().filter(|n| n.is_element()) {
        if !is_core(type_node) {
            continue;
        }
        if type_node.tag_name().name() != "type" {
            continue;
        }

        let id = required_attr(type_node, "id")?;
        let kind = required_attr(type_node, "kind")?;

        let mut properties = Vec::new();
        let mut additional_properties = None;

        for child in type_node.children().filter(|n| n.is_element()) {
            if !is_core(child) {
                continue;
            }
            match child.tag_name().name() {
                "property" => {
                    let mut property = Map::new();
                    for attr in child.attributes() {
                        property.insert(
                            attr.name().to_string(),
                            Value::String(attr.value().to_string()),
                        );
                    }
                    properties.push(Value::Object(property));
                }
                "additionalProperties" => {
                    additional_properties = Some(child.attribute("value") == Some("true"));
                }
                _ => {}
            }
        }

        types.push(json!({
            "id": id,
            "kind": kind,
            "properties": properties,
            "additionalProperties": additional_properties,
        }));
    }

    Ok(types)
}

fn parse_service(node: Node<'_, '_>) -> Result<Value, Web4Error> {
    let id = required_attr(node, "id")?;
    let binding_ref = required_attr(node, "sourceRef")?;

    if node
        .children()
        .any(|n| n.is_element() && is_core(n) && n.tag_name().name() == "bindings")
    {
        return Err(invalid(
            "inline bindings are not supported; declare service sourceRef to a .w4s file",
        ));
    }

    let intents = node
        .children()
        .filter(|n| n.is_element() && is_core(*n) && n.tag_name().name() == "intent")
        .map(|n| {
            json!({
                "lang": n.attribute("lang"),
                "text": normalized_text(n),
            })
        })
        .collect::<Vec<_>>();

    let input_type_ref = node
        .children()
        .find(|n| n.is_element() && is_core(*n) && n.tag_name().name() == "input")
        .and_then(|n| n.attribute("typeRef"));

    let output_type_ref = node
        .children()
        .find(|n| n.is_element() && is_core(*n) && n.tag_name().name() == "output")
        .and_then(|n| n.attribute("typeRef"));

    let effects = node
        .children()
        .find(|n| n.is_element() && is_core(*n) && n.tag_name().name() == "effects")
        .and_then(|n| n.attribute("level"))
        .unwrap_or("unknown");

    let consent_mode = node
        .children()
        .find(|n| n.is_element() && is_core(*n) && n.tag_name().name() == "consent")
        .and_then(|n| n.attribute("mode"))
        .unwrap_or(if effects == "none" {
            "open"
        } else {
            "capability"
        });

    Ok(json!({
        "id": id,
        "name": node.attribute("name"),
        "kind": node.attribute("kind").unwrap_or("tool"),
        "load": node.attribute("load").unwrap_or("eager"),
        "class": node.attribute("class"),
        "intents": intents,
        "input_type_ref": input_type_ref,
        "output_type_ref": output_type_ref,
        "input": null,
        "output": null,
        "effects": effects,
        "consent": {"mode": consent_mode},
        "policy": parse_policy(node),
        "errors": parse_errors(node),
        "source_ref": binding_ref,
        "bindings": [],
    }))
}

fn parse_policy(node: Node<'_, '_>) -> Value {
    let Some(policy) = node
        .children()
        .find(|n| n.is_element() && is_core(*n) && n.tag_name().name() == "policy")
    else {
        return json!({});
    };

    let mut map = Map::new();
    for field in policy.children().filter(|n| n.is_element() && is_core(*n)) {
        let mut entry = Map::new();
        for attr in field.attributes() {
            entry.insert(
                attr.name().to_string(),
                Value::String(attr.value().to_string()),
            );
        }
        map.insert(field.tag_name().name().to_string(), Value::Object(entry));
    }
    Value::Object(map)
}

fn parse_errors(node: Node<'_, '_>) -> Vec<Value> {
    let Some(errors) = node
        .children()
        .find(|n| n.is_element() && is_core(*n) && n.tag_name().name() == "errors")
    else {
        return Vec::new();
    };

    errors
        .children()
        .filter(|n| n.is_element() && is_core(*n) && n.tag_name().name() == "error")
        .map(|err| {
            json!({
                "code": err.attribute("code"),
                "retryable": err.attribute("retryable").unwrap_or("false") == "true",
            })
        })
        .collect()
}

fn parse_extension(node: Node<'_, '_>) -> Value {
    let mut attrs = Map::new();
    for attr in node.attributes() {
        attrs.insert(
            attr.name().to_string(),
            Value::String(attr.value().to_string()),
        );
    }

    json!({
        "tag": node.tag_name().name(),
        "namespace": node.tag_name().namespace(),
        "attributes": attrs,
        "raw_text": normalized_text(node),
    })
}

fn build_index(model: &Value) -> ModelIndex {
    let mut index = ModelIndex::default();

    if let Some(sections) = model.get("sections").and_then(Value::as_array) {
        for (i, section) in sections.iter().enumerate() {
            if let Some(id) = section.get("id").and_then(Value::as_str) {
                index.section_ids.insert(id.to_string(), i);
            }
        }
    }

    if let Some(services) = model.get("services").and_then(Value::as_array) {
        for (i, service) in services.iter().enumerate() {
            if let Some(id) = service.get("id").and_then(Value::as_str) {
                index.service_ids.insert(id.to_string(), i);
            }
        }
    }

    if let Some(types) = model.get("types").and_then(Value::as_array) {
        for (i, typ) in types.iter().enumerate() {
            if let Some(id) = typ.get("id").and_then(Value::as_str) {
                index.type_ids.insert(id.to_string(), i);
            }
        }
    }

    index
}

fn normalized_text(node: Node<'_, '_>) -> String {
    node.descendants()
        .filter_map(|n| n.text())
        .map(str::trim)
        .filter(|text| !text.is_empty())
        .collect::<Vec<_>>()
        .join(" ")
}

fn is_core(node: Node<'_, '_>) -> bool {
    node.tag_name().namespace() == Some(W4_NS)
}

fn is_extension(node: Node<'_, '_>) -> bool {
    matches!(node.tag_name().namespace(), Some(ns) if ns != W4_NS)
}

fn required_attr<'a>(node: Node<'a, 'a>, attr: &str) -> Result<String, Web4Error> {
    node.attribute(attr)
        .map(str::to_string)
        .ok_or_else(|| invalid(format!("missing required attribute {attr}")))
}

fn value_str<'a>(value: &'a Value, key: &str) -> Result<&'a str, Web4Error> {
    value
        .get(key)
        .and_then(Value::as_str)
        .ok_or_else(|| invalid(format!("missing field {key}")))
}

fn fatal(message: impl Into<String>) -> Web4Error {
    Web4Error::Runtime(RuntimeError::new(
        ErrorCode::InvalidArgument,
        message,
        false,
    ))
}

fn invalid(message: impl Into<String>) -> Web4Error {
    Web4Error::Runtime(RuntimeError::new(
        ErrorCode::InvalidArgument,
        message,
        false,
    ))
}
