use super::archiving::get_archive_files;
use super::format::{is_archive, is_mp3};
use crate::client::grpc::client::GrpcClient;
use crate::lib::LOGGER;
use crate::service::archiving::Service;
use axum::extract::Multipart;
use axum::Extension;
use axum::{http::StatusCode, response::IntoResponse, Json};
use ffmpeg_next::ffi::BUFSIZ;
use http::HeaderMap;
use serde_json::Value;
use std::fs::{self, File};
use std::io::Read;
use std::io::Write;
use std::path::Path;
use std::sync::Arc;
use tokio::sync::Mutex;
use zip::ZipArchive;

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
        fs::create_dir_all(&mp3_dir).expect("Failed to create mp3 directory");
        fs::create_dir_all(&archive_dir).expect("Failed to create archive directory");
        let mut bit_artist = String::new();
        let mut bit_genre = String::new();
        while let Some(field) = multipart.next_field().await.unwrap() {
            if let Some(file_name) = field.file_name() {
                let fname = field.file_name().unwrap().to_string();
                let data = field.bytes().await.unwrap();
                let dir = if is_mp3(&data) {
                    &mp3_dir
                } else if is_archive(&data) {
                    &archive_dir
                } else {
                    &upload_dir
                };
                let file_path = dir.join(&fname);
                let mut file = File::create(&file_path).unwrap();
                file.write_all(&data).unwrap();
                if is_mp3(&data) {
                    let _ = &self
                        .service
                        .reduce_bitrate(&file_path.to_str().unwrap(), 200)
                        .unwrap();
                    let grpc_client = self.grpc_client.clone();
                    let file_path_str = file_path.to_str().unwrap().to_string();
                    let bit_artist_clone = bit_artist.clone();
                    let bit_genre_clone = bit_genre.clone();
                    tokio::spawn(async move {
                        let mut client = grpc_client.lock().await;
                        client
                            .upload_beat(
                                888811,
                                1,
                                &bit_artist_clone,
                                &bit_genre_clone,
                                &file_path_str,
                            )
                            .await
                            .unwrap();
                    });
                    let _ = &self
                        .service
                        .get_file_metadata(&file_path.to_str().unwrap(), &fname, user_id)
                        .await;
                }
                if is_archive(&data) {
                    let files = get_archive_files(&file_path, &mp3_dir).await.unwrap();
                    let tasks: Vec<_> = files
                        .into_iter()
                        .map(|file| {
                            let service = Arc::clone(&self.service);
                            let file = Arc::new(file);
                            let bit_artist_clone = bit_artist.clone();
                            let bit_genre_clone = bit_genre.clone();
                            let file_path_str = file_path.to_str().unwrap().to_string();
                            async move {
                                tokio::task::spawn_blocking({
                                    let service = Arc::clone(&service);
                                    let file = Arc::clone(&file);
                                    move || {
                                        service
                                            .reduce_bitrate(file.to_str().unwrap(), 200)
                                            .unwrap();
                                    }
                                })
                                .await
                                .unwrap();
                                let grpc_client = self.grpc_client.clone();
                                
                                tokio::spawn(async move {
                                    let mut client = grpc_client.lock().await;
                                    client
                                        .upload_beat(
                                            888811,
                                            1,
                                            &bit_artist_clone,
                                            &bit_genre_clone,
                                            &file_path_str,
                                        )
                                        .await
                                        .unwrap();
                                });
                                let _ = service
                                    .get_file_metadata(
                                        file.to_str().unwrap(),
                                        file.file_name().unwrap().to_str().unwrap(),
                                        user_id,
                                    )
                                    .await;
                            }
                        })
                        .collect();
                    futures::future::join_all(tasks).await;
                }
            } else if field.name() == Some("metadata") {
                let data = field.text().await.unwrap();
                let metadata: Value = serde_json::from_str(&data).unwrap();
                bit_artist = metadata
                    .get("bit_artist")
                    .and_then(|v| v.as_str())
                    .unwrap()
                    .to_string();
                bit_genre = metadata
                    .get("bit_genre")
                    .and_then(|v| v.as_str())
                    .unwrap()
                    .to_string();
                println!("{}", &bit_artist);
                println!("{}", &bit_genre);
                
            }
        }

        (StatusCode::OK, Json("File uploaded successfully"))
    }
}
