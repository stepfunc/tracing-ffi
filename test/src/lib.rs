//! Test crate

pub mod ffi;
mod implementation;

pub(crate) use implementation::*;

lazy_static::lazy_static! {
    static ref VERSION: std::ffi::CString = std::ffi::CString::new("0.1.0").unwrap();
}

fn version() -> &'static std::ffi::CStr {
    &VERSION
}

impl From<crate::TracingInitError> for std::os::raw::c_int {
    fn from(_: crate::TracingInitError) -> Self {
        ffi::InitError::TracingInitFailed.into()
    }
}
