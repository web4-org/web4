use crate::runtime::{
    DefaultConsentEngine, GatewayBindingExecutor, HttpBindingExecutor, JsonSchemaValidator,
    LocalBindingExecutor, ServiceRuntime,
};

pub type DefaultServiceRuntime =
    ServiceRuntime<JsonSchemaValidator, DefaultConsentEngine, GatewayBindingExecutor>;

#[derive(Debug, Clone)]
pub struct RuntimeOptions {
    pub http_base_url: Option<String>,
    pub local_working_dir: Option<String>,
}

impl Default for RuntimeOptions {
    fn default() -> Self {
        Self {
            http_base_url: None,
            local_working_dir: None,
        }
    }
}

pub fn build_default_runtime(options: RuntimeOptions) -> DefaultServiceRuntime {
    ServiceRuntime {
        validator: JsonSchemaValidator,
        consent_engine: DefaultConsentEngine,
        binding_executor: GatewayBindingExecutor {
            http: HttpBindingExecutor {
                base_url: options.http_base_url,
            },
            local: LocalBindingExecutor {
                working_dir: options.local_working_dir,
                mcp: None,
            },
        },
    }
}
