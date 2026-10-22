use std::collections::HashMap;
use std::fmt;
use std::io;
use std::path::PathBuf;
use std::sync::Arc;

use tokio::sync::{Mutex, MutexGuard};
use uuid::Uuid;

mod file;
mod upload;
pub use file::{safe_filename, SharedFile};

#[derive(Clone, Copy)]
pub struct UploadLimits {
    pub file_bytes: u64,
    pub room_bytes: u64,
    pub room_files: usize,
}

#[derive(Debug)]
pub enum UploadLimitError {
    FileTooLarge,
    RoomFileLimit,
    RoomSizeLimit,
}

impl fmt::Display for UploadLimitError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        let message = match self {
            Self::FileTooLarge => "Each file must be 512 MB or smaller.",
            Self::RoomFileLimit => "This room already has 200 files.",
            Self::RoomSizeLimit => "This room has reached its 2 GB limit.",
        };
        formatter.write_str(message)
    }
}

#[derive(Clone)]
pub struct RoomStore {
    inner: Arc<RoomStoreInner>,
}

struct RoomStoreInner {
    directory: PathBuf,
    files: Mutex<HashMap<Uuid, SharedFile>>,
    upload_gate: Mutex<()>,
    limits: UploadLimits,
}

impl RoomStore {
    pub async fn new() -> io::Result<Self> {
        let directory = std::env::temp_dir().join(format!("handoff-{}", Uuid::new_v4()));
        tokio::fs::create_dir(&directory).await?;

        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            tokio::fs::set_permissions(&directory, std::fs::Permissions::from_mode(0o700)).await?;
        }

        Ok(Self {
            inner: Arc::new(RoomStoreInner {
                directory,
                files: Mutex::new(HashMap::new()),
                upload_gate: Mutex::new(()),
                limits: UploadLimits {
                    file_bytes: crate::config::MAX_FILE_BYTES,
                    room_bytes: crate::config::MAX_ROOM_BYTES,
                    room_files: crate::config::MAX_ROOM_FILES,
                },
            }),
        })
    }

    pub fn directory(&self) -> &PathBuf {
        &self.inner.directory
    }

    pub async fn list(&self) -> Vec<SharedFile> {
        let files = self.inner.files.lock().await;
        let mut result = files.values().cloned().collect::<Vec<_>>();
        result.sort_by(|left, right| right.created_at_ms.cmp(&left.created_at_ms));
        result
    }

    pub async fn get(&self, id: Uuid) -> Option<SharedFile> {
        self.inner.files.lock().await.get(&id).cloned()
    }

    pub fn limits(&self) -> UploadLimits {
        self.inner.limits
    }

    pub async fn lock_uploads(&self) -> MutexGuard<'_, ()> {
        self.inner.upload_gate.lock().await
    }

    pub async fn ensure_capacity(
        &self,
        additional_files: usize,
        additional_bytes: u64,
    ) -> Result<(), UploadLimitError> {
        if additional_bytes > self.inner.limits.file_bytes {
            return Err(UploadLimitError::FileTooLarge);
        }

        let files = self.inner.files.lock().await;
        if files.len() + additional_files > self.inner.limits.room_files {
            return Err(UploadLimitError::RoomFileLimit);
        }

        let current_bytes = files.values().map(|file| file.size).sum::<u64>();
        if current_bytes + additional_bytes > self.inner.limits.room_bytes {
            return Err(UploadLimitError::RoomSizeLimit);
        }

        Ok(())
    }

    pub async fn cleanup(&self) -> io::Result<()> {
        tokio::fs::remove_dir_all(&self.inner.directory).await
    }
}

impl Drop for RoomStoreInner {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.directory);
    }
}
