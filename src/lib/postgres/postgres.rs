use std::time::{self, Duration, Instant};
use tokio_postgres::{Client, Config, NoTls};
use tokio::time::sleep;
use once_cell::sync::Lazy;
use tracing::{info, error, debug};
use std::io;
use anyhow::Error;
use tokio_postgres::Row;

use crate::logger::LOGGER;

const DEFAULT_MAX_POOL_SIZE: u32 = 10;
const DEFAULT_CONN_ATTEMPTS: u32 = 10;
const DEFAULT_CONN_TIMEOUT: Duration = Duration::from_secs(1);

pub struct Postgres {
    max_pool_size: u32 ,
    conn_attempts: u32,
    conn_timeout: Duration,
    client: Option<Client>,

}

impl Postgres {
    pub fn new(max_pool_size: u32, conn_attempts: u32, conn_timeout: Duration) -> Self {
        Self {
            max_pool_size,
            conn_attempts,
            conn_timeout,
            client: None
        }
    }

    pub async fn connect(&mut self, db_url: &str) -> Result<(), Error> {
        let mut attempts_left = self.conn_attempts;
        let config: Config = db_url.parse().expect("Failed to parse db_url");
        while attempts_left > 0 {
            let start = Instant::now();
            LOGGER.info(&format!("Attempt to connect to database, attempts left: {}", attempts_left));
            match config.connect(NoTls).await {
                Ok((client, connection)) => {
                    tokio::spawn(async move {
                        if let Err(e) = connection.await {
                            LOGGER.error(&format!("Connection to database error: {}", e));
                        }
                    });
                            
                    let elapsed = start.elapsed();
                    LOGGER.info(&format!("Connection to database was successfully in {:?}", elapsed));
                    self.client = Some(client);
                    LOGGER.info("Successfully connected to the database, returning Ok");
                    return Ok(());
                }

                Err(err) => {
                    let elapsed = start.elapsed();
                    LOGGER.error(&format!("Connection to database was failed, elapsed: {:?}, {}", elapsed, err));
                    LOGGER.debug(&format!("Retrying to connect to database in {:?}", self.conn_timeout));
                    attempts_left -= 1;
                    sleep(self.conn_timeout).await;
                }
            }   
        }
        LOGGER.error(&format!("Failed to connect to database after {} attempts", self.conn_attempts));
        Err(anyhow::anyhow!("Failed to connect to database"))
    }
    

    pub async fn execute_query(&self, query: &str) -> Result<Vec<Row>, Error> {
        if let Some(client) = &self.client {
            LOGGER.info(&format!("Executing query: {}", query));
            let rows = client.query(query, &[]).await?;
            LOGGER.info("Query executed successfully.");
            Ok(rows)
        } else {
            Err(anyhow::anyhow!("Not connected to the database"))
        }
    }
    
}




impl Default for Postgres {
    fn default() -> Self {
        Self {
            max_pool_size: DEFAULT_MAX_POOL_SIZE,
            conn_attempts: DEFAULT_CONN_ATTEMPTS,
            conn_timeout: DEFAULT_CONN_TIMEOUT,
            client: None,
        }
    }
}