use crate::error::{ErrorCode, RuntimeError, Web4Error};
use crate::model::BindingInvocation;
use crate::runtime::error::{internal, invalid};
use crate::traits::BindingExecutor;
use async_trait::async_trait;
use reqwest::Method;
use serde_json::Value;
use std::process::Stdio;
use tokio::io::AsyncWriteExt;
use tokio::process::Command;

#[derive(Default)]
pub struct GatewayBindingExecutor {
    pub http: HttpBindingExecutor,
    pub local: LocalBindingExecutor,
}

#[async_trait]
impl BindingExecutor for GatewayBindingExecutor {
    async fn execute(&self, invocation: &BindingInvocation) -> Result<Value, Web4Error> {
        let binding_type = invocation
            .binding
            .get("type")
            .and_then(Value::as_str)
            .ok_or_else(|| invalid("binding.type is required"))?;

        match binding_type {
            "http" => self.http.execute(invocation).await,
            "local" => self.local.execute(invocation).await,
            _ => Err(invalid(format!("unsupported binding type: {binding_type}"))),
        }
    }
}

#[derive(Default)]
pub struct HttpBindingExecutor {
    pub base_url: Option<String>,
}

#[async_trait]
impl BindingExecutor for HttpBindingExecutor {
    async fn execute(&self, invocation: &BindingInvocation) -> Result<Value, Web4Error> {
        let method = invocation
            .binding
            .get("method")
            .and_then(Value::as_str)
            .unwrap_or("POST");
        let endpoint = invocation
            .binding
            .get("endpoint")
            .and_then(Value::as_str)
            .ok_or_else(|| invalid("http binding endpoint is required"))?;
        let content_type = invocation
            .binding
            .get("contentType")
            .and_then(Value::as_str)
            .unwrap_or("application/json");

        if content_type != "application/json" {
            return Err(invalid("http binding contentType must be application/json"));
        }

        let url = if endpoint.starts_with("http://") || endpoint.starts_with("https://") {
            endpoint.to_string()
        } else {
            let base = self
                .base_url
                .as_deref()
                .ok_or_else(|| invalid("relative endpoint requires runtime base_url"))?;
            format!("{}{}", base.trim_end_matches('/'), endpoint)
        };

        let method = Method::from_bytes(method.as_bytes())
            .map_err(|_| invalid(format!("invalid http method: {method}")))?;

        let client = reqwest::Client::new();
        let response = client
            .request(method, &url)
            .json(&invocation.input)
            .send()
            .await
            .map_err(|err| internal(format!("http binding request failed: {err}")))?;

        let status = response.status();
        let body: Value = response
            .json()
            .await
            .map_err(|err| internal(format!("http binding response json error: {err}")))?;

        if !status.is_success() {
            if let Some(err) = body.get("error") {
                let code = err
                    .get("code")
                    .and_then(Value::as_str)
                    .unwrap_or("INTERNAL_ERROR")
                    .to_string();
                let message = err
                    .get("message")
                    .and_then(Value::as_str)
                    .unwrap_or("binding failed")
                    .to_string();
                let retryable = err
                    .get("retryable")
                    .and_then(Value::as_bool)
                    .unwrap_or(false);
                return Err(Web4Error::Runtime(RuntimeError {
                    code,
                    message,
                    retryable,
                    details: err.get("details").cloned(),
                }));
            }
            return Err(Web4Error::Runtime(RuntimeError::new(
                ErrorCode::InternalError,
                format!("http binding returned status {status}"),
                false,
            )));
        }

        Ok(body)
    }
}

#[async_trait]
pub trait McpInvoker: Send + Sync {
    async fn call_tool(&self, server: &str, tool: &str, args: Value) -> Result<Value, Web4Error>;
}

#[derive(Default)]
pub struct LocalBindingExecutor {
    pub working_dir: Option<String>,
    pub mcp: Option<std::sync::Arc<dyn McpInvoker>>,
}

#[async_trait]
impl BindingExecutor for LocalBindingExecutor {
    async fn execute(&self, invocation: &BindingInvocation) -> Result<Value, Web4Error> {
        let exec = invocation
            .binding
            .get("exec")
            .and_then(Value::as_str)
            .ok_or_else(|| invalid("local binding exec is required"))?;

        if let Some(command_line) = exec.strip_prefix("bin:") {
            return run_bin(command_line, self.working_dir.as_deref(), &invocation.input).await;
        }
        if let Some(target) = exec.strip_prefix("mcp:") {
            return run_mcp(self, target, &invocation.input).await;
        }

        Err(invalid("unsupported local exec runtime"))
    }
}

async fn run_bin(
    command_line: &str,
    working_dir: Option<&str>,
    input: &Value,
) -> Result<Value, Web4Error> {
    let argv = shell_words::split(command_line)
        .map_err(|err| invalid(format!("bin exec parse failed: {err}")))?;
    if argv.is_empty() {
        return Err(invalid("bin exec command is required"));
    }

    let mut command = Command::new(&argv[0]);
    command
        .args(&argv[1..])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());
    if let Some(path) = working_dir {
        command.current_dir(path);
    }

    let mut child = command
        .spawn()
        .map_err(|err| internal(format!("spawn command failed: {err}")))?;

    if let Some(mut stdin) = child.stdin.take() {
        let bytes = serde_json::to_vec(input).map_err(|err| internal(err.to_string()))?;
        stdin
            .write_all(&bytes)
            .await
            .map_err(|err| internal(format!("write stdin failed: {err}")))?;
    }

    let output = child
        .wait_with_output()
        .await
        .map_err(|err| internal(format!("wait process failed: {err}")))?;

    if !output.status.success() {
        return Err(Web4Error::Runtime(RuntimeError::new(
            ErrorCode::InternalError,
            format!(
                "local command exited with status {}",
                output.status.code().unwrap_or(-1)
            ),
            false,
        )));
    }

    let parsed = serde_json::from_slice::<Value>(&output.stdout)
        .map_err(|err| invalid(format!("local command stdout is not valid json: {err}")))?;
    Ok(parsed)
}

async fn run_mcp(
    exec: &LocalBindingExecutor,
    target: &str,
    input: &Value,
) -> Result<Value, Web4Error> {
    let (server, tool) = target
        .split_once('/')
        .ok_or_else(|| invalid("mcp exec must be mcp:<server>/<tool>"))?;
    let invoker = exec
        .mcp
        .as_ref()
        .ok_or_else(|| internal("mcp invoker not configured"))?;
    invoker.call_tool(server, tool, input.clone()).await
}
