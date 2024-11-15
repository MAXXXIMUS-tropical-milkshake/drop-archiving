use crate::lib::Minio;
use anyhow::Error;

pub struct MinioStore {
    minio: Minio,
}

impl MinioStore {
    pub fn new(minio: Minio) -> Self {
        Self { minio }
    }

    pub async fn upload_file(&self, file: &str, key: &str) -> Result<(), Error> {
        let client = self.minio.connect().await?;
        let _ = &self.minio.create_bucket(&client).await?;
        let _ = &self.minio.upload(&client, file, key).await?;
        Ok(())
    }
}
