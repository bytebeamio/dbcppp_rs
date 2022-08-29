extern crate core;

use dbcppp_rs::CanProcessor;
use crate::utils::test_error_in_file;

mod utils;

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

#[test]
fn iso8859_test() {
    let text = utils::load_dbc_file("river.dbc");
    CanProcessor::from_dbc(text.as_str()).unwrap();
}