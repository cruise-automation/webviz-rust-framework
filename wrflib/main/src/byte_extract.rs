//! Read the desired data type in as few instructions as possible. This is technically
//! unsafe if you're reading past the end of a buffer, but we're not going to worry
//! about that for performance. This saves a relatively expensive boundary
//! check when using safe functions like `from_le_bytes` with `try_into().unwrap()`.
//!
//! TODO(JP): see if we can implement this with generics; might be tricky since you
//! can only specialize on traits. Or might be able to use code generation here?

#[inline]
pub fn extract_i8_le_fast(data: &[u8], offset: u32) -> i8 {
    unsafe { *(data.as_ptr().add(offset as usize) as *const i8) }
}
#[inline]
pub fn extract_u8_le_fast(data: &[u8], offset: u32) -> u8 {
    unsafe { *(data.as_ptr().add(offset as usize) as *const u8) }
}
#[inline]
pub fn extract_i16_le_fast(data: &[u8], offset: u32) -> i16 {
    unsafe { i16::from_le(*(data.as_ptr().add(offset as usize) as *const i16)) }
}
#[inline]
pub fn extract_u16_le_fast(data: &[u8], offset: u32) -> u16 {
    unsafe { u16::from_le(*(data.as_ptr().add(offset as usize) as *const u16)) }
}
#[inline]
pub fn extract_i32_le_fast(data: &[u8], offset: u32) -> i32 {
    unsafe { i32::from_le(*(data.as_ptr().add(offset as usize) as *const i32)) }
}
#[inline]
pub fn extract_u32_le_fast(data: &[u8], offset: u32) -> u32 {
    unsafe { u32::from_le(*(data.as_ptr().add(offset as usize) as *const u32)) }
}
#[inline]
pub fn extract_i64_le_fast(data: &[u8], offset: u32) -> i64 {
    unsafe { i64::from_le(*(data.as_ptr().add(offset as usize) as *const i64)) }
}
#[inline]
pub fn extract_u64_le_fast(data: &[u8], offset: u32) -> u64 {
    unsafe { u64::from_le(*(data.as_ptr().add(offset as usize) as *const u64)) }
}
#[inline]
pub fn extract_f32_le_fast(data: &[u8], offset: u32) -> f32 {
    unsafe { f32::from_bits(u32::from_le(*(data.as_ptr().add(offset as usize) as *const u32))) }
}
#[inline]
pub fn extract_f64_le_fast(data: &[u8], offset: u32) -> f64 {
    unsafe { f64::from_bits(u64::from_le(*(data.as_ptr().add(offset as usize) as *const u64))) }
}

// Cast to f32; common for 3d rendering.
#[inline]
pub fn extract_i8_le_fast_as_f32(data: &[u8], offset: u32) -> f32 {
    extract_i8_le_fast(data, offset) as f32
}
#[inline]
pub fn extract_u8_le_fast_as_f32(data: &[u8], offset: u32) -> f32 {
    extract_u8_le_fast(data, offset) as f32
}
#[inline]
pub fn extract_i16_le_fast_as_f32(data: &[u8], offset: u32) -> f32 {
    extract_i16_le_fast(data, offset) as f32
}
#[inline]
pub fn extract_u16_le_fast_as_f32(data: &[u8], offset: u32) -> f32 {
    extract_u16_le_fast(data, offset) as f32
}
#[inline]
pub fn extract_i32_le_fast_as_f32(data: &[u8], offset: u32) -> f32 {
    extract_i32_le_fast(data, offset) as f32
}
#[inline]
pub fn extract_u32_le_fast_as_f32(data: &[u8], offset: u32) -> f32 {
    extract_u32_le_fast(data, offset) as f32
}
#[inline]
pub fn extract_i64_le_fast_as_f32(data: &[u8], offset: u32) -> f32 {
    extract_i64_le_fast(data, offset) as f32
}
#[inline]
pub fn extract_u64_le_fast_as_f32(data: &[u8], offset: u32) -> f32 {
    extract_u64_le_fast(data, offset) as f32
}
#[inline]
pub fn extract_f32_le_fast_as_f32(data: &[u8], offset: u32) -> f32 {
    extract_f32_le_fast(data, offset) as f32
}
#[inline]
pub fn extract_f64_le_fast_as_f32(data: &[u8], offset: u32) -> f32 {
    extract_f64_le_fast(data, offset) as f32
}
