pub mod api;
pub mod compiler;
pub mod error;
pub mod model;
pub mod runtime;
pub mod traits;
pub mod w4ml;

pub use api::{build_default_runtime, DefaultServiceRuntime, RuntimeOptions};
pub use compiler::{compile_to_w4_json, ToolCompileInput};
pub use error::{ErrorCode, ErrorEnvelope, RuntimeError, Web4Error};
pub use model::{
    BindingInvocation, ConsentRequest, ModelIndex, ParseResult, RenderRequest, ValidationTarget,
};
pub use runtime::{
    w4_type_to_json_schema, DefaultConsentEngine, GatewayBindingExecutor, HttpBindingExecutor,
    InvocationContext, JsonSchemaValidator, LocalBindingExecutor, McpInvoker, ServiceRuntime,
};
pub use traits::{BindingExecutor, ConsentEngine, Renderer, Validator};
pub use w4ml::{W4mlParser, W4mlValidator};
