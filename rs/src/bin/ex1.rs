use dbcppp_rs::CanProcessor;
use dbcppp_rs::utils::StrHelpers;

fn main() {
    unsafe {
        f();
    }
}

unsafe fn f() {
    let data = std::fs::read_to_string("./my.tmp.dir/test.dbc").unwrap().as_str().ascii();
    let dbc = CanProcessor::from_dbc(data.as_str()).unwrap();
    dbg!(dbc.decode_frame(2348873389, vec![
        0x01,
        0x01,
        0x02,
        0x42,
        0x9e,
        0x00,
        0x00,
        0x00,
    ].as_slice()).unwrap());
}