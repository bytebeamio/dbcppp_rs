use anyhow::Context;
use rand::rngs::StdRng;
use crate::RngCore;

pub fn load_dbc_file(name: &str) -> String {
    let path = format!("tests/dbcs/{name}");
    let data = std::fs::read(path.as_str()).unwrap();
    dbcppp_rs::load_dbc_file(data.as_slice())
        .context(path)
        .unwrap()
}

const fn gen_seed() -> [u8; 32] {
    let mut res = [0u8; 32];
    res[0] = 0xca;
    res[1] = 0xfe;
    res[2] = 0xba;
    res[3] = 0xbe;
    res
}

pub const SEED: [u8; 32] = gen_seed();

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