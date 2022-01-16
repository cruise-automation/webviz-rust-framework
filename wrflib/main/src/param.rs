use std::sync::Arc;

// WrfParam types that can come back from JavaScript
// Keep in sync with WrfParamType in types.ts
// TODO(Paras): This could be cleaner as an enum, but casting between u32s and enums is a bit annoying.
#[cfg(any(target_arch = "wasm32", feature = "cef"))]
pub(crate) const WRF_PARAM_STRING: u32 = 0;
#[cfg(any(target_arch = "wasm32", feature = "cef"))]
pub(crate) const WRF_PARAM_READ_ONLY_UINT8_BUFFER: u32 = 1;
#[cfg(any(target_arch = "wasm32", feature = "cef"))]
pub(crate) const WRF_PARAM_UINT8_BUFFER: u32 = 2;
#[cfg(any(target_arch = "wasm32", feature = "cef"))]
pub(crate) const WRF_PARAM_FLOAT32_BUFFER: u32 = 3;
#[cfg(any(target_arch = "wasm32", feature = "cef"))]
pub(crate) const WRF_PARAM_READ_ONLY_FLOAT32_BUFFER: u32 = 4;

#[derive(Clone, Debug, PartialEq)]
pub enum WrfParam {
    /// An arbitrary string supplied by the user (e.g. JSON encoded).
    /// TODO(Paras): I wish I could just put references here, since we end up cloning the string anyways when
    /// calling zerde. But then we have to declare many lifetimes - maybe worth it.
    String(String),
    /// Buffers to pass read-only memory from JS to Rust
    ReadOnlyU8Buffer(Arc<Vec<u8>>),
    ReadOnlyF32Buffer(Arc<Vec<f32>>),
    /// Buffers to transfer ownership of memory from JS to Rust
    U8Buffer(Vec<u8>),
    F32Buffer(Vec<f32>),
}

impl WrfParam {
    pub fn as_string(&self) -> &str {
        match self {
            WrfParam::String(v) => v,
            _ => panic!("WrfParam is not a String"),
        }
    }
    pub fn as_read_only_u8_buffer(&self) -> &Arc<Vec<u8>> {
        match self {
            WrfParam::ReadOnlyU8Buffer(v) => v,
            _ => panic!("{:?} is not a ReadOnlyU8Buffer", self),
        }
    }
    pub fn as_u8_buffer(&self) -> &Vec<u8> {
        match self {
            WrfParam::U8Buffer(v) => v,
            _ => panic!("{:?} is not a U8Buffer", self),
        }
    }
    pub fn as_read_only_f32_buffer(&self) -> &Arc<Vec<f32>> {
        match self {
            WrfParam::ReadOnlyF32Buffer(v) => v,
            _ => panic!("{:?} is not a ReadOnlyF32Buffer", self),
        }
    }
    pub fn as_f32_buffer(&self) -> &Vec<f32> {
        match self {
            WrfParam::F32Buffer(v) => v,
            _ => panic!("{:?} is not a F32Buffer", self),
        }
    }
}

pub trait IntoParam {
    fn into_param(self) -> WrfParam;
}

impl IntoParam for String {
    fn into_param(self) -> WrfParam {
        WrfParam::String(self)
    }
}
impl IntoParam for Vec<u8> {
    fn into_param(self) -> WrfParam {
        WrfParam::U8Buffer(self)
    }
}
impl IntoParam for Vec<f32> {
    fn into_param(self) -> WrfParam {
        WrfParam::F32Buffer(self)
    }
}
impl IntoParam for Arc<Vec<u8>> {
    fn into_param(self) -> WrfParam {
        WrfParam::ReadOnlyU8Buffer(self)
    }
}
impl IntoParam for Arc<Vec<f32>> {
    fn into_param(self) -> WrfParam {
        WrfParam::ReadOnlyF32Buffer(self)
    }
}
