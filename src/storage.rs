use crate::error::Result;
use crate::serialization::WorldData;
use std::fs;
use std::path::Path;

/// Format for serialization
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SerializationFormat {
    Json,
    Binary,
}

/// File storage for game saves
pub struct GameStorage;

impl GameStorage {
    /// Save world to file
    pub fn save_world(world: &WorldData, path: &Path, format: SerializationFormat) -> Result<()> {
        let data = match format {
            SerializationFormat::Json => world.to_json_bytes()?,
            SerializationFormat::Binary => world.to_binary_bytes()?,
        };

        fs::write(path, data).map_err(|e| {
            crate::error::EcsError::SerializationError(format!("Failed to write save file: {e}"))
        })
    }

    /// Load world from file
    pub fn load_world(path: &Path, format: SerializationFormat) -> Result<WorldData> {
        let data = fs::read(path).map_err(|e| {
            crate::error::EcsError::DeserializationError(format!("Failed to read save file: {e}"))
        })?;

        match format {
            SerializationFormat::Json => WorldData::from_json_bytes(&data),
            SerializationFormat::Binary => WorldData::from_binary_bytes(&data),
        }
    }

    /// Get file size
    pub fn get_file_size(path: &Path) -> Result<u64> {
        fs::metadata(path).map(|m| m.len()).map_err(|e| {
            crate::error::EcsError::SerializationError(format!("Failed to get file size: {e}"))
        })
    }

    /// List all save files in directory
    pub fn list_saves(directory: &Path) -> Result<Vec<String>> {
        let mut saves = Vec::new();

        if !directory.exists() {
            fs::create_dir_all(directory).map_err(|e| {
                crate::error::EcsError::SerializationError(format!(
                    "Failed to create directory: {e}"
                ))
            })?;
        }

        for entry in fs::read_dir(directory).map_err(|e| {
            crate::error::EcsError::SerializationError(format!("Failed to read directory: {e}"))
        })? {
            let entry = entry.map_err(|e| {
                crate::error::EcsError::SerializationError(format!(
                    "Failed to read directory entry: {e}"
                ))
            })?;
            let path = entry.path();
            if path.is_file() {
                if let Some(name) = path.file_name() {
                    if let Some(name_str) = name.to_str() {
                        saves.push(name_str.to_string());
                    }
                }
            }
        }

        Ok(saves)
    }

    /// Delete save file
    pub fn delete_save(path: &Path) -> Result<()> {
        fs::remove_file(path).map_err(|e| {
            crate::error::EcsError::SerializationError(format!("Failed to delete save file: {e}"))
        })
    }

    /// Create backup of save file
    pub fn backup_save(source: &Path, backup: &Path) -> Result<()> {
        fs::copy(source, backup).map_err(|e| {
            crate::error::EcsError::SerializationError(format!("Failed to create backup: {e}"))
        })?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::serialization::WorldData;
    use std::path::PathBuf;

    #[test]
    fn test_save_and_load_json() {
        let temp_path = PathBuf::from("test_save.json");

        let mut world = WorldData::new();
        world.add_metadata("test".to_string(), "value".to_string());

        GameStorage::save_world(&world, &temp_path, SerializationFormat::Json).unwrap();
        let loaded = GameStorage::load_world(&temp_path, SerializationFormat::Json).unwrap();

        assert_eq!(loaded.version, 1);
        assert_eq!(loaded.metadata.get("test"), Some(&"value".to_string()));

        let _ = fs::remove_file(temp_path);
    }

    #[test]
    fn test_save_and_load_binary() {
        let temp_path = PathBuf::from("test_save.bin");

        let mut world = WorldData::new();
        world.add_metadata("test".to_string(), "binary".to_string());

        GameStorage::save_world(&world, &temp_path, SerializationFormat::Binary).unwrap();
        let loaded = GameStorage::load_world(&temp_path, SerializationFormat::Binary).unwrap();

        assert_eq!(loaded.version, 1);
        assert_eq!(loaded.metadata.get("test"), Some(&"binary".to_string()));

        let _ = fs::remove_file(temp_path);
    }
}
