use dbcppp_rs::{CanProcessor, load_dbc_file};

fn main() {
    let p = CanProcessor::from_dbc(load_dbc_file(std::fs::read("rs/tests/dbcs/river.dbc").unwrap().as_slice()).unwrap().as_str()).unwrap();
    dbg!(p.schema());
}