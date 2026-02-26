use crate::config::RenderingConfig;
use crate::error::GatewayError;
use serde::Serialize;
use serde_json::Value;
use std::path::{Path, PathBuf};
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use tera::{Context as TeraContext, Tera};

#[derive(Debug, Serialize)]
pub struct RenderItem {
    pub kind: String,
    pub id: String,
    pub title: String,
    pub content: String,
    pub class_attr: String,
    pub load: String,
    pub collapsed: bool,
    pub effects: Option<String>,
    pub consent_mode: Option<String>,
    pub source_ref: Option<String>,
}

#[derive(Debug, Serialize)]
struct PageContext {
    title: String,
    description: String,
    lang: String,
}

#[derive(Debug, Serialize)]
struct RequestContext {
    fragment: Option<String>,
}

#[derive(Debug, Serialize)]
struct RuntimeContext {
    template_href: Option<String>,
    template_source: String,
    generated_at: u64,
}

#[derive(Debug, Serialize)]
struct RenderStats {
    sections_total: usize,
    services_total: usize,
    eager_count: usize,
    lazy_count: usize,
    never_count: usize,
    visible_count: usize,
}

#[derive(Debug, Serialize)]
struct HtmlRenderContext {
    page: PageContext,
    doc: Value,
    sections: Vec<Value>,
    services: Vec<Value>,
    items: Vec<RenderItem>,
    stats: RenderStats,
    request: RequestContext,
    runtime: RuntimeContext,
}

pub struct RenderOptions<'a> {
    pub fragment_selector: Option<&'a str>,
    pub rendering: &'a RenderingConfig,
    pub document_root: &'a Path,
    pub document_dir: &'a Path,
}

const DEFAULT_HTML_TEMPLATE: &str = include_str!("../templates/default.html.tera");

pub async fn render_html(ir: &Value, options: &RenderOptions<'_>) -> String {
    let declared_href = declared_template_href(ir).map(str::to_string);
    let (template, template_source) = match resolve_template(ir, options).await {
        Ok((template, source)) => (template, source),
        Err(err) => {
            tracing::warn!(
                error = %err.0.message,
                code = %err.0.code,
                template_href = declared_href.as_deref().unwrap_or("<none>"),
                "template load failed; fallback to built-in default"
            );
            (
                DEFAULT_HTML_TEMPLATE.to_string(),
                "builtin-default-fallback".to_string(),
            )
        }
    };

    let context = build_render_context(ir, &declared_href, &template_source, options);

    render_template(&template, &context).unwrap_or_else(|err| {
        tracing::warn!(
            error = %err.0.message,
            code = %err.0.code,
            "template render failed; fallback to built-in default"
        );
        render_template(DEFAULT_HTML_TEMPLATE, &context).unwrap_or_else(|_| {
            "<!doctype html><html><body><h1>Web4 Document</h1></body></html>".to_string()
        })
    })
}

fn build_render_context(
    ir: &Value,
    declared_href: &Option<String>,
    template_source: &str,
    options: &RenderOptions<'_>,
) -> HtmlRenderContext {
    let doc = ir.get("doc").cloned().unwrap_or(Value::Null);
    let sections = ir
        .get("sections")
        .and_then(Value::as_array)
        .cloned()
        .unwrap_or_default();
    let services = ir
        .get("services")
        .and_then(Value::as_array)
        .cloned()
        .unwrap_or_default();
    let items = build_render_items(ir);

    let title = doc
        .get("titles")
        .and_then(Value::as_array)
        .and_then(|titles| titles.first())
        .and_then(|title| title.get("text"))
        .and_then(Value::as_str)
        .unwrap_or("Web4 Document")
        .to_string();
    let lang = doc
        .get("titles")
        .and_then(Value::as_array)
        .and_then(|titles| titles.first())
        .and_then(|title| title.get("lang"))
        .and_then(Value::as_str)
        .unwrap_or("en")
        .to_string();
    let description = doc
        .get("meta")
        .and_then(Value::as_array)
        .and_then(|metas| {
            metas
                .iter()
                .find(|meta| meta.get("name").and_then(Value::as_str) == Some("description"))
        })
        .and_then(|meta| meta.get("content"))
        .and_then(Value::as_str)
        .unwrap_or_default()
        .to_string();

    let eager_count = items.iter().filter(|item| item.load == "eager").count();
    let lazy_count = items.iter().filter(|item| item.load == "lazy").count();
    let visible_count = items.len();
    let never_count = sections
        .iter()
        .chain(services.iter())
        .filter(|entry| entry.get("load").and_then(Value::as_str) == Some("never"))
        .count();

    HtmlRenderContext {
        page: PageContext {
            title,
            description,
            lang,
        },
        doc,
        sections,
        services,
        items,
        stats: RenderStats {
            sections_total: ir
                .get("sections")
                .and_then(Value::as_array)
                .map_or(0, Vec::len),
            services_total: ir
                .get("services")
                .and_then(Value::as_array)
                .map_or(0, Vec::len),
            eager_count,
            lazy_count,
            never_count,
            visible_count,
        },
        request: RequestContext {
            fragment: options.fragment_selector.map(str::to_string),
        },
        runtime: RuntimeContext {
            template_href: declared_href.clone(),
            template_source: template_source.to_string(),
            generated_at: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .map_or(0, |dur| dur.as_secs()),
        },
    }
}

async fn resolve_template(
    ir: &Value,
    options: &RenderOptions<'_>,
) -> Result<(String, String), GatewayError> {
    let Some(href) = declared_template_href(ir) else {
        return Ok((
            DEFAULT_HTML_TEMPLATE.to_string(),
            "builtin-default".to_string(),
        ));
    };

    if href == "builtin:default" {
        return Ok((
            DEFAULT_HTML_TEMPLATE.to_string(),
            "builtin-default".to_string(),
        ));
    }

    let template = load_template(href, options).await?;
    Ok((template, "declared-template".to_string()))
}

pub fn build_render_items(ir: &Value) -> Vec<RenderItem> {
    let sections = ir
        .get("sections")
        .and_then(Value::as_array)
        .cloned()
        .unwrap_or_default();
    let services = ir
        .get("services")
        .and_then(Value::as_array)
        .cloned()
        .unwrap_or_default();

    let mut section_by_id = std::collections::HashMap::new();
    for section in &sections {
        if let Some(id) = section.get("id").and_then(Value::as_str) {
            section_by_id.insert(id.to_string(), section.clone());
        }
    }
    let mut service_by_id = std::collections::HashMap::new();
    for service in &services {
        if let Some(id) = service.get("id").and_then(Value::as_str) {
            service_by_id.insert(id.to_string(), service.clone());
        }
    }

    let mut items = Vec::new();
    if let Some(order) = ir.get("body_order").and_then(Value::as_array) {
        for slot in order {
            let kind = slot.get("kind").and_then(Value::as_str).unwrap_or_default();
            let id = slot.get("id").and_then(Value::as_str).unwrap_or_default();
            match kind {
                "section" => {
                    if let Some(section) = section_by_id.get(id) {
                        if let Some(item) = render_item_from_section(section) {
                            items.push(item);
                        }
                    }
                }
                "service" => {
                    if let Some(service) = service_by_id.get(id) {
                        if let Some(item) = render_item_from_service(service) {
                            items.push(item);
                        }
                    }
                }
                _ => {}
            }
        }
    } else {
        for section in &sections {
            if let Some(item) = render_item_from_section(section) {
                items.push(item);
            }
        }
        for service in &services {
            if let Some(item) = render_item_from_service(service) {
                items.push(item);
            }
        }
    }

    items
}

fn render_item_from_section(section: &Value) -> Option<RenderItem> {
    let id = section.get("id").and_then(Value::as_str)?.to_string();
    let load = section
        .get("load")
        .and_then(Value::as_str)
        .unwrap_or("eager")
        .to_string();
    if load == "never" {
        return None;
    }
    let collapsed = load == "lazy";
    Some(RenderItem {
        kind: "section".to_string(),
        title: section
            .get("name")
            .and_then(Value::as_str)
            .unwrap_or(&id)
            .to_string(),
        content: section
            .get("content")
            .and_then(Value::as_str)
            .unwrap_or_default()
            .to_string(),
        class_attr: section
            .get("class")
            .and_then(Value::as_str)
            .unwrap_or_default()
            .to_string(),
        id,
        load,
        collapsed,
        effects: None,
        consent_mode: None,
        source_ref: None,
    })
}

fn render_item_from_service(service: &Value) -> Option<RenderItem> {
    let id = service.get("id").and_then(Value::as_str)?.to_string();
    let load = service
        .get("load")
        .and_then(Value::as_str)
        .unwrap_or("eager")
        .to_string();
    if load == "never" {
        return None;
    }
    let collapsed = load == "lazy";
    let intent = service
        .get("intents")
        .and_then(Value::as_array)
        .and_then(|arr| arr.first())
        .and_then(|item| item.get("text"))
        .and_then(Value::as_str)
        .unwrap_or("No description")
        .to_string();
    Some(RenderItem {
        kind: "service".to_string(),
        title: service
            .get("name")
            .and_then(Value::as_str)
            .unwrap_or(&id)
            .to_string(),
        content: intent,
        class_attr: service
            .get("class")
            .and_then(Value::as_str)
            .unwrap_or_default()
            .to_string(),
        id,
        load,
        collapsed,
        effects: service
            .get("effects")
            .and_then(Value::as_str)
            .map(str::to_string),
        consent_mode: service
            .get("consent")
            .and_then(|consent| consent.get("mode"))
            .and_then(Value::as_str)
            .map(str::to_string),
        source_ref: service
            .get("source_ref")
            .and_then(Value::as_str)
            .map(str::to_string),
    })
}

fn declared_template_href(ir: &Value) -> Option<&str> {
    let links = ir
        .get("doc")
        .and_then(|doc| doc.get("links"))
        .and_then(Value::as_array)?;
    for link in links {
        if link.get("rel").and_then(Value::as_str) == Some("template") {
            return link.get("href").and_then(Value::as_str);
        }
    }
    None
}

async fn load_template(href: &str, options: &RenderOptions<'_>) -> Result<String, GatewayError> {
    let loader = &options.rendering.template_loader;

    if href.starts_with("http://") || href.starts_with("https://") {
        if !loader.allow_remote {
            return Err(invalid("remote template loading is disabled"));
        }
        let url = reqwest::Url::parse(href)
            .map_err(|err| invalid_owned(format!("invalid template url: {err}")))?;
        if !loader.allowed_remote_hosts.is_empty() {
            let Some(host) = url.host_str() else {
                return Err(invalid("template url host is missing"));
            };
            if !loader
                .allowed_remote_hosts
                .iter()
                .any(|allowed| allowed == host)
            {
                return Err(invalid_owned(format!("template host not allowed: {host}")));
            }
        }

        let client = reqwest::Client::builder()
            .timeout(Duration::from_millis(loader.timeout_ms))
            .build()
            .map_err(|err| invalid_owned(format!("template client init failed: {err}")))?;
        let response = client
            .get(url)
            .send()
            .await
            .map_err(|err| invalid_owned(format!("template fetch failed: {err}")))?
            .error_for_status()
            .map_err(|err| invalid_owned(format!("template fetch failed: {err}")))?;

        if let Some(size) = response.content_length() {
            if size > loader.max_bytes as u64 {
                return Err(invalid_owned(format!(
                    "template too large: {size} bytes (max {})",
                    loader.max_bytes
                )));
            }
        }

        let bytes = response
            .bytes()
            .await
            .map_err(|err| invalid_owned(format!("template read failed: {err}")))?;
        if bytes.len() > loader.max_bytes {
            return Err(invalid_owned(format!(
                "template too large: {} bytes (max {})",
                bytes.len(),
                loader.max_bytes
            )));
        }
        return String::from_utf8(bytes.to_vec())
            .map_err(|err| invalid_owned(format!("template is not valid UTF-8: {err}")));
    }

    let path = resolve_template_path(href, options.document_dir, options.document_root)?;
    let bytes = tokio::fs::read(path)
        .await
        .map_err(|err| invalid_owned(format!("template open failed: {err}")))?;
    if bytes.len() > loader.max_bytes {
        return Err(invalid_owned(format!(
            "template too large: {} bytes (max {})",
            bytes.len(),
            loader.max_bytes
        )));
    }

    String::from_utf8(bytes)
        .map_err(|err| invalid_owned(format!("template is not valid UTF-8: {err}")))
}

fn resolve_template_path(
    href: &str,
    document_dir: &Path,
    document_root: &Path,
) -> Result<PathBuf, GatewayError> {
    let raw = href.strip_prefix("file://").unwrap_or(href);
    let candidate = {
        let path = Path::new(raw);
        if path.is_absolute() {
            path.to_path_buf()
        } else {
            document_dir.join(path)
        }
    };

    let resolved = std::fs::canonicalize(&candidate)
        .map_err(|err| invalid_owned(format!("template resolve failed ({raw}): {err}")))?;
    if !resolved.starts_with(document_root) {
        return Err(invalid_owned(format!(
            "template path escapes document.root: {raw}"
        )));
    }
    Ok(resolved)
}

fn render_template(template: &str, context: &HtmlRenderContext) -> Result<String, GatewayError> {
    let mut tera = Tera::default();
    tera.autoescape_on(vec![".html", ".htm", ".xml"]);
    tera.add_raw_template("page.html", template)
        .map_err(|err| invalid_owned(format!("template parse failed: {err}")))?;

    let mut tera_context = TeraContext::new();
    tera_context.insert("page", &context.page);
    tera_context.insert("doc", &context.doc);
    tera_context.insert("sections", &context.sections);
    tera_context.insert("services", &context.services);
    tera_context.insert("items", &context.items);
    tera_context.insert("stats", &context.stats);
    tera_context.insert("request", &context.request);
    tera_context.insert("runtime", &context.runtime);

    tera.render("page.html", &tera_context)
        .map_err(|err| invalid_owned(format!("template render failed: {err}")))
}

fn invalid(message: &str) -> GatewayError {
    invalid_owned(message.to_string())
}

fn invalid_owned(message: String) -> GatewayError {
    GatewayError(web4_core::RuntimeError::new(
        web4_core::ErrorCode::InvalidArgument,
        message,
        false,
    ))
}
