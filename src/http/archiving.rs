use std::{
    fs::File,
    io::{Read, Write},
    path::{Path, PathBuf},
    sync::Arc,
};

use zip::ZipArchive;

use super::format::is_mp3;

pub async fn get_archive_files(
    path: &Path,
    mp3_dir: &PathBuf,
) -> Result<Vec<PathBuf>, Box<dyn std::error::Error>> {
    let file = File::open(path)?;
    let archive = ZipArchive::new(file)?;
    let path = Arc::new(path.to_path_buf());
    let mp3_dir = Arc::new(mp3_dir.clone());
    let tasks: Vec<_> = (0..archive.len())
        .map(|i| {
            let path = Arc::clone(&path);
            let mp3_dir = Arc::clone(&mp3_dir);
            tokio::task::spawn_blocking(move || {
                let file = File::open(&*path).unwrap();
                let mut archive = ZipArchive::new(file).unwrap();
                let mut file = archive.by_index(i).unwrap();
                let fname = Path::new(file.name())
                    .file_name()
                    .unwrap()
                    .to_str()
                    .unwrap()
                    .to_string();
                let mut data = Vec::new();
                file.read_to_end(&mut data).unwrap();
                if is_mp3(&data) {
                    let file_path = mp3_dir.join(&fname);
                    println!("{}", &file_path.to_str().unwrap());
                    let mut file = File::create(&file_path).unwrap();
                    file.write_all(&data).unwrap();
                    Some(file_path)
                } else {
                    None
                }
            })
        })
        .collect();
    let mut files = Vec::new();
    for task in tasks {
        if let Some(file_path) = task.await.unwrap() {
            files.push(file_path);
        }
    }
    Ok(files)
}
