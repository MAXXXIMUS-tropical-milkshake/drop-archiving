use dotenv::dotenv;
use std::env;

pub struct Config {
    pub postgres_url: String,
    pub minio_url: String,
    pub minio_user: String,
    pub minio_password: String,
    pub minio_region: String,
    pub grpc_url: String,
    pub bucket_name: String,
    pub body_limit: usize,
}

impl Config {
    pub fn new() -> Self {
        dotenv().ok();

        Self {
            postgres_url: env::var("DATABASE_URL").expect("DATABASE_URL not set"),
            minio_url: env::var("MINIO_URL").expect("MINIO_URL not set"),
            minio_user: env::var("MINIO_USER").expect("MINIO_USER not set"),
            minio_password: env::var("MINIO_PASSWORD").expect("MINIO_PASSWORD not set"),
            minio_region: env::var("MINIO_REGION").unwrap_or_else(|_| "us-east-1".to_string()),
            grpc_url: env::var("GRPC_URL").expect("GRPC_URL not set"),
            bucket_name: env::var("MINIO_BUCKET").expect("MINIO_BUCKET not set"),
            body_limit: 5000 * 1024 * 1024,
        }
    }
}
