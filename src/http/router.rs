use super::{handlers::Handler, middleware::token_middleware};
use axum::{
    extract::{Extension, Multipart},
    middleware,
    response::IntoResponse,
    routing::post,
    Router,
};
use std::sync::Arc;

pub struct AppRouter {
    pub router: Router,
}

impl AppRouter {
    pub fn new(handler: Handler) -> Self {
        let handler = Arc::new(handler);

        let router = Router::new()
            .route("/upload", post(upload_route))
            .layer(Extension(handler))
            .layer(middleware::from_fn(token_middleware));

        Self { router }
    }
}

async fn upload_route(
    Extension(handler): Extension<Arc<Handler>>,
    Extension(user_id): Extension<i64>,
    multipart: Multipart,
) -> impl IntoResponse {
    handler.upload(multipart, user_id).await
}
