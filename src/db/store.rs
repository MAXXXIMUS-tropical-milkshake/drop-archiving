use std::env;

use crate::libr::Minio;
use crate::libr::LOGGER;
use anyhow::Error;
pub struct MinioStore {
    minio: Minio,
}
use std::fs::File;
use std::path::Path;
use std::io::Write;
impl MinioStore {
    pub fn new(minio: Minio) -> Self {
        Self { minio }
    }

    pub async fn upload_file(
        &self,
        file_mp3: &str,
        key_mp3: &str,
        file_image: &str,
        key_image: &str,
        file_archive: &str,
        key_archive: &str,
    ) -> Result<(), Error> {
        let client = self.minio.connect().await?;
        let _ = &self.minio.create_bucket(&client).await?;
        let _ = &self.minio.upload(&client, file_mp3, key_mp3).await.unwrap();
        let _ = &self
            .minio
            .upload(&client, file_image, key_image)
            .await
            .unwrap();
        let _ = &self
            .minio
            .upload(&client, file_archive, key_archive)
            .await
            .unwrap();
        Ok(())
    }
    pub async fn get_file(&self, link: String) -> Result<String, Error> {
        let client = self.minio.connect().await?;
        let object = client
            .get_object()
            .bucket(env::var("MINIO_BUCKET").unwrap())
            .key(&link)
            .send()
            .await
            .map_err(|e| anyhow::anyhow!("Failed to get object: {}", e))
            .unwrap();
        let tmp_file_path = format!("./temp/{}", link);
        let tmp_dir = Path::new("./temp");
        if !tmp_dir.exists() {
            std::fs::create_dir_all(tmp_dir)?;
        }

        let mut file = File::create(&tmp_file_path)?;
        let body = object
            .body
            .collect()
            .await
            .map_err(|e| anyhow::anyhow!("Failed to read object body: {}", e))?;
        file.write_all(&body.into_bytes())?;

        LOGGER.info(&format!(
            "File successfully downloaded to: {}",
            tmp_file_path
        ));
        Ok(tmp_file_path)
    }
}
