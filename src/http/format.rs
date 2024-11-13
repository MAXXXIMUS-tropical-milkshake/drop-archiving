use std::{
    fs::{self, File},
    io::Write,
    path::Path,
    process::Command,
};


pub fn is_mp3(data: &[u8]) -> bool {
    if let Some(kind) = infer::get(data) {
        kind.mime_type() == "audio/mpeg"
    } else {
        false
    }
}

pub fn is_archive(data: &[u8]) -> bool {
    if let Some(kind) = infer::get(data) {
        kind.mime_type() == "application/zip"
    } else {
        false
    }
}
