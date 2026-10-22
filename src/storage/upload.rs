use std::io;

use axum::extract::multipart::Field;
use tokio::io::AsyncWriteExt;
use uuid::Uuid;

use super::{safe_filename, RoomStore, SharedFile};

impl RoomStore {
    pub async fn save_upload(&self, field: &mut Field<'_>) -> Result<SharedFile, String> {
        let _upload_guard = self.lock_uploads().await;
        let name = safe_filename(field.file_name().unwrap_or("file"));
        self.ensure_capacity(1, 0)
            .await
            .map_err(|error| error.to_string())?;

        let path = self.directory().join(Uuid::new_v4().to_string());
        let result = self.write_field(field, &path).await;
        match result {
            Ok(size) => {
                let shared_file = SharedFile::new(name, size, path);
                self.inner
                    .files
                    .lock()
                    .await
                    .insert(shared_file.id, shared_file.clone());
                Ok(shared_file)
            }
            Err(error) => {
                let _ = tokio::fs::remove_file(path).await;
                Err(error)
            }
        }
    }

    async fn write_field(
        &self,
        field: &mut Field<'_>,
        path: &std::path::Path,
    ) -> Result<u64, String> {
        let mut file = tokio::fs::OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(path)
            .await
            .map_err(storage_error)?;
        let mut size = 0u64;

        loop {
            let chunk = field
                .chunk()
                .await
                .map_err(|_| "The upload was interrupted. Try again.".to_owned())?;
            let Some(chunk) = chunk else { break };
            size = size.saturating_add(chunk.len() as u64);
            self.ensure_capacity(1, size)
                .await
                .map_err(|error| error.to_string())?;
            file.write_all(&chunk).await.map_err(storage_error)?;
        }

        file.flush().await.map_err(storage_error)?;
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            tokio::fs::set_permissions(path, std::fs::Permissions::from_mode(0o600))
                .await
                .map_err(storage_error)?;
        }

        Ok(size)
    }
}

fn storage_error(_: io::Error) -> String {
    "The file could not be saved. Check available disk space and try again.".to_owned()
}
