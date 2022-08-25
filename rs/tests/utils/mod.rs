#![allow(unused)]
use anyhow::Context;
use rand::RngCore;
use rand::rngs::StdRng;
use dbcppp_rs::CanProcessor;

pub fn load_dbc_file(name: &str) -> String {
    let path = format!("tests/dbcs/{name}");
    let data = std::fs::read(path.as_str()).unwrap();
    dbcppp_rs::load_dbc_file(data.as_slice())
        .context(path)
        .unwrap()
}

lazy_static::lazy_static! {
    pub static ref SEED: [u8; 32] = u64_to_seed(0xcafebabe);
}

pub fn u64_to_seed(n: u64) -> [u8; 32] {
    let mut res = [0u8; 32];
    let mut u64_view = unsafe { std::slice::from_raw_parts_mut(res.as_mut_ptr() as *mut u64, 4) };
    u64_view[0] = n;
    res
}

pub trait RngHelper {
    fn next_u8(&mut self) -> u8;
}

impl RngHelper for StdRng {
    fn next_u8(&mut self) -> u8 {
        (self.next_u32() & 0xff) as u8
    }
}

pub fn sequential<T: Ord>(n1: T, n2: T, n3: T) -> bool {
    n2 >= n1 && n3 >= n2
}

pub fn test_error_in_file(file: &str, error_text: &str) {
    let text = load_dbc_file(file);
    match CanProcessor::from_dbc(text.as_str()) {
        Ok(dbc) => {
            panic!("should've gotten an error, instead got following result:\n{:#?}", dbc);
        }
        Err(e) => {
            if !format!("{:?}", e).contains(error_text) {
                panic!("Wrong error:\n{:?}", e);
            }
        }
    }
}

