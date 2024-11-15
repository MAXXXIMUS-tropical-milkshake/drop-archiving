use std::fs;

use aws_sdk_s3::config::Builder;
use aws_sdk_s3::primitives::ByteStream;
use aws_sdk_s3::Client;
use aws_sdk_s3::{config::Credentials, Error};
use aws_types::region::{self, Region};

use crate::lib::LOGGER;

pub struct Minio {
    user: String,
    password: String,
    region: String,
    endpoint: String,
    bucket: String,
}

impl Minio {
    pub fn new(user: &str, password: &str, region: &str, endpoint: &str, bucket: &str) -> Self {
        Self {
            user: user.to_string(),
            password: password.to_string(),
            region: region.to_string(),
            endpoint: endpoint.to_string(),
            bucket: bucket.to_string(),
        }
    }

    pub async fn connect(&self) -> Result<Client, aws_sdk_s3::Error> {
        let credentials = Credentials::new(&self.user, &self.password, None, None, "static");
        let config = Builder::new()
            .region(Region::new(self.region.clone()))
            .endpoint_url(&self.endpoint)
            .credentials_provider(credentials)
            .behavior_version_latest()
            .build();
        let client = Client::from_conf(config);
        LOGGER.info("Connection to minio was successfull");
        Ok(client)
    }

    pub async fn create_bucket(&self, client: &Client) -> Result<(), Error> {
        match client.head_bucket().bucket(&self.bucket).send().await {
            Ok(_) => {
            }
            Err(_) => {
                client.create_bucket().bucket(&self.bucket).send().await?;
                LOGGER.info(&format!("Bucket {} creating was successfull", &self.bucket));
            }
        }
        Ok(())
    }

    pub async fn upload(&self, client: &Client, file_path: &str, key: &str) -> Result<(), Error> {
        let file_content = fs::read(file_path).unwrap();
        client
            .put_object()
            .bucket(&self.bucket)
            .key(key)
            .body(file_content.into())
            .send()
            .await?;
            LOGGER.info(&format!("Adding file {} to minio bucket {} was successfull", file_path, &self.bucket));

        Ok(())
    }
}
