use std::fs::metadata;
use std::process::Command;
use std::str;

use crate::db::models::FileMetadata;
use crate::db::query::Db;
use crate::db::store::MinioStore;
use crate::lib::LOGGER;

pub struct Service {
    db: Db,
    store: MinioStore, //pub logger: Logger,
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
        let file = FileMetadata::new(&file_path, bitrate, duration, size);
        let _ = self.db.insert(file).await;
        let _ = self.store.upload_file(file_path,fname).await;
    }
}
