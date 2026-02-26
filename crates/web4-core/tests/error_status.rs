use http::StatusCode;
use web4_core::{ErrorCode, RuntimeError};

#[test]
fn standard_code_maps_to_expected_status() {
    let status = ErrorCode::RateLimited.http_status();
    assert_eq!(status, StatusCode::TOO_MANY_REQUESTS);
}

#[test]
fn runtime_error_reports_status_from_code() {
    let err = RuntimeError::new(ErrorCode::ConsentRequired, "consent needed", false);
    assert_eq!(err.status_code(), StatusCode::FORBIDDEN);
}
