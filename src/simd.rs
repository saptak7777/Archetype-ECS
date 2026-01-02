//! SIMD chunk iteration for numerical components

/// Returns SIMD chunk size for x86_64, single-element chunks otherwise
#[cfg(target_arch = "x86_64")]
pub fn chunk_size<T>() -> usize {
    8  // AVX2: 256-bit / 32-bit = 8 elements
}

/// Returns SIMD chunk size for x86_64, single-element chunks otherwise
#[cfg(not(target_arch = "x86_64"))]
pub fn chunk_size<T>() -> usize {
    1  // Fallback: scalar
}

/// Split data into SIMD-sized chunks for x86_64
#[cfg(target_arch = "x86_64")]
pub fn chunks<T: Copy>(data: &mut [T]) -> Vec<&mut [T]> {
    data.chunks_exact_mut(8).collect()
}

/// Split data into SIMD-sized chunks for fallback platforms
#[cfg(not(target_arch = "x86_64"))]
pub fn chunks<T: Copy>(data: &mut [T]) -> Vec<&mut [T]> {
    data.chunks_exact_mut(1).collect()
}
