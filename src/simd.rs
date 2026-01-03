//! SIMD chunk iteration for numerical components

/// Returns optimal SIMD chunk size for type T
/// Target: 256-bit AVX2 registers (32 bytes)
pub const fn chunk_size<T>() -> usize {
    let type_size = std::mem::size_of::<T>();
    if type_size == 0 {
        return 1; // ZST
    }

    // Target 256-bit (32 bytes)
    let target_bytes = 32;
    if target_bytes % type_size == 0 {
        target_bytes / type_size
    } else {
        // Fallback for types that don't fit perfectly into 256 bits
        1
    }
}

/// Split data into SIMD-sized chunks
pub fn chunks<T>(data: &mut [T]) -> std::slice::ChunksExactMut<'_, T> {
    let size = chunk_size::<T>();
    data.chunks_exact_mut(size)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_chunk_sizes() {
        assert_eq!(chunk_size::<f32>(), 8); // 4 bytes * 8 = 32 bytes
        assert_eq!(chunk_size::<f64>(), 4); // 8 bytes * 4 = 32 bytes
        assert_eq!(chunk_size::<u8>(), 32); // 1 byte * 32 = 32 bytes
        assert_eq!(chunk_size::<u32>(), 8); // 4 bytes * 8 = 32 bytes
        assert_eq!(chunk_size::<()>(), 1); // ZST
    }
}
