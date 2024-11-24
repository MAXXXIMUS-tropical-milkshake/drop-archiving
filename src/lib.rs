use axum::Server;
use client::grpc::client::GrpcClient;
use config::Config;
use db::store::MinioStore;
use http::{handlers::Handler, router::AppRouter};
use libr::{Minio, LOGGER};
use service::archiving::Service;
use std::net::SocketAddr;
mod client;
mod config;
mod db;
mod http;
mod libr;
mod service;

use crate::db::query::Db;
use crate::libr::Postgres;
use axum::extract::DefaultBodyLimit;

pub async fn run() -> Result<(), Box<dyn std::error::Error>> {
    let config = Config::new();
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .init();
    let postgres = Postgres::new(&config.postgres_url).await?;
    let minio = Minio::new(
        &config.minio_user,
        &config.minio_password,
        &config.minio_region,
        &config.minio_url,
        &config.bucket_name,
    );
    let minio_store = MinioStore::new(minio);
    let db = Db::new(postgres);
    let grpc = GrpcClient::connect(&config.grpc_url).await?;
    let service = Service::new(db, minio_store);
    let handler = Handler::new(service, grpc);
    let router = AppRouter::new(handler);
    let app = router
        .router
        .layer(DefaultBodyLimit::max(config.body_limit));
    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));
    LOGGER.info(&format!("Server running on http://{}", addr));
    Server::bind(&addr)
        .serve(app.into_make_service())
        .await
        .unwrap();
    Ok(())
}
