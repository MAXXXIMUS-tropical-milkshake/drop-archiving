use once_cell::sync::Lazy;
use sqlx::{Error, PgPool};
use sqlx::postgres::PgPoolOptions;
use std::io;
use std::time::{self, Duration, Instant};
use tokio::time::sleep;
use tokio_postgres::{Client, Config, NoTls};
use tracing::{debug, error, info};
// use anyhow::Error;
use tokio_postgres::Row;

// use crate::logger::LOGGER;
use crate::{db, libr::LOGGER};

const DEFAULT_MAX_POOL_SIZE: u32 = 10;
const DEFAULT_CONN_ATTEMPTS: u32 = 10;
const DEFAULT_CONN_TIMEOUT: Duration = Duration::from_secs(1);

pub struct Postgres {
    pub pool: PgPool,
}

impl Postgres {
    pub async fn new(db_url: &str) -> Result<Self, Error> {
        let mut attempts_left = DEFAULT_CONN_ATTEMPTS;

        while attempts_left > 0 {
            match PgPoolOptions::new()
                .max_connections(DEFAULT_MAX_POOL_SIZE) // Максимальный размер пула
                .acquire_timeout(std::time::Duration::from_secs(300))
                .connect(db_url)
                .await
            {
                Ok(pool) => {
                    LOGGER.info("Successfully connected to the database, returning Ok");
                    return Ok(Self { pool });
                }
                Err(e) => {
                    attempts_left -= 1;
                    LOGGER.error(&format!(
                        "Connection to database was failed, attemts left: {}",
                        attempts_left
                    ));
                    if attempts_left > 0 {
                        LOGGER.debug(&format!(
                            "Retrying to connect to database in {:?}",
                            DEFAULT_CONN_TIMEOUT
                        ));
                        sleep(DEFAULT_CONN_TIMEOUT).await;
                    } else {
                        LOGGER.error(&format!(
                            "Failed to connect to database after {} attempts",
                            DEFAULT_CONN_ATTEMPTS
                        ));
                        return Err(e);
                    }
                }
            }
        }
        Err(Error::Protocol("Failed to connect to database".into()))
    }

    //     pub async fn connect(&mut self, db_url: &str) -> Result<(), Error> {
    //         let mut attempts_left = self.conn_attempts;
    //         let config: Config = db_url.parse().expect("Failed to parse db_url");
    //         while attempts_left > 0 {
    //             let start = Instant::now();
    //             LOGGER.info(&format!("Attempt to connect to database, attempts left: {}", attempts_left));
    //             match config.connect(NoTls).await {
    //                 Ok((client, connection)) => {
    //                     tokio::spawn(async move {
    //                         if let Err(e) = connection.await {
    //                             LOGGER.error(&format!("Connection to database error: {}", e));
    //                         }
    //                     });

    //                     let elapsed = start.elapsed();
    //                     LOGGER.info(&format!("Connection to database was successfully in {:?}", elapsed));
    //                     self.client = Some(client);
    //                     LOGGER.info("Successfully connected to the database, returning Ok");
    //                     return Ok(());
    //                 }

    //                 Err(err) => {
    //                     let elapsed = start.elapsed();
    //                     LOGGER.error(&format!("Connection to database was failed, elapsed: {:?}, {}", elapsed, err));
    //                     LOGGER.debug(&format!("Retrying to connect to database in {:?}", self.conn_timeout));
    //                     attempts_left -= 1;
    //                     sleep(self.conn_timeout).await;
    //                 }
    //             }
    //         }
    //         LOGGER.error(&format!("Failed to connect to database after {} attempts", self.conn_attempts));
    //         Err(anyhow::anyhow!("Failed to connect to database"))
    //     }

    //     pub async fn execute_query(&self, query: &str) -> Result<Vec<Row>, Error> {
    //         if let Some(client) = &self.client {
    //             LOGGER.info(&format!("Executing query: {}", query));
    //             let rows = client.query(query, &[]).await?;
    //             LOGGER.info("Query executed successfully.");
    //             Ok(rows)
    //         } else {
    //             Err(anyhow::anyhow!("Not connected to the database"))
    //         }
    //     }
}

// impl Default for Postgres {
//     fn default() -> Self {
//         Self {
//             max_pool_size: DEFAULT_MAX_POOL_SIZE,
//             conn_attempts: DEFAULT_CONN_ATTEMPTS,
//             conn_timeout: DEFAULT_CONN_TIMEOUT,
//             client: None,
//         }
//     }
// }
