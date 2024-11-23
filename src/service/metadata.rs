use core::str;
use std::process::Command;

use anyhow::Error;
pub fn get_bitrate(file_path: &str) -> Result<f64, Error> {
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
    Ok(bitrate)
}

pub fn get_duration(file_path: &str) -> Result<f64, Error> {
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
    Ok(duration)
}
