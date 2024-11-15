//use crate::http::handlers::upload;
use super::handlers::Handler;
use axum::{routing::post, Router};
use std::sync::Arc;
pub struct AppRouter {
    pub router: Router,
}
impl AppRouter {
    pub fn new(handler: Handler) -> Self {
        let handler = Arc::new(handler);
        let router = Router::new().route(
            "/upload",
            post({
                let handler = Arc::clone(&handler);
                move |multipart| async move { handler.upload(multipart).await }
            }),
        );
        Self { router }
    }
}