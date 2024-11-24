use anyhow::Error;

use crate::db::models::{BeatData, FileMetadata};
use crate::db::query::Db;
use crate::db::store::MinioStore;
use crate::libr::LOGGER;
use std::fs;
use std::fs::metadata;
use std::path::Path;
use std::process::Command;
use std::str;

use super::metadata::get_duration;

pub struct Service {
    db: Db,
    store: MinioStore,
}

impl Service {
    pub fn new(db: Db, store: MinioStore) -> Self {
        Self { db, store }
    }
    pub async fn insert_beat(
        &self,
        file_path_mp3: &str,
        fname_mp3: &str,
        user_id: i64,
        name: &str,
        description: &str,
        genres: &Vec<String>,
        file_path_image: &str,
        fname_image: &str,
        bitrate: f64
    ) -> Result<i64, Error> {
        // let bitrate_cmd = Command::new("ffprobe")
        //     .args(&[
        //         "-v",
        //         "error",
        //         "-select_streams",
        //         "v:0",
        //         "-show_entries",
        //         "format=bit_rate",
        //         "-of",
        //         "default=noprint_wrappers=1:nokey=1",
        //         file_path_mp3,
        //     ])
        //     .output()
        //     .unwrap();
        // let bitrate = str::from_utf8(&bitrate_cmd.stdout)
        //     .unwrap()
        //     .trim()
        //     .parse::<f64>()
        //     .unwrap();
        // println!("Bitrate: {} bps", bitrate);

        // let duration_cmd = Command::new("ffprobe")
        //     .args(&[
        //         "-v",
        //         "error",
        //         "-show_entries",
        //         "format=duration",
        //         "-of",
        //         "default=noprint_wrappers=1:nokey=1",
        //         file_path_mp3,
        //     ])
        //     .output()
        //     .unwrap();
        // let duration = str::from_utf8(&duration_cmd.stdout)
        //     .unwrap()
        //     .trim()
        //     .parse::<f64>()
        //     .unwrap();
        // println!("Duration: {} seconds", duration);
        let duration = get_duration(&file_path_mp3).unwrap();
        let size = metadata(&file_path_mp3).unwrap().len() as f64 / (1024.0 * 1024.0);
        let file = FileMetadata::new(&fname_mp3, bitrate, duration, size);
        let beat_id = self.db.insert_file_metadata(file).await.unwrap();
        let data = BeatData::new(
            beat_id,
            name.to_string(),
            description.to_string(),
            genres.clone(),
            user_id,
        );
        let _ = self.db.insert_beat_data(data).await.unwrap();
        let _ = self
            .db
            .insert_image(beat_id, fname_image.to_string())
            .await
            .unwrap();
        let _ = self
            .store
            .upload_file(file_path_mp3, fname_mp3, file_path_image, fname_image)
            .await;

        Ok(beat_id)
    }

    pub fn reduce_bitrate(&self, input_file_path: &str, input_file_name: &str, target_bitrate: u32) -> Result<(f64, String, String), Error> {
        if !Path::new(input_file_path).exists() {
            return Err(anyhow::anyhow!("Input file does not exist: {}", input_file_path));
        }

        let reduced_file = format!(
                "{}_reduced{}",
                input_file_path.trim_end_matches(".mp3"),
                ".mp3"
            );
        let reduced_file_name = format!(
                "{}_reduced{}",
                input_file_name.trim_end_matches(".mp3"),
                ".mp3"
            );
        let target_bitrate_str = format!("{}k", target_bitrate);
        println!("{}", reduced_file);

        let status = Command::new("ffmpeg")
            .args(&[
                "-i",
                input_file_path,
                "-map",
                "0:a",
                "-b:a",
                &target_bitrate_str,
                "-c:a",
                "libmp3lame",
                "-f",
                "mp3",
                "-y",
                &reduced_file,
            ])
            .status();
        match status {
            Ok(s) if s.success() => {
                LOGGER.info(&format!("Bitrate reduced successfully, updated file: {}", input_file_path));
                Ok((target_bitrate as f64 * 1000f64, reduced_file_name, reduced_file))
            }
            Ok(s) => {
                let _ = fs::remove_file(&reduced_file);
                Err(anyhow::anyhow!(
                    "ffmpeg exited with a non-zero status: {}",
                    s.code().unwrap_or(-1)
                ))
            }
            Err(e) => {
                let _ = fs::remove_file(&reduced_file);
                Err(anyhow::anyhow!("Failed to execute ffmpeg: {}", e))
            }
        }
    }
}
