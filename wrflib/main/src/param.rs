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
    MutableU8Buffer(Vec<u8>),
    MutableF32Buffer(Vec<f32>),
}

impl WrfParam {
    /// Borrow contents of `WrfParam::String` as `&str`.
    pub fn as_str(&self) -> &str {
        match self {
            WrfParam::String(v) => v,
            _ => panic!("WrfParam is not a String"),
        }
    }
    /// Borrow contents of `WrfParam::MutableU8Buffer` or `WrfParam::ReadOnlyU8Buffer` as `&[u8]`.
    pub fn as_u8_slice(&self) -> &[u8] {
        match self {
            WrfParam::MutableU8Buffer(v) => v,
            WrfParam::ReadOnlyU8Buffer(v) => v,
            _ => panic!("{:?} is not a U8Buffer or ReadOnlyU8Buffer", self),
        }
    }
    /// Borrow contents of `WrfParam::MutableF32Buffer` or `WrfParam::ReadOnlyF32Buffer` as `&[f32]`.
    pub fn as_f32_slice(&self) -> &[f32] {
        match self {
            WrfParam::MutableF32Buffer(v) => v,
            WrfParam::ReadOnlyF32Buffer(v) => v,
            _ => panic!("{:?} is not a F32Buffer or ReadOnlyF32Buffer", self),
        }
    }
    /// Get contents of `WrfParam::ReadOnlyU8Buffer`, without having to consume it.
    pub fn as_arc_vec_u8(&self) -> Arc<Vec<u8>> {
        match self {
            WrfParam::ReadOnlyU8Buffer(v) => Arc::clone(v),
            _ => panic!("{:?} is not a ReadOnlyU8Buffer", self),
        }
    }
    /// Get contents of `WrfParam::ReadOnlyU8Buffer`, without having to consume it.
    pub fn as_arc_vec_f32(&self) -> Arc<Vec<f32>> {
        match self {
            WrfParam::ReadOnlyF32Buffer(v) => Arc::clone(v),
            _ => panic!("{:?} is not a ReadOnlyF32Buffer", self),
        }
    }
    /// Get contents of `WrfParam::String`, consuming it.
    pub fn into_string(self) -> String {
        match self {
            WrfParam::String(v) => v,
            _ => panic!("WrfParam is not a String"),
        }
    }
    /// Get contents of `WrfParam::MutableU8Buffer`, consuming it.
    pub fn into_vec_u8(self) -> Vec<u8> {
        match self {
            WrfParam::MutableU8Buffer(v) => v,
            _ => panic!("{:?} is not a U8Buffer", self),
        }
    }
    /// Get contents of `WrfParam::MutableF32Buffer`, consuming it.
    pub fn into_vec_f32(self) -> Vec<f32> {
        match self {
            WrfParam::MutableF32Buffer(v) => v,
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
        WrfParam::MutableU8Buffer(self)
    }
}
impl IntoParam for Vec<f32> {
    fn into_param(self) -> WrfParam {
        WrfParam::MutableF32Buffer(self)
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
