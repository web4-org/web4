use async_trait::async_trait;
use serde_json::Value;

use crate::error::Web4Error;
use crate::model::{BindingInvocation, ConsentRequest, RenderRequest, ValidationTarget};

pub trait Validator: Send + Sync {
    fn validate(&self, target: &ValidationTarget) -> Result<(), Web4Error>;
}

#[async_trait]
pub trait ConsentEngine: Send + Sync {
    async fn check(&self, request: &ConsentRequest) -> Result<(), Web4Error>;
}

#[async_trait]
pub trait BindingExecutor: Send + Sync {
    async fn execute(&self, invocation: &BindingInvocation) -> Result<Value, Web4Error>;
}

pub trait Renderer: Send + Sync {
    fn render_html(&self, request: &RenderRequest) -> Result<String, Web4Error>;
}
