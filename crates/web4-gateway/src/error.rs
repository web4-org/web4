use axum::{response::IntoResponse, Json};
use web4_core::{ErrorEnvelope, RuntimeError};

#[derive(Debug)]
pub struct GatewayError(pub RuntimeError);

impl IntoResponse for GatewayError {
    fn into_response(self) -> axum::response::Response {
        let status = self.0.status_code();
        let envelope = ErrorEnvelope { error: self.0 };
        (status, Json(envelope)).into_response()
    }
}
