use std::{borrow::Cow, path::PathBuf};

use rust_embed::Embed;

/// Embedded default resources
#[derive(Embed)]
#[folder = "resources/"]
pub struct EmbeddedResources;

pub struct Resources;

impl Resources {
    fn resources_path() -> std::io::Result<PathBuf> {
        let exe_path = std::env::current_exe()?;
        let parent_path = exe_path
            .parent()
            .ok_or_else(|| std::io::Error::other("executable missing parent directory"))?;

        Ok(parent_path.join("resources"))
    }

    /// Get a file resources as bytes for the provided `path`
    ///
    /// At runtime this will check the file system relative ./resources
    /// path before checking the embedded resources
    pub async fn get_resource_bytes(path: &str) -> std::io::Result<Cow<'static, [u8]>> {
        let resources_path = Self::resources_path()?;
        let file_path = resources_path.join(path);

        let embedded: Option<rust_embed::EmbeddedFile> = EmbeddedResources::get(path);
        let mut file_contents: Option<std::io::Result<Vec<u8>>> = None;
        if file_path.exists() {
            let result = tokio::fs::read(file_path).await;
            file_contents = Some(result);
        }

        if let Some(file_contents) = file_contents {
            match file_contents {
                Ok(value) => return Ok(Cow::Owned(value)),
                Err(error) => {
                    tracing::error!(?error, %path, "failed to load resource file");
                }
            }
        }

        let embedded =
            embedded.ok_or_else(|| std::io::Error::other("embedded file for resource missing"))?;

        Ok(embedded.data.clone())
    }
}
