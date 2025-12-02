use crate::assets::Asset;
use crate::error::Result;
use std::path::Path;

/// Context provided to asset loaders
pub struct LoadContext<'a> {
    pub path: &'a Path,
    pub bytes: &'a [u8],
}

/// Trait for loading assets from bytes
pub trait AssetLoader: Send + Sync {
    type Asset: Asset;
    type Settings: Default + Send + Sync;

    /// Load asset from bytes
    fn load(&self, context: LoadContext, settings: &Self::Settings) -> Result<Self::Asset>;

    /// File extensions this loader supports
    fn extensions(&self) -> &[&str];
}

/// Simple binary data asset
#[derive(Clone, Debug)]
pub struct BinaryAsset {
    pub data: Vec<u8>,
    pub path: String,
}

impl Asset for BinaryAsset {}

/// Binary asset loader
pub struct BinaryLoader;

impl AssetLoader for BinaryLoader {
    type Asset = BinaryAsset;
    type Settings = ();

    fn load(&self, context: LoadContext, _settings: &Self::Settings) -> Result<Self::Asset> {
        Ok(BinaryAsset {
            data: context.bytes.to_vec(),
            path: context.path.to_string_lossy().to_string(),
        })
    }

    fn extensions(&self) -> &[&str] {
        &["bin", "dat"]
    }
}

/// JSON data asset
#[derive(Clone, Debug)]
pub struct JsonAsset {
    pub value: serde_json::Value,
    pub path: String,
}

impl Asset for JsonAsset {}

/// JSON asset loader
pub struct JsonLoader;

impl AssetLoader for JsonLoader {
    type Asset = JsonAsset;
    type Settings = ();

    fn load(&self, context: LoadContext, _settings: &Self::Settings) -> Result<Self::Asset> {
        let value: serde_json::Value = serde_json::from_slice(context.bytes).map_err(|e| {
            crate::error::EcsError::AssetLoadError(format!("JSON parse error: {e}"))
        })?;

        Ok(JsonAsset {
            value,
            path: context.path.to_string_lossy().to_string(),
        })
    }

    fn extensions(&self) -> &[&str] {
        &["json"]
    }
}

/// Text file asset
#[derive(Clone, Debug)]
pub struct TextAsset {
    pub content: String,
    pub path: String,
}

impl Asset for TextAsset {}

/// Text asset loader
pub struct TextLoader;

impl AssetLoader for TextLoader {
    type Asset = TextAsset;
    type Settings = ();

    fn load(&self, context: LoadContext, _settings: &Self::Settings) -> Result<Self::Asset> {
        let content = String::from_utf8(context.bytes.to_vec()).map_err(|e| {
            crate::error::EcsError::AssetLoadError(format!("UTF-8 decode error: {e}"))
        })?;

        Ok(TextAsset {
            content,
            path: context.path.to_string_lossy().to_string(),
        })
    }

    fn extensions(&self) -> &[&str] {
        &["txt", "md", "toml", "yaml", "yml"]
    }
}
