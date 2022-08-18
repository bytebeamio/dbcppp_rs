use std::ffi::{CStr, CString};
use std::os::raw::c_char;
use anyhow::Result;

pub trait TryToString {
    fn try_to_string(&self) -> Result<String>;
}

impl TryToString for *const c_char {
    fn try_to_string(&self) -> Result<String> {
        unsafe {
            Ok(CStr::from_ptr(*self).to_str()?.to_string())
        }
    }
}

pub trait StrHelpers {
    fn try_to_cstring(&self) -> Result<CString>;
    fn ascii(&self) -> String;
}

impl StrHelpers for &str {
    fn try_to_cstring(&self) -> Result<CString> {
        Ok(CString::new(self.as_bytes())?)
    }

    fn ascii(&self) -> String {
        self.chars()
            .map(|uc| if uc.is_ascii() { uc } else { '_' })
            .collect::<String>()
    }
}
