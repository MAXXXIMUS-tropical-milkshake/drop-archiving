use super::format::{is_archive, is_image, is_mp3};
use crate::client::grpc::client::GrpcClient;
use crate::libr::LOGGER;
use crate::service::archiving::Service;
use crate::service::metadata::get_bitrate;
use axum::extract::Multipart;
use axum::{http::StatusCode, response::IntoResponse, Json};
use serde_json::Value;
use std::fs::{self, File};
use std::io::Write;
use std::path::Path;
use std::sync::Arc;
use tokio::sync::Mutex;



pub struct Handler {
    service: Arc<Service>,
    grpc_client: Arc<Mutex<GrpcClient>>,
}

impl Handler {
    pub fn new(service: Service, grpc_client: GrpcClient) -> Self {
        Self {
            service: Arc::new(service),
            grpc_client: Arc::new(Mutex::new(grpc_client)),
        }
    }

    pub async fn upload(&self, mut multipart: Multipart, user_id: i64) -> impl IntoResponse {
        LOGGER.info("Upload started");
        let upload_dir = Path::new("./uploads").to_path_buf();
        let mp3_dir = upload_dir.join("mp3");
        let archive_dir = upload_dir.join("archive");
        let image_dir = upload_dir.join("image");

        if let Err(e) = fs::create_dir_all(&mp3_dir) {
            LOGGER.error(&format!("Failed to create mp3 directory: {}", e));
            return (StatusCode::INTERNAL_SERVER_ERROR, Json("Failed to create mp3 directory"));
        }
        if let Err(e) = fs::create_dir_all(&archive_dir) {
            LOGGER.error(&format!("Failed to create archive directory: {}", e));
            return (StatusCode::INTERNAL_SERVER_ERROR, Json("Failed to create archive directory"));
        }
        if let Err(e) = fs::create_dir_all(&image_dir) {
            LOGGER.error(&format!("Failed to create image directory: {}", e));
            return (StatusCode::INTERNAL_SERVER_ERROR, Json("Failed to create image directory"));
        }

        let mut name = String::new();
        let mut beat_genre = Vec::new();
        let mut description = String::new();
        let mut file_path_mp3 = String::new();
        let mut file_path_image = String::new();
        let mut file_path_archive = String::new();
        let mut fname_mp3 = String::new();
        let mut fname_image = String::new();
        let mut fname_archive = String::new();

        while let Some(field) = multipart.next_field().await.unwrap_or_else(|e| {
            LOGGER.error(&format!("Error reading multipart field: {}", e));
            None
        }) {
            if let Some(_file_name) = field.file_name() {
                let fname = field.file_name().unwrap().to_string();
                let data = match field.bytes().await {
                    Ok(data) => data,
                    Err(e) => {
                        LOGGER.error(&format!("Error reading file bytes: {}", e));
                        return (StatusCode::BAD_REQUEST, Json("Failed to read file bytes"));
                    }
                };

                let dir = if is_mp3(&data) {
                    &mp3_dir
                } else if is_archive(&data) {
                    &archive_dir
                } else if is_image(&data) {
                    &image_dir
                } else {
                    &upload_dir
                };

                let file_path = dir.join(&fname);
                if let Err(e) = File::create(&file_path).and_then(|mut file| file.write_all(&data)) {
                    LOGGER.error(&format!("Error saving file {}: {}", fname, e));
                    return (StatusCode::INTERNAL_SERVER_ERROR, Json("Failed to save file"));
                }

                if is_mp3(&data) {
                    file_path_mp3 = file_path.to_str().unwrap_or_default().to_string();
                    fname_mp3 = fname;
                } else if is_image(&data) {
                    file_path_image = file_path.to_str().unwrap_or_default().to_string();
                    fname_image = fname;
                } else if is_archive(&data) {
                    file_path_archive = file_path.to_str().unwrap_or_default().to_string();
                    fname_archive = fname;
                }
            } else if field.name() == Some("metadata") {
                let data = match field.text().await {
                    Ok(data) => data,
                    Err(e) => {
                        LOGGER.error(&format!("Error reading metadata: {}", e));
                        return (StatusCode::BAD_REQUEST, Json("Failed to read metadata"));
                    }
                };

                match serde_json::from_str::<Value>(&data) {
                    Ok(metadata) => {
                        name = metadata.get("name").and_then(|v| v.as_str()).unwrap_or_default().to_string();
                        beat_genre = metadata
                            .get("beat_genre")
                            .and_then(|v| v.as_array())
                            .map(|arr| arr.iter().filter_map(|v| v.as_str().map(|s| s.to_string())).collect())
                            .unwrap_or_default();
                        description = metadata.get("description").and_then(|v| v.as_str()).unwrap_or_default().to_string();
                    }
                    Err(e) => {
                        LOGGER.error(&format!("Error parsing metadata: {}", e));
                        return (StatusCode::BAD_REQUEST, Json("Invalid metadata format"));
                    }
                }
            }
        }

        let mut bitrate = match get_bitrate(&file_path_mp3) {
            Ok(bitrate) => bitrate,
            Err(e) => {
                LOGGER.error(&format!("Failed to get bitrate: {}", e));
                return (StatusCode::INTERNAL_SERVER_ERROR, Json("Failed to get bitrate"));
            }
        };

        let mut beat_id = match self
            .service
            .insert_beat(
                &file_path_mp3,
                &fname_mp3,
                user_id,
                &name,
                &description,
                &beat_genre,
                &file_path_image,
                &fname_image,
                bitrate,
                &file_path_archive,
                &fname_archive,
            )
            .await
        {
            Ok(beat_id) => beat_id,
            Err(e) => {
                LOGGER.error(&format!("Failed to insert beat: {}", e));
                return (StatusCode::INTERNAL_SERVER_ERROR, Json("Failed to insert beat"));
            }
        };

        if bitrate > 200_000f64 {
            match self
                .service
                .reduce_bitrate(&file_path_mp3, &fname_mp3, 200)
            {
                Ok((reduced_bitrate, fname_reduced, path_reduced)) => {
                    file_path_mp3 = path_reduced;
                    fname_mp3 = fname_reduced;
                    bitrate = reduced_bitrate;
                }
                Err(e) => {
                    LOGGER.error(&format!("Failed to reduce bitrate: {}", e));
                    return (StatusCode::INTERNAL_SERVER_ERROR, Json("Failed to reduce bitrate"));
                }
            }

            beat_id = match self
                .service
                .insert_beat(
                    &file_path_mp3,
                    &fname_mp3,
                    user_id,
                    &name,
                    &description,
                    &beat_genre,
                    &file_path_image,
                    &fname_image,
                    bitrate,
                    &file_path_archive,
                    &fname_archive,
                )
                .await
            {
                Ok(beat_id) => beat_id,
                Err(e) => {
                    LOGGER.error(&format!("Failed to re-insert beat: {}", e));
                    return (StatusCode::INTERNAL_SERVER_ERROR, Json("Failed to re-insert beat"));
                }
            };
        }

        if let Err(e) = self.grpc_client.lock().await.upload_beat(
            beat_id,
            user_id,
            &name,
            &description,
            beat_genre,
            &file_path_mp3,
            &file_path_image,
        ).await
        {
            LOGGER.error(&format!("Failed to upload beat via GRPC: {}", e));
            return (StatusCode::INTERNAL_SERVER_ERROR, Json("Failed to upload beat"));
        }

        (StatusCode::OK, Json("File uploaded successfully"))
    }

    pub async fn get_beat(&self, beat_id: i64) -> Result<String, anyhow::Error> {
        LOGGER.info("start get_beat");
        match self.service.get_beat_by_id(beat_id).await {
            Ok(file_path) => Ok(file_path),
            Err(e) => {
                LOGGER.error(&format!("Failed to get beat: {}", e));
                Err(anyhow::anyhow!("Failed to get beat"))
            }
        }
    }
}