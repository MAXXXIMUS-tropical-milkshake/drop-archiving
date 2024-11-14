
pub struct FileMetadata {
    pub name: String,
    pub bitrate: f64,
    pub duration: f64,
    pub size: f64,
}

impl FileMetadata {
    pub fn new(name: &str, bitrate: f64, duration: f64, size: f64) -> Self {
        Self {
            name: name.to_string(),
            bitrate,
            duration,
            size,
        }
    }
}


