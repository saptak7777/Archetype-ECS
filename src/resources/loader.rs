use crate::error::Result;
use crate::resources::asset_types::DataResource;
use std::fs;
use std::path::Path;

/// Synchronous resource loader
pub struct ResourceLoader;

impl ResourceLoader {
    /// Load binary data from file
    pub fn load_binary(path: &str) -> Result<DataResource> {
        let data = fs::read(path).map_err(|e| {
            crate::error::EcsError::ResourceLoadError(format!(
                "Failed to load file {path}: {e}"
            ))
        })?;

        Ok(DataResource::new(path.to_string(), data))
    }

    /// Load text from file
    pub fn load_text(path: &str) -> Result<String> {
        fs::read_to_string(path).map_err(|e| {
            crate::error::EcsError::ResourceLoadError(format!(
                "Failed to load file {path}: {e}"
            ))
        })
    }

    /// Check if file exists
    pub fn file_exists(path: &str) -> bool {
        Path::new(path).exists()
    }

    /// Get file size
    pub fn get_file_size(path: &str) -> Result<u64> {
        fs::metadata(path).map(|m| m.len()).map_err(|e| {
            crate::error::EcsError::ResourceLoadError(format!(
                "Failed to get file size {path}: {e}"
            ))
        })
    }

    /// List files in directory
    pub fn list_files(directory: &str, extension: &str) -> Result<Vec<String>> {
        let mut files = Vec::new();

        for entry in fs::read_dir(directory).map_err(|e| {
            crate::error::EcsError::ResourceLoadError(format!(
                "Failed to read directory {directory}: {e}"
            ))
        })? {
            let entry = entry.map_err(|e| {
                crate::error::EcsError::ResourceLoadError(format!("Failed to read entry: {e}"))
            })?;
            let path = entry.path();
            if path.is_file() {
                if let Some(ext) = path.extension() {
                    if ext == extension {
                        if let Some(name) = path.file_name() {
                            if let Some(name_str) = name.to_str() {
                                files.push(name_str.to_string());
                            }
                        }
                    }
                }
            }
        }

        Ok(files)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_file_exists() {
        let exists = ResourceLoader::file_exists("Cargo.toml");
        assert!(exists);

        let not_exists = ResourceLoader::file_exists("nonexistent.txt");
        assert!(!not_exists);
    }

    #[test]
    fn test_get_file_size() {
        if ResourceLoader::file_exists("Cargo.toml") {
            let size = ResourceLoader::get_file_size("Cargo.toml").unwrap();
            assert!(size > 0);
        }
    }
}
