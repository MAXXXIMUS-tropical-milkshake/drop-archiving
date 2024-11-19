use std::{
    fs::File,
    io::{Read, Write},
    path::{Path, PathBuf},
};

use zip::ZipArchive;

use super::format::is_mp3;

pub fn get_archive_files(
    path: &Path,
    mp3_dir: &PathBuf,
) -> Result<Vec<PathBuf>, Box<dyn std::error::Error>> {
    let file = File::open(path)?;
    let mut archive = ZipArchive::new(file)?;
    let mut files = Vec::new();
    for i in 0..archive.len() {
        let mut file = archive.by_index(i).unwrap();
        let fname = Path::new(file.name())
            .file_name()
            .unwrap()
            .to_str()
            .unwrap()
            .to_string();
        let mut data = Vec::new();
        file.read_to_end(&mut data)?;
        if is_mp3(&data) {
            let file_path = mp3_dir.join(&fname);
            println!("{}", &file_path.to_str().unwrap());
            let mut file = File::create(&file_path).unwrap();
            file.write_all(&data).unwrap();
            files.push(file_path);
        }
    }
    Ok(files)
}
