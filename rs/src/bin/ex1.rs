use std::ffi::{CStr, CString};
use std::os::raw::c_char;
use std::ptr::null;
use dbcppp_rs::CanProcessor;
use dbcppp_rs::dbc::Dbc;
use dbcppp_rs_sys::*;
use dbcppp_rs::utils::StrHelpers;

fn str_to_cstring(s: &str) -> CString {
    CString::new(s).unwrap()
}

fn chars_to_string(ptr: *const c_char) -> String {
    unsafe { CStr::from_ptr(ptr).to_str().unwrap().to_string() }
}

fn main() {
    unsafe {
        f();
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

unsafe fn f() {
    let data = std::fs::read_to_string("./my.tmp.dir/test.dbc").unwrap().as_str().ascii();
    let dbc = CanProcessor::from_dbc(data.as_str()).unwrap();
    dbg!(dbc);
}