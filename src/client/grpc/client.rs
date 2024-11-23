use std::fs;

use audio::audio_service_client::AudioServiceClient;
use audio::{UploadRequest, UploadResponse};
use chrono::format::parse;
use reqwest::Client;
use tonic::transport::Channel;
use url::Url;

use crate::lib::LOGGER;
mod audio {
    tonic::include_proto!("audio");
}

pub struct GrpcClient {
    client: AudioServiceClient<Channel>,
}

impl GrpcClient {
    pub async fn connect(endpoint: &str) -> Result<Self, Box<dyn std::error::Error>> {
        let client = AudioServiceClient::connect(endpoint.to_string()).await?;
        Ok(Self { client })
    }

    pub async fn upload_beat(
        &mut self,
        beat_id: i64,
        beatmaker_id: i64,
        name: &str,
        description: &str,
        beat_genre: Vec<String>,
        file_path_mp3: &str,
        file_path_image: &str,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let request = UploadRequest {
            beat_id,
            beatmaker_id,
            name: name.to_string(),
            description: description.to_string(),
            beat_genre,
        };
        let response = self.client.upload(request).await?.into_inner();
        let mut parsed_file_upload_url = Url::parse(&response.file_upload_url).unwrap();
        let mut parsed_image_upload_url = Url::parse(&response.image_upload_url).unwrap();
        parsed_file_upload_url.set_host(Some("localhost")).unwrap();
        parsed_image_upload_url.set_host(Some("localhost")).unwrap();
        LOGGER.info(&format!(
            "URL {} was received successfully",
            parsed_file_upload_url
        ));
        LOGGER.info(&format!(
            "URL {} was received successfully",
            parsed_image_upload_url
        ));
        let file_content_mp3 = fs::read(file_path_mp3).unwrap();
        let file_content_image = fs::read(file_path_image).unwrap();
        let client = Client::new();
        client
            .put(parsed_file_upload_url)
            .body(file_content_mp3)
            .send()
            .await?
            .error_for_status()?;
        LOGGER.info(&format!("File {} was sent successfully", &file_path_mp3));
        client
            .put(parsed_image_upload_url)
            .body(file_content_image)
            .send()
            .await?
            .error_for_status()?;
        LOGGER.info(&format!("Image {} was sent successfully", &file_path_image));
        Ok(())
    }
}
