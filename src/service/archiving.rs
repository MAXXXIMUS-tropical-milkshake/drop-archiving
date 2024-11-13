use std::fs::metadata;
use std::process::Command;
use std::str;

pub fn get_file_metadata(file_path: &str) {
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
    let bitrate = str::from_utf8(&bitrate_cmd.stdout).unwrap().trim();
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
    let duration = str::from_utf8(&duration_cmd.stdout).unwrap().trim();
    println!("Duration: {} seconds", duration);

    println!(
        "Size of file is {} Mb",
        metadata(&file_path).unwrap().len() as f64 / (1024.0 * 1024.0)
    );
}
