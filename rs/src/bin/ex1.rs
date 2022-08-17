use std::ffi::{CStr, CString};
use std::os::raw::c_char;
use std::ptr::null;
use dbcppp_rs_sys::*;

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

unsafe fn f() {
    let dbc = dbcppp_NetworkLoadDBCFromFile(
        str_to_cstring("./my.tmp.dir/test.dbc").as_ptr(),
    );
    if dbc == null() {
        println!("invalid dbc file");
        return;
    }
    for msg in (0..dbcppp_NetworkMessages_Size(dbc))
        .map(|idx| dbcppp_NetworkMessages_Get(dbc, idx)) {
        println!("{}: {}", chars_to_string(dbcppp_MessageName(msg)), dbcppp_MessageId(msg));
    }
}