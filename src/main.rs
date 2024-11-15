use db::store::MinioStore;
use http::{handlers::Handler, router::AppRouter};
use lib::Minio;
use service::archiving::Service;
use tokio;
// mod audio {
//     tonic::include_proto!("audio");
// }
use axum::Server;
use std::net::SocketAddr;
mod http;
mod service;
mod db;
mod lib;
use axum::extract::DefaultBodyLimit;
use crate::db::query::Db;
use crate::lib::Postgres;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .init();
    let p = Postgres::new("postgres://postgres:postgres@localhost:5432/files_metadata").await?;
    let m = Minio::new("minioadmin", "minioadmin", "us-east-1", "http://127.0.0.1:9000", "drop-test");
    let ms = MinioStore::new(m);
    let db = Db::new(p);
    let service = Service::new(db, ms);
    let handler = Handler::new(service);
    let router = AppRouter::new(handler);
    let app = router.router.layer(DefaultBodyLimit::max(500 * 1024 * 1024));
    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));
    println!("Server running on http://{}", addr);
    Server::bind(&addr)
        .serve(app.into_make_service())
        .await
        .unwrap();
    Ok(())
}
