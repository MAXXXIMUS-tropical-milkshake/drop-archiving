use super::archiving::get_archive_files;
use super::format::{is_archive, is_mp3};
use crate::client::grpc::client::GrpcClient;
use crate::lib::LOGGER;
use crate::service::archiving::Service;
use axum::extract::Multipart;
use axum::{http::StatusCode, response::IntoResponse, Json};
use http::HeaderMap;
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
    pub async fn upload(&self, mut multipart: Multipart, headers: HeaderMap) -> impl IntoResponse {
        LOGGER.info("Upload started");
        let user_id = headers
            .get("user_id")
            .and_then(|val| val.to_str().ok())
            .and_then(|val| val.parse::<i64>().ok());
        if user_id.is_none() {
            return (StatusCode::BAD_REQUEST, Json("Invalid header"));
        }
        let user_id = user_id.unwrap();
        let upload_dir = Path::new("./uploads").to_path_buf();
        let mp3_dir = upload_dir.join("mp3");
        let archive_dir = upload_dir.join("archive");
        fs::create_dir_all(&mp3_dir).expect("Failed to create mp3 directory");
        fs::create_dir_all(&archive_dir).expect("Failed to create archive directory");

        while let Some(field) = multipart.next_field().await.unwrap() {
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
                // let grpc_client = self.grpc_client.clone();
                // let file_path_str = file_path.to_str().unwrap().to_string();
                // tokio::spawn(async move {
                //     let mut client = grpc_client.lock().await;
                //     client
                //         .upload_beat(888811, 1, "dfdf", "dffd", &file_path_str)
                //         .await
                //         .unwrap();
                // });
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
                        async move {
                            tokio::task::spawn_blocking({
                                let service = Arc::clone(&service);
                                let file = Arc::clone(&file);
                                move || {
                                service.reduce_bitrate(file.to_str().unwrap(), 200).unwrap();
                            }})
                            .await
                            .unwrap();
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
        }
        (StatusCode::OK, Json("File uploaded successfully"))
    }
}
