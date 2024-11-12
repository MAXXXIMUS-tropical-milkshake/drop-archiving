use crate::http::handlers::upload;
use axum::{routing::post, Router};

pub fn create_router() -> Router {
    Router::new().route("/upload", post(upload))
}
