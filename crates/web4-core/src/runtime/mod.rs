mod binding;
mod consent;
mod error;
mod schema;
mod validation;

pub use binding::{GatewayBindingExecutor, HttpBindingExecutor, LocalBindingExecutor, McpInvoker};
pub use consent::DefaultConsentEngine;
pub use schema::w4_type_to_json_schema;
pub use validation::JsonSchemaValidator;

use crate::error::{ErrorCode, RuntimeError, Web4Error};
use crate::model::{BindingInvocation, ConsentRequest, ValidationTarget};
use crate::runtime::error::internal;
use crate::traits::{BindingExecutor, ConsentEngine, Validator};
use serde_json::Value;

#[derive(Debug, Clone, Default)]
pub struct InvocationContext {
    pub capability_token: Option<String>,
    pub interactive_approved: bool,
}

pub struct ServiceRuntime<V, C, B>
where
    V: Validator,
    C: ConsentEngine,
    B: BindingExecutor,
{
    pub validator: V,
    pub consent_engine: C,
    pub binding_executor: B,
}

impl<V, C, B> ServiceRuntime<V, C, B>
where
    V: Validator,
    C: ConsentEngine,
    B: BindingExecutor,
{
    pub async fn invoke(
        &self,
        ir: &Value,
        service_id: &str,
        input: Value,
        ctx: InvocationContext,
    ) -> Result<Value, Web4Error> {
        let service = find_service(ir, service_id)?;

        if let Some(input_type) = service.get("input") {
            let schema = w4_type_to_json_schema(input_type);
            self.validator.validate(&ValidationTarget::JsonSchema {
                schema,
                payload: input.clone(),
            })?;
        }

        let effects = service
            .get("effects")
            .and_then(Value::as_str)
            .unwrap_or("unknown")
            .to_string();
        let mode = service
            .get("consent")
            .and_then(|v| v.get("mode"))
            .and_then(Value::as_str)
            .unwrap_or(if effects == "none" {
                "open"
            } else {
                "capability"
            })
            .to_string();

        self.consent_engine
            .check(&ConsentRequest {
                service_id: service_id.to_string(),
                effects,
                mode,
                capability_token: ctx.capability_token,
                interactive_approved: ctx.interactive_approved,
            })
            .await?;

        let binding = service
            .get("bindings")
            .and_then(Value::as_array)
            .and_then(|arr| arr.first())
            .cloned()
            .ok_or_else(|| internal("service binding not declared"))?;

        let output = self
            .binding_executor
            .execute(&BindingInvocation {
                service_id: service_id.to_string(),
                binding,
                input,
            })
            .await?;

        if let Some(output_type) = service.get("output") {
            let schema = w4_type_to_json_schema(output_type);
            self.validator.validate(&ValidationTarget::JsonSchema {
                schema,
                payload: output.clone(),
            })?;
        }

        Ok(output)
    }
}

fn find_service<'a>(ir: &'a Value, service_id: &str) -> Result<&'a Value, Web4Error> {
    ir.get("services")
        .and_then(Value::as_array)
        .and_then(|arr| {
            arr.iter()
                .find(|service| service.get("id").and_then(Value::as_str) == Some(service_id))
        })
        .ok_or_else(|| {
            Web4Error::Runtime(RuntimeError::new(
                ErrorCode::NotFound,
                format!("service not found: {service_id}"),
                false,
            ))
        })
}
