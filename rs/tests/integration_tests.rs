extern crate core;

mod utils;

use std::collections::HashMap;
use rand::{RngCore, SeedableRng};
use dbcppp_rs::CanProcessor;
use crate::utils::{load_dbc_file, RngHelper, SEED, sequential};

#[test]
fn extended_multiplexing() {
    let mut r = rand::rngs::StdRng::from_seed(SEED);
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
        assert_eq!(expected, signals, "index: {}\npayload: {:?}", idx, payload);
    }
}

#[test]
fn simple_multiplexing() {
    let mut r = rand::rngs::StdRng::from_seed(SEED);
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
        assert_eq!(expected, signals, "index: {}\npayload: {:?}", idx, payload);
    }
}

fn test_error_in_file(file: &str, error_text: &str) {
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

#[test]
fn extended_multiplexing_cycles_1() {
    test_error_in_file("cycles1.dbc", "found cycle in extended multiplexing specification");
}

#[test]
fn extended_multiplexing_cycles_2() {
    test_error_in_file("cycles2.dbc", "found cycle in extended multiplexing specification");
}

#[test]
fn unknown_mux_switch() {
    test_error_in_file("mux_invalid_switch.dbc", "Signal(New_Signal_4) has an invalid multiplexer switch: \"Unknown_Signal\"");
}

