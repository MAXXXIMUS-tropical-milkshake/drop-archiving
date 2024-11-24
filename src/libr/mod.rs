pub mod logger;
pub mod postgres;
pub mod minio;
pub use self::postgres::postgres::Postgres;
pub use self::logger::logger::LOGGER;
pub use self::minio::minio::Minio;

