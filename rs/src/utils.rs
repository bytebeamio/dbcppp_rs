#![allow(unused_macros)]
use std::ffi::{CStr, CString};
use std::fmt::Display;
use std::os::raw::c_char;
use std::ptr::null;
use anyhow::{Context, Error, Result};

pub trait TryToString {
    fn try_to_string(self) -> Result<String>;
}

impl TryToString for *const c_char {
    fn try_to_string(self) -> Result<String> {
        if self == null() {
            return Err(Error::msg("string is null"));
        }
        unsafe {
            Ok(CStr::from_ptr(self).to_str()?.to_string())
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

macro_rules! time {
    ($e:expr) => {{
        let start = std::time::Instant::now();
        let result = $e;
        println!("{:?}", start.elapsed());
        result
    }}
}

macro_rules! here {
    () => ({
        format!("{}:{}:{}", file!(), line!(), column!())
    })
}

pub trait LocationContext<T> {
    /// Wrap the error value with additional context.
    fn loc_context<C>(self, loc: String, context: C) -> Result<T>
        where
            C: Display;

    /// Wrap the error value with additional context that is evaluated lazily
    /// only once an error does occur.
    fn with_loc_context<C, F>(self, loc: String, f: F) -> Result<T>
        where
            C: Display,
            F: FnOnce() -> C;
}

impl<T> LocationContext<T> for Result<T> {
    fn loc_context<C>(self, loc: String, context: C) -> Result<T>
        where
            C: Display,
    {
        self.context(format!("{} {}", loc, context))
    }

    fn with_loc_context<C, F>(self, loc: String, context: F) -> Result<T>
        where
            C: Display,
            F: FnOnce() -> C,
    {
        self.with_context(|| format!("{} {}", loc, context()))
    }
}

