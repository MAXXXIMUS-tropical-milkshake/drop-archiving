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
                move |headers, multipart| {
                    let handler = Arc::clone(&handler);
                    async move { handler.upload(multipart, headers).await }
                }
            }),
        );
        Self { router }
    }
}
