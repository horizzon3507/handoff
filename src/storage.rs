use std::collections::HashMap;
use std::io;
use std::path::PathBuf;
use std::sync::Arc;

use tokio::sync::Mutex;
use uuid::Uuid;

mod file;
pub use file::{safe_filename, SharedFile};

#[derive(Clone)]
pub struct RoomStore {
    inner: Arc<RoomStoreInner>,
}

struct RoomStoreInner {
    directory: PathBuf,
    files: Mutex<HashMap<Uuid, SharedFile>>,
    upload_gate: Mutex<()>,
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

    pub async fn cleanup(&self) -> io::Result<()> {
        tokio::fs::remove_dir_all(&self.inner.directory).await
    }
}

impl Drop for RoomStoreInner {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.directory);
    }
}
