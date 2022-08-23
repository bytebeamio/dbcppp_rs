use std::fs::read;
use dbcppp_rs::{CanProcessor, load_dbc_file};
use dbcppp_rs::utils::StrHelpers;

fn main() {
    let dbc = CanProcessor::from_dbc(
        load_dbc_file(read("rs/tests/dbcs/cycles.dbc").unwrap().as_slice()).unwrap().as_str()
    ).unwrap();

}