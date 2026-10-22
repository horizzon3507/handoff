use std::io;
use std::path::PathBuf;

use uuid::Uuid;

pub struct RoomStore {
    directory: PathBuf,
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

        Ok(Self { directory })
    }

    pub fn directory(&self) -> &PathBuf {
        &self.directory
    }

    pub async fn cleanup(&self) -> io::Result<()> {
        tokio::fs::remove_dir_all(&self.directory).await
    }
}

impl Drop for RoomStore {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.directory);
    }
}
