use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

use percent_encoding::{utf8_percent_encode, NON_ALPHANUMERIC};
use uuid::Uuid;

#[derive(Clone)]
pub struct SharedFile {
    pub id: Uuid,
    pub name: String,
    pub size: u64,
    pub created_at_ms: u128,
    pub(crate) path: PathBuf,
}

impl SharedFile {
    pub fn new(name: String, size: u64, path: PathBuf) -> Self {
        let created_at_ms = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis();
        Self {
            id: Uuid::new_v4(),
            name,
            size,
            created_at_ms,
            path,
        }
    }

    pub fn content_type(&self) -> &'static str {
        match Path::new(&self.name)
            .extension()
            .and_then(|extension| extension.to_str())
            .unwrap_or_default()
            .to_ascii_lowercase()
            .as_str()
        {
            "7z" => "application/x-7z-compressed",
            "avif" => "image/avif",
            "csv" => "text/csv; charset=utf-8",
            "gif" => "image/gif",
            "heic" => "image/heic",
            "jpeg" | "jpg" => "image/jpeg",
            "json" => "application/json",
            "m4a" => "audio/mp4",
            "mp3" => "audio/mpeg",
            "mp4" => "video/mp4",
            "pdf" => "application/pdf",
            "png" => "image/png",
            "svg" => "image/svg+xml",
            "txt" => "text/plain; charset=utf-8",
            "webp" => "image/webp",
            "zip" => "application/zip",
            _ => "application/octet-stream",
        }
    }

    pub fn content_disposition(&self) -> String {
        let fallback = self
            .name
            .chars()
            .map(|character| {
                if character.is_ascii_graphic() && !matches!(character, '"' | '\\') {
                    character
                } else {
                    '_'
                }
            })
            .collect::<String>();
        let encoded = utf8_percent_encode(&self.name, NON_ALPHANUMERIC);
        format!("attachment; filename=\"{fallback}\"; filename*=UTF-8''{encoded}")
    }
}

pub fn safe_filename(value: &str) -> String {
    let normalized = value.replace('\\', "/");
    let leaf = normalized.rsplit('/').next().unwrap_or("file");
    let cleaned = leaf
        .chars()
        .filter(|character| !character.is_control())
        .take(180)
        .collect::<String>()
        .trim()
        .to_owned();
    if cleaned.is_empty() {
        "file".to_owned()
    } else {
        cleaned
    }
}
