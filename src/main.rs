#[path = "../lib/logger/logger.rs"]
mod logger;

#[path = "../lib/postgres/postgres.rs"]
mod postgres;

use std::env;
use std::time::Instant;
use std::time::Duration;
use tokio_postgres::{Config, NoTls};
use tracing::{info, error};
use postgres::Postgres;
use logger::LOGGER;

#[tokio::main]
async fn main() {
    
}


