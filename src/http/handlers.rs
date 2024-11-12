use axum::extract::Multipart;
use axum::{http::StatusCode, response::IntoResponse, Json};
use std::fs::{self, File};
use std::io::Write;
use std::path::Path;


pub async fn upload(mut multipart: Multipart) -> impl IntoResponse {
    let upload_dir = Path::new("./uploads");
    if !upload_dir.exists() {
        fs::create_dir_all(&upload_dir).expect("Failed to create upload directory");
    }
    while let Some(field) = multipart.next_field().await.unwrap() {
        let fname = field.file_name().unwrap().to_string();
        let data = field.bytes().await.unwrap();
        let mut file = File::create(format!("./uploads/{}", fname)).unwrap();
        file.write_all(&data).unwrap()
       
    }
    (StatusCode::OK, Json("File uploaded successfully"))
}