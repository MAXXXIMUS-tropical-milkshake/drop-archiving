use crate::db::models::FileMetadata;
use crate::db::query::Db;
use crate::db::store::MinioStore;
use crate::lib::LOGGER;
use std::fs;
use std::fs::metadata;
use std::path::Path;
use std::process::Command;
use std::str;

pub struct Service {
    db: Db,
    store: MinioStore,
}

impl Service {
    pub fn new(db: Db, store: MinioStore) -> Self {
        Self { db, store }
    }
    pub async fn get_file_metadata(&self, file_path: &str, fname: &str) {
        let bitrate_cmd = Command::new("ffprobe")
            .args(&[
                "-v",
                "error",
                "-select_streams",
                "v:0",
                "-show_entries",
                "format=bit_rate",
                "-of",
                "default=noprint_wrappers=1:nokey=1",
                file_path,
            ])
            .output()
            .unwrap();
        let bitrate = str::from_utf8(&bitrate_cmd.stdout)
            .unwrap()
            .trim()
            .parse::<f64>()
            .unwrap();
        println!("Bitrate: {} bps", bitrate);

        let duration_cmd = Command::new("ffprobe")
            .args(&[
                "-v",
                "error",
                "-show_entries",
                "format=duration",
                "-of",
                "default=noprint_wrappers=1:nokey=1",
                file_path,
            ])
            .output()
            .unwrap();
        let duration = str::from_utf8(&duration_cmd.stdout)
            .unwrap()
            .trim()
            .parse::<f64>()
            .unwrap();
        println!("Duration: {} seconds", duration);
        let size = metadata(&file_path).unwrap().len() as f64 / (1024.0 * 1024.0);
        println!("Size of file is {} Mb", size);
        let file = FileMetadata::new(&fname, bitrate, duration, size);
        let _ = self.db.insert(file).await;
        let _ = self.store.upload_file(file_path, fname).await;
    }

    pub fn reduce_bitrate(
        &self,
        input_file: &str,
        target_bitrate: u32,
    ) -> Result<(), String> {
        if !Path::new(input_file).exists() {
            return Err(format!("Input file does not exist: {}", input_file));
        }

        let temp_file = format!("{}.temp", input_file);
        let target_bitrate_str = format!("{}k", target_bitrate);

        let status = Command::new("ffmpeg")
            .args(&[
                "-i",
                input_file,
                "-map",
                "0:a",
                "-b:a",
                &target_bitrate_str,
                "-c:a",
                "libmp3lame",
                "-f",
                "mp3",
                "-y",
                &temp_file,
            ])
            .status();

        match status {
            Ok(s) if s.success() => {
                fs::rename(&temp_file, input_file)
                    .map_err(|e| format!("Failed to overwrite input file: {}", e))?;
                println!("Bitrate reduced successfully, updated file: {}", input_file);
                Ok(())
            }
            Ok(s) => {
                let _ = fs::remove_file(&temp_file);
                Err(format!(
                    "ffmpeg exited with a non-zero status: {}",
                    s.code().unwrap_or(-1)
                ))
            }
            Err(e) => {
                let _ = fs::remove_file(&temp_file);
                Err(format!("Failed to execute ffmpeg: {}", e))
            }
        }
    }
}
