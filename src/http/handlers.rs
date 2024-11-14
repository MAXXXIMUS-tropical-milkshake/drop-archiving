use axum::extract::Multipart;
use axum::{http::StatusCode, response::IntoResponse, Json};
use std::fs::{self, File};
use std::io::Write;

use super::format::{is_archive, is_mp3};
use crate::service::archiving::Service;
use std::path::Path;

pub struct Handler {
    service: Service,
}

impl Handler {
    pub fn new(service: Service) -> Self {
        Self { service }
    }
    pub async fn upload(&self, mut multipart: Multipart) -> impl IntoResponse {
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
                self.service.get_file_metadata(&file_path.to_str().unwrap()).await;
            }
        }
        (StatusCode::OK, Json("File uploaded successfully"))
    }
}
