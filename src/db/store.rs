use crate::libr::Minio;
use anyhow::Error;

pub struct MinioStore {
    minio: Minio,
}

impl MinioStore {
    pub fn new(minio: Minio) -> Self {
        Self { minio }
    }

    pub async fn upload_file(&self, file_mp3: &str, key_mp3: &str, file_image: &str, key_image: &str) -> Result<(), Error> {
        let client = self.minio.connect().await?;
        let _ = &self.minio.create_bucket(&client).await?;
        let _ = &self.minio.upload(&client, file_mp3, key_mp3).await.unwrap();
        let _ = &self.minio.upload(&client, file_image, key_image).await.unwrap();
        Ok(())
    }
}
