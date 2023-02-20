use hyper::StatusCode;

pub async fn readiness_handler() -> axum::http::StatusCode {
    StatusCode::NO_CONTENT
}
