use axum::http::StatusCode;

/// Currently a no-op that returns a `204 NO CONTENT` response. If Cactuar gets
/// to the point where this can be served, the controller task should already be
/// running and the service is ready to handle requests.
pub async fn readiness_handler() -> axum::http::StatusCode {
    StatusCode::NO_CONTENT
}
