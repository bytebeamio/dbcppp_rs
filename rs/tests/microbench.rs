use criterion::{black_box, criterion_group, criterion_main, Criterion};
use rand::{RngCore, SeedableRng};
use dbcppp_rs::CanProcessor;
use crate::utils::SEED;

mod utils;

fn parsing_bench(c: &mut Criterion) {
    let dbc = utils::load_dbc_file("csselectronics.dbc");
    c.bench_function(
        "parsing",
        |b| b.iter(|| CanProcessor::from_dbc(black_box(dbc.as_str())).unwrap()),
    );
}

fn decoding_bench(c: &mut Criterion) {
    let dbc = utils::load_dbc_file("csselectronics.dbc");
    let processor = CanProcessor::from_dbc(dbc.as_str()).unwrap();
    let message_id = 2564485392;

    let mut r = rand::rngs::StdRng::from_seed(*SEED);
    let mut payload: [u8; 8] = Default::default();
    r.fill_bytes(&mut payload);

    c.bench_function(
        "decoding",
        |b| b.iter(|| processor.decode_frame(message_id, black_box(&payload)).unwrap()),
    );
}

criterion_group!(
    name = benches;
    config = Criterion::default();
    targets = parsing_bench, decoding_bench
);
criterion_main!(benches);
