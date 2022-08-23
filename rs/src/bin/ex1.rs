use std::os::raw::c_char;
use std::ptr::null;
use dbcppp_rs::utils::TryToString;

fn main() {
    dbg!(null::<c_char>().try_to_string().unwrap());
}