pub struct Config {
    pub postgres_url: &'static str,
    pub minio_url: &'static str,
    pub minio_user: &'static str,
    pub minio_password: &'static str,
    pub minio_region: &'static str,
    pub grpc_url: &'static str,
    pub bucket_name: &'static str,
    pub body_limit: usize,
}

impl Config {
    pub fn new() -> Self {
        Self {
            postgres_url: "postgres://postgres:postgres@localhost:5432/files_metadata",
            minio_url: "http://127.0.0.1:9002",
            minio_user: "minioadmin",
            minio_password: "minioadmin",
            minio_region: "us-east-1",
            grpc_url: "http://127.0.0.1:50052",
            bucket_name: "drop-test",
            body_limit: 5000 * 1024 * 1024,
        }
    }
}
