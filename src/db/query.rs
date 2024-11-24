use crate::db::models::FileMetadata;
use crate::libr::{Postgres, LOGGER};
use anyhow::Error;
use sqlx::Row;

use super::models::BeatData;
pub struct Db {
    postgres: Postgres,
}

impl Db {
    pub fn new(postgres: Postgres) -> Self {
        Self { postgres }
    }

    pub async fn insert_file_metadata(&self, file: FileMetadata) -> Result<i64, Error> {
        let query = "insert into files_metadata (name, bitrate, duration, size, created, updated)
                            values ($1, $2, $3, $4, now(), now())
                            returning id";
        LOGGER.info(&format!("Trying to insert metadata of file {}", &file.name));
        match sqlx::query(&query)
            .bind(&file.name)
            .bind(&file.bitrate)
            .bind(&file.duration)
            .bind(&file.size)
            .fetch_one(&self.postgres.pool)
            .await
        {
            Ok(row) => {
                let id: i32 = row.get("id");
                LOGGER.info(&format!("Insertion file {} was succesfully", &file.name));
                Ok(id as i64)
            }
            Err(e) => {
                LOGGER.error(&format!(
                    "Insertion file's {} metadata was failed with error {:?}",
                    &file.name, e
                ));
                Err(e.into())
            }
        }
    }

    pub async fn insert_beat_data(&self, data: BeatData) -> Result<(), Error> {
        let cnt_queries = data.genres.len();
        for i in 0..cnt_queries {
            let query = "insert into beats (beat_id, name, description, beatmaker_id, genre)
                                values ($1, $2, $3, $4, $5)";
            match sqlx::query(&query)
                .bind(&data.beat_id)
                .bind(&data.name)
                .bind(&data.description)
                .bind(&data.beatmaker_id)
                .bind(&data.genres[i])
                .execute(&self.postgres.pool)
                .await
            {
                Ok(_) => {}
                Err(e) => {
                    LOGGER.error(&format!(
                        "Insertion beat's {} data was failed with error {:?}",
                        &data.name, e
                    ));
                    return Err(e.into());
                }
            }
        }
        Ok(())
    }

    pub async fn insert_image(&self, beat_id: i64, image: String) -> Result<(), Error> {
        let query = "insert into images (beat_id, image)
                            values ($1, $2)";
        match sqlx::query(&query)
            .bind(&(beat_id as i32))
            .bind(image)
            .execute(&self.postgres.pool)
            .await
        {
            Ok(_) => {}
            Err(e) => {
                LOGGER.error(&format!(
                    "Insertion beat's {} image was failed with error {:?}",
                    &beat_id, e
                ));
                return Err(e.into());
            }
        }
        Ok(())
    }
}
