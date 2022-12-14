extern crate core;

mod utils;

use std::collections::HashMap;
use rand::SeedableRng;
use dbcppp_rs::CanProcessor;
use crate::utils::{load_dbc_file, RngHelper, sequential, u64_to_seed};

#[test]
fn extended_multiplexing() {
    let r0 = rand::random::<u64>();
    let mut r = rand::rngs::StdRng::from_seed(u64_to_seed(r0));
    let processor = CanProcessor::from_dbc(load_dbc_file("test1.dbc").as_str()).unwrap();
    let message_id = 2348873389;
    for idx in 0..100000 {
        let sig1 = r.next_u8();
        let sig2 = r.next_u8();
        let sig3 = r.next_u8();
        let sig4 = r.next_u8();
        let sig5 = r.next_u8();
        let payload = [
            sig1,
            sig2,
            sig3,
            sig4,
            sig5,
            0,
            0,
            0,
        ];
        let mut expected = HashMap::new();
        expected.insert("New_Signal_1", sig1 as u64);
        if sequential(1, sig1, 176) {
            expected.insert("New_Signal_2", sig2 as u64);
            if sequential(10, sig2, 100) {
                expected.insert("New_Signal_3", sig3 as u64);
                if sequential(0, sig3, 3) {
                    expected.insert("New_Signal_4", sig4 as u64);
                }
                if sequential(0, sig3, 5) {
                    expected.insert("New_Signal_5", sig5 as u64);
                }
            }
        }
        let result = processor.decode_frame(message_id, &payload).unwrap();
        assert_eq!(result.message_name, "New_Message_1");
        let signals = result.signals.iter()
            .map(|(&k, v)| (k, v.raw))
            .collect::<HashMap<_, _>>();
        assert_eq!(expected, signals, "seed: {}\nindex: {}\npayload: {:?}", r0, idx, payload);
    }
}

#[test]
fn simple_multiplexing() {
    let r0 = rand::random::<u64>();
    let mut r = rand::rngs::StdRng::from_seed(u64_to_seed(r0));
    let processor = CanProcessor::from_dbc(load_dbc_file("test1.dbc").as_str()).unwrap();
    let message_id = 2348941054;
    for idx in 0..100000 {
        let sig1 = r.next_u8();
        let sig2 = r.next_u8();
        let sig3 = r.next_u8();
        let sig4 = r.next_u8();
        let sig5 = r.next_u8();
        let payload = [
            sig1,
            sig2,
            sig3,
            sig4,
            sig5,
            0,
            0,
            0,
        ];
        let mut expected = HashMap::new();
        expected.insert("New_Signal_3", sig3 as u64);
        expected.insert("New_Signal_4", sig4 as u64);
        expected.insert("New_Signal_5", sig5 as u64);
        if sig3 == 66 {
            expected.insert("New_Signal_1", sig1 as u64);
        }
        if sig3 == 0 {
            expected.insert("New_Signal_2", sig2 as u64);
        }

        let result = processor.decode_frame(message_id, &payload).unwrap();
        assert_eq!(result.message_name, "New_Message_2");
        let signals = result.signals.iter()
            .map(|(&k, v)| (k, v.raw))
            .collect::<HashMap<_, _>>();
        assert_eq!(expected, signals, "seed: {}\nindex: {}\npayload: {:?}", r0, idx, payload);
    }
}
