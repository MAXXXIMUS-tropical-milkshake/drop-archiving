pub struct FileMetadata {
    pub name: String,
    pub bitrate: f64,
    pub duration: f64,
    pub size: f64,
}

pub struct BeatData {
    pub beat_id: i32,
    pub name: String,
    pub description: String,
    pub genres: Vec<String>,
    pub beatmaker_id: i64,
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

impl BeatData {
    pub fn new(
        beat_id: i64,
        name: String,
        description: String,
        genres: Vec<String>,
        beatmaker_id: i64,
    ) -> Self {
        Self {
            beat_id: beat_id as i32,
            name,
            description,
            genres,
            beatmaker_id,
        }
    }
}
