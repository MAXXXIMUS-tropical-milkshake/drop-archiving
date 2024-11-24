use super::archiving::get_archive_files;
use super::format::{is_archive, is_image, is_mp3};
use crate::client::grpc::client::GrpcClient;
use crate::libr::LOGGER;
use crate::service::archiving::Service;
use crate::service::metadata::get_bitrate;
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
        let image_dir = upload_dir.join("image");
        fs::create_dir_all(&mp3_dir).expect("Failed to create mp3 directory");
        fs::create_dir_all(&archive_dir).expect("Failed to create archive directory");
        fs::create_dir_all(&image_dir).expect("Failed to create image directory");
        let mut name = String::new();
        let mut beat_genre = Vec::new();
        let mut description = String::new();
        let mut file_path_mp3 = String::new();
        let mut file_path_image = String::new();
        let mut fname_mp3 = String::new();
        let mut fname_image = String::new();
        while let Some(field) = multipart.next_field().await.unwrap() {
            if let Some(_file_name) = field.file_name() {
                let fname = field.file_name().unwrap().to_string();
                let data = field.bytes().await.unwrap();
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
                let mut file = File::create(&file_path).unwrap();
                file.write_all(&data).unwrap();
                if is_mp3(&data) {
                    file_path_mp3 = file_path.to_str().unwrap().to_string();
                    fname_mp3 = fname;
                } else if is_image(&data) {
                    file_path_image = file_path.to_str().unwrap().to_string();
                    fname_image = fname;
                }
                // if is_mp3(&data) {
                //     let _ = &self
                //         .service
                //         .reduce_bitrate(&file_path.to_str().unwrap(), 200)
                //         .unwrap();
                //     let grpc_client = self.grpc_client.clone();
                //     let file_path_str = file_path.to_str().unwrap().to_string();
                //     let artist_name_clone = artist_name.clone();
                //     let description_clone = description.clone();
                //     let bit_genre_clone = bit_genre.clone();

                //     tokio::spawn(async move {
                //         let mut client = grpc_client.lock().await;
                //         client
                //             .upload_beat(
                //                 1343,
                //                 user_id,
                //                 &artist_name_clone,
                //                 &description_clone,
                //                 bit_genre_clone,
                //                 &file_path_str,
                //             )
                //             .await
                //             .unwrap();
                //     });
                //     let _ = &self
                //         .service
                //         .get_file_metadata(&file_path.to_str().unwrap(), &fname, user_id)
                //         .await;
                // }
                // if is_archive(&data) {
                //     let files = get_archive_files(&file_path, &mp3_dir).await.unwrap();
                //     let tasks: Vec<_> = files
                //         .into_iter()
                //         .map(|file| {
                //             let service = Arc::clone(&self.service);
                //             let file = Arc::new(file);
                //             async move {
                //                 tokio::task::spawn_blocking({
                //                     let service = Arc::clone(&service);
                //                     let file = Arc::clone(&file);
                //                     move || {
                //                         service
                //                             .reduce_bitrate(file.to_str().unwrap(), 200)
                //                             .unwrap();
                //                     }
                //                 })
                //                 .await
                //                 .unwrap();
                //                 let _ = service
                //                     .insert_beat(
                //                         file.to_str().unwrap(),
                //                         file.file_name().unwrap().to_str().unwrap(),
                //                         user_id,
                //                     )
                //                     .await;
                //             }
                //         })
                //         .collect();
                //     futures::future::join_all(tasks).await;
                // }
            } else if field.name() == Some("metadata") {
                let data = field.text().await.unwrap();
                let metadata: Value = serde_json::from_str(&data).unwrap();
                name = metadata
                    .get("name")
                    .and_then(|v| v.as_str())
                    .unwrap()
                    .to_string();
                beat_genre = metadata
                    .get("beat_genre")
                    .and_then(|v| v.as_array())
                    .unwrap()
                    .iter()
                    .map(|v| v.as_str().unwrap().to_string())
                    .collect();
                description = metadata
                    .get("description")
                    .and_then(|v| v.as_str())
                    .unwrap()
                    .to_string();

                println!("{}", &name);
                println!("{:?}", &beat_genre);
                println!("{}", &description);
            }
        }
        let bitrate = get_bitrate(&file_path_mp3).unwrap();
        let mut beat_id = self
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
            )
            .await
            .unwrap();
        if bitrate > 200_000f64 {
            let (bitrate, fname_mp3_reduced, file_path_mp3_reduced) = self.service.reduce_bitrate(&file_path_mp3, &fname_mp3, 200).unwrap();
            file_path_mp3 = file_path_mp3_reduced;
            beat_id = self
                .service
                .insert_beat(
                    &file_path_mp3,
                    &fname_mp3_reduced,
                    user_id,
                    &name,
                    &description,
                    &beat_genre,
                    &file_path_image,
                    &fname_image,
                    bitrate,
                )
                .await
                .unwrap();
        }
        let mut client = self.grpc_client.lock().await;
        let _ = client
            .upload_beat(
                beat_id,
                user_id,
                &name,
                &description,
                beat_genre,
                &file_path_mp3,
                &file_path_image,
            )
            .await
            .unwrap();

        (StatusCode::OK, Json("File uploaded successfully"))
    }
}
