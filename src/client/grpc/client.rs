use std::fs;

use audio::audio_service_client::AudioServiceClient;
use audio::{UploadRequest, UploadResponse};
use reqwest::Client;
use tonic::transport::Channel;

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
        beat_artist: &str,
        beat_genre: &str,
        file_path: &str,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let request = UploadRequest {
            beat_id,
            beatmaker_id,
            beat_artist: beat_artist.to_string(),
            beat_genre: beat_genre.to_string(),
        };
        let response = self.client.upload(request).await?.into_inner();
        LOGGER.info(&format!("URL {} was received successfully", &response.beat_upload_url));

        let file_content = fs::read(file_path)?;
        let client = Client::new();
        client
            .put(&response.beat_upload_url)
            .body(file_content)
            .send()
            .await?
            .error_for_status()?;
        LOGGER.info(&format!("File {} was sent successfully", &file_path));
        Ok(())
    }
}
