use serde::Deserialize;
use std::{fs, path::Path};
use web4_core::{ErrorCode, RuntimeError};

use crate::error::GatewayError;

pub const DEFAULT_CONFIG_PATH: &str = "config.json";

#[derive(Debug, Clone, Deserialize)]
pub struct GatewayConfig {
    pub server: ServerConfig,
    pub document: DocumentConfig,
    pub runtime: RuntimeConfig,
    pub security: SecurityConfig,
    #[serde(default)]
    pub debug: DebugConfig,
    #[serde(default)]
    pub rendering: RenderingConfig,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ServerConfig {
    pub bind_addr: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct DocumentConfig {
    pub root: String,
    pub entry_w4: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct RuntimeConfig {
    pub http_base_url: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct SecurityConfig {
    pub jwt_secret: String,
    pub admin_token: String,
}

#[derive(Debug, Clone, Deserialize, Default)]
pub struct DebugConfig {
    #[serde(default)]
    pub enable_error_route: bool,
}

#[derive(Debug, Clone, Deserialize)]
pub struct RenderingConfig {
    #[serde(default)]
    pub template_loader: TemplateLoaderConfig,
}

impl Default for RenderingConfig {
    fn default() -> Self {
        Self {
            template_loader: TemplateLoaderConfig::default(),
        }
    }
}

#[derive(Debug, Clone, Deserialize)]
pub struct TemplateLoaderConfig {
    #[serde(default)]
    pub allow_remote: bool,
    #[serde(default)]
    pub allowed_remote_hosts: Vec<String>,
    #[serde(default = "default_template_timeout_ms")]
    pub timeout_ms: u64,
    #[serde(default = "default_template_max_bytes")]
    pub max_bytes: usize,
}

impl Default for TemplateLoaderConfig {
    fn default() -> Self {
        Self {
            allow_remote: false,
            allowed_remote_hosts: Vec::new(),
            timeout_ms: default_template_timeout_ms(),
            max_bytes: default_template_max_bytes(),
        }
    }
}

impl GatewayConfig {
    pub fn from_file(path: &Path) -> Result<Self, GatewayError> {
        let raw = fs::read_to_string(path).map_err(|err| {
            GatewayError(RuntimeError::new(
                ErrorCode::InvalidArgument,
                format!("read gateway config failed ({}): {err}", path.display()),
                false,
            ))
        })?;
        let mut config: GatewayConfig = serde_json::from_str(&raw).map_err(|err| {
            GatewayError(RuntimeError::new(
                ErrorCode::InvalidArgument,
                format!("parse gateway config failed ({}): {err}", path.display()),
                false,
            ))
        })?;
        config.normalize();
        config.validate()?;
        Ok(config)
    }

    pub fn default_path() -> &'static str {
        DEFAULT_CONFIG_PATH
    }

    pub fn http_base_url(&self) -> String {
        self.runtime
            .http_base_url
            .clone()
            .unwrap_or_else(|| format!("http://{}", self.server.bind_addr))
    }

    fn normalize(&mut self) {
        if self.runtime.http_base_url.is_none() {
            self.runtime.http_base_url = Some(format!("http://{}", self.server.bind_addr));
        }
    }

    fn validate(&self) -> Result<(), GatewayError> {
        if self.server.bind_addr.trim().is_empty() {
            return Err(invalid("server.bind_addr must not be empty"));
        }
        if self.document.root.trim().is_empty() {
            return Err(invalid("document.root must not be empty"));
        }
        if self.document.entry_w4.trim().is_empty() {
            return Err(invalid("document.entry_w4 must not be empty"));
        }
        if self.security.jwt_secret.trim().is_empty() {
            return Err(invalid("security.jwt_secret must not be empty"));
        }
        if self.security.admin_token.trim().is_empty() {
            return Err(invalid("security.admin_token must not be empty"));
        }
        if self.rendering.template_loader.timeout_ms == 0 {
            return Err(invalid("rendering.template_loader.timeout_ms must be > 0"));
        }
        if self.rendering.template_loader.max_bytes == 0 {
            return Err(invalid("rendering.template_loader.max_bytes must be > 0"));
        }
        Ok(())
    }
}

fn invalid(message: &str) -> GatewayError {
    GatewayError(RuntimeError::new(
        ErrorCode::InvalidArgument,
        message,
        false,
    ))
}

const fn default_template_timeout_ms() -> u64 {
    3_000
}

const fn default_template_max_bytes() -> usize {
    256 * 1024
}
