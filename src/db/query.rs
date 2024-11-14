use anyhow::Error;

use crate::db::models::FileMetadata;
use crate::lib::{Postgres, LOGGER};
pub struct Db {
    postgres: Postgres,
}

impl Db {
    pub fn new(postgres: Postgres) -> Self {
        Self { postgres }
    }

    pub async fn insert(&self, file: FileMetadata) -> Result<(), Error> {
        let query =
            "insert into files_metadata (name, bitrate, duration, size, created, updated)
                            values ($1, $2, $3, $4, now(), now())";
        LOGGER.info(&format!("Trying to insert metadata of file {}", &file.name));
        match sqlx::query(&query)
            .bind(&file.name)
            .bind(&file.bitrate)
            .bind(&file.duration)
            .bind(&file.size)
            .execute(&self.postgres.pool)
            .await
        {
            Ok(_) => {
                LOGGER.info(&format!("Insertion file {} was succesfully", &file.name));
                Ok(())
            }
            Err(e) => {
                LOGGER.error(&format!("Insertion file's {} metadata was failed with error {:?}", &file.name, e));
                Err(e.into())
            }
        }
    }
}
