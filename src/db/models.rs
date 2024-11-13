
struct FileMetadata {
    id: i32,
    name: String,
    bitrate: f64,
    duration: f64,
    size: f64,
}

impl FileMetadata {
    pub fn new(id: i32, name: &str, bitrate: f64, duration: f64, size: f64) -> Self {
        Self {
            id,
            name: name.to_string(),
            bitrate,
            duration,
            size,
        }
    }
}


