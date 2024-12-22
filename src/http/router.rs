use crate::libr::LOGGER;

use super::{handlers::Handler, middleware::token_middleware};
use axum::{
    extract::{Extension, Multipart, Path},
    http::StatusCode,
    middleware,
    response::{IntoResponse, Response},
    routing::{get, post},
    Json, Router,
};
use std::{path::Path as StdPath, sync::Arc};
use tokio::fs;

pub struct AppRouter {
    pub router: Router,
}

impl AppRouter {
    pub fn new(handler: Handler) -> Self {
        let handler = Arc::new(handler);

        let router = Router::new()
            .route("/upload", post(upload_route))
            .route("/get_beat/:id", get(get_beat_file_route))
            .layer(Extension(handler))
            .layer(middleware::from_fn(token_middleware));
        LOGGER.info("Router initialized with routes");
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

async fn get_beat_file_route(
    Path(beat_id): Path<i64>,
    Extension(handler): Extension<Arc<Handler>>,
) -> impl IntoResponse {
    match handler.get_beat(beat_id).await {
        Ok(file_path) => match fs::read(&file_path).await {
            Ok(file_bytes) => {
                let file_name = StdPath::new(&file_path)
                    .file_name()
                    .and_then(|f| f.to_str())
                    .unwrap();

                Response::builder()
                    .status(StatusCode::OK)
                    .header("Content-Type", "application/zip")
                    .header(
                        "Content-Disposition",
                        format!("attachment; filename=\"{}\"", file_name),
                    )
                    .body(axum::body::Body::from(file_bytes))
                    .unwrap()
                    .into_response()
            }
            Err(_) => (StatusCode::INTERNAL_SERVER_ERROR, "Failed to read file").into_response(),
        },
        Err(_) => (StatusCode::NOT_FOUND, "Beat not found").into_response(),
    }
}
