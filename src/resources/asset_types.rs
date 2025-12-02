use crate::error::Result;
use crate::resources::Resource;
use std::any::TypeId;

/// Texture resource
#[derive(Clone, Debug)]
pub struct TextureResource {
    path: String,
    width: u32,
    height: u32,
    data: Vec<u8>,
}

impl TextureResource {
    pub fn new(path: String, width: u32, height: u32, data: Vec<u8>) -> Self {
        Self {
            path,
            width,
            height,
            data,
        }
    }

    pub fn width(&self) -> u32 {
        self.width
    }
    pub fn height(&self) -> u32 {
        self.height
    }
    pub fn data(&self) -> &[u8] {
        &self.data
    }
}

impl Resource for TextureResource {
    fn get_path(&self) -> &str {
        &self.path
    }
    fn get_size(&self) -> usize {
        self.data.len()
    }
    fn get_type_name(&self) -> &str {
        "Texture"
    }
    fn get_type_id(&self) -> TypeId {
        TypeId::of::<Self>()
    }
    fn unload(&mut self) -> Result<()> {
        self.data.clear();
        Ok(())
    }
    fn reload(&mut self) -> Result<()> {
        // In real implementation, reload from disk
        Ok(())
    }
    fn is_valid(&self) -> bool {
        !self.data.is_empty()
    }
}

/// Audio resource
#[derive(Clone, Debug)]
pub struct AudioResource {
    path: String,
    sample_rate: u32,
    channels: u8,
    data: Vec<f32>,
}

impl AudioResource {
    pub fn new(path: String, sample_rate: u32, channels: u8, data: Vec<f32>) -> Self {
        Self {
            path,
            sample_rate,
            channels,
            data,
        }
    }

    pub fn sample_rate(&self) -> u32 {
        self.sample_rate
    }
    pub fn channels(&self) -> u8 {
        self.channels
    }
    pub fn data(&self) -> &[f32] {
        &self.data
    }
    pub fn duration_seconds(&self) -> f32 {
        self.data.len() as f32 / (self.sample_rate as f32 * self.channels as f32)
    }
}

impl Resource for AudioResource {
    fn get_path(&self) -> &str {
        &self.path
    }
    fn get_size(&self) -> usize {
        self.data.len() * 4
    }
    fn get_type_name(&self) -> &str {
        "Audio"
    }
    fn get_type_id(&self) -> TypeId {
        TypeId::of::<Self>()
    }
    fn unload(&mut self) -> Result<()> {
        self.data.clear();
        Ok(())
    }
    fn reload(&mut self) -> Result<()> {
        Ok(())
    }
    fn is_valid(&self) -> bool {
        !self.data.is_empty()
    }
}

/// Data resource (generic binary data)
#[derive(Clone, Debug)]
pub struct DataResource {
    path: String,
    data: Vec<u8>,
}

impl DataResource {
    pub fn new(path: String, data: Vec<u8>) -> Self {
        Self { path, data }
    }

    pub fn data(&self) -> &[u8] {
        &self.data
    }
    pub fn data_mut(&mut self) -> &mut Vec<u8> {
        &mut self.data
    }
}

impl Resource for DataResource {
    fn get_path(&self) -> &str {
        &self.path
    }
    fn get_size(&self) -> usize {
        self.data.len()
    }
    fn get_type_name(&self) -> &str {
        "Data"
    }
    fn get_type_id(&self) -> TypeId {
        TypeId::of::<Self>()
    }
    fn unload(&mut self) -> Result<()> {
        self.data.clear();
        Ok(())
    }
    fn reload(&mut self) -> Result<()> {
        Ok(())
    }
    fn is_valid(&self) -> bool {
        !self.data.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_texture_resource() {
        let texture =
            TextureResource::new("test.png".to_string(), 256, 256, vec![0u8; 256 * 256 * 4]);
        assert_eq!(texture.get_type_name(), "Texture");
        assert_eq!(texture.width(), 256);
        assert_eq!(texture.height(), 256);
        assert!(texture.is_valid());
    }

    #[test]
    fn test_audio_resource() {
        let audio = AudioResource::new("test.wav".to_string(), 44100, 2, vec![0.0f32; 44100]);
        assert_eq!(audio.sample_rate(), 44100);
        assert!(audio.duration_seconds() > 0.0);
    }

    #[test]
    fn test_data_resource() {
        let data = DataResource::new("test.bin".to_string(), vec![1, 2, 3, 4]);
        assert_eq!(data.get_size(), 4);
        assert_eq!(data.data(), &[1, 2, 3, 4]);
    }
}
