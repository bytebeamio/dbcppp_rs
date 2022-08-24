use std::collections::HashMap;
use std::ptr::null;
use anyhow::{Error, Result};
use dbcppp_rs_sys::*;
use crate::dbc::cycles::find_cycle;
use crate::{LocationContext, TryToString};

mod cycles;

#[derive(Debug, Clone)]
pub struct Dbc {
    pub messages: Vec<Message>,
}

#[derive(Debug, Clone)]
pub struct Message {
    pub id: u64,
    pub name: String,
    pub signals: Vec<Signal>,
    pub payload_size: u64,
    pub mux_sig: Option<Signal>,
    pub is_ex_mux: bool,
}

#[derive(Debug, Clone)]
pub struct Signal {
    pub raw: *const dbcppp_Signal,
    pub name: String,
    pub unit: String,
    pub comment: String,
    pub enum_map: HashMap<u64, String>,
    pub mux_flag: SignalMuxFlag,
    pub mux_value: u64,
    pub ex_mux_parent: Option<ExMuxInfo>,
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub enum SignalMuxFlag {
    NoMux,
    Switch,
    Value,
}

#[derive(Debug, Clone)]
pub struct ExMuxInfo {
    pub switch: String,
    pub ranges: Vec<dbcppp_ValueRange>,
}

impl Dbc {
    pub fn new(raw: *const dbcppp_Network) -> Result<Dbc> {
        unsafe {
            let mut messages = Vec::new();
            for idx in 0..dbcppp_NetworkMessages_Size(raw) {
                let msg = dbcppp_NetworkMessages_Get(raw, idx);
                if msg == null() {
                    return Err(Error::msg(format!("message #{idx} is null")));
                }

                messages.push(Message::new(msg)
                    .with_loc_context(here!(), || format!("message #{idx} is invalid"))?);
            }

            Ok(Dbc {
                messages
            })
        }
    }
}


impl Message {
    pub fn new(raw: *const dbcppp_Message) -> Result<Message> {
        unsafe {
            let id = dbcppp_MessageId(raw);
            let name = dbcppp_MessageName(raw).try_to_string()
                .with_loc_context(here!(), || format!("Message({id}): invalid name"))?;
            let payload_size = dbcppp_MessageMessageSize(raw);

            let mut mux_sig = None;
            let mut has_muxed_sigs = false;

            let signals_count = dbcppp_MessageSignals_Size(raw);
            let mut signals = Vec::with_capacity(signals_count as _);
            for idx in 0..signals_count {
                let sig = dbcppp_MessageSignals_Get(raw, idx);
                if sig == null() {
                    return Err(Error::msg(format!("signal #{idx} is null")));
                }

                let sig = Signal::new(sig)
                    .with_loc_context(here!(), || format!("signal #{idx} is invalid"))?;

                match sig.mux_flag {
                    SignalMuxFlag::Switch => {
                        mux_sig = Some(sig.clone());
                    }
                    SignalMuxFlag::Value => {
                        has_muxed_sigs = true;
                    }
                    SignalMuxFlag::NoMux => {}
                }
                if sig.mux_flag == SignalMuxFlag::Switch {
                    mux_sig = Some(sig.clone());
                }

                signals.push(sig);
            }

            for sig in signals.iter() {
                if let Some(ExMuxInfo { switch, .. }) = sig.ex_mux_parent.as_ref() {
                    if signals.iter().find(|s| &s.name == switch).is_none() {
                        return Err(Error::msg(format!("Signal({}) has an invalid multiplexer switch: {:?}", sig.name, switch)));
                    }
                }
            }

            let is_ex_mux = signals.iter().find(|sig| sig.ex_mux_parent.is_some()).is_some();

            if !is_ex_mux && has_muxed_sigs && mux_sig.is_none() {
                return Err(Error::msg(format!("Message({name}) has no extended multiplexing info and no multiplexor signal for multiplexed signals")));
            }

            if let Some(cycle) = find_cycle(signals.as_slice()) {
                return Err(Error::msg(format!("found cycle in extended multiplexing specification: {:?}", cycle)));
            }

            Ok(Message {
                id,
                name,
                payload_size,
                signals,
                mux_sig,
                is_ex_mux,
            })
        }
    }
}

impl Signal {
    pub fn new(raw: *const dbcppp_Signal) -> Result<Signal> {
        unsafe {
            let name = dbcppp_SignalName(raw)
                .try_to_string()
                .loc_context(here!(), "invalid name")?;
            let unit = dbcppp_SignalUnit(raw)
                .try_to_string()
                .loc_context(here!(), "invalid unit")?;
            let comment = dbcppp_SignalComment(raw)
                .try_to_string()
                .loc_context(here!(), "invalid comment")?;

            let mut enum_map = HashMap::new();
            for idx in 0..dbcppp_SignalValueEncodingDescriptions_Size(raw) {
                let desc = dbcppp_SignalValueEncodingDescriptions_Get(raw, idx);
                if desc == null() {
                    return Err(Error::msg(format!("invalid signal({name})")));
                }
                let value = dbcppp_ValueEncodingDescriptionValue(desc);
                let name = dbcppp_ValueEncodingDescriptionDescription(desc)
                    .try_to_string()
                    .with_loc_context(here!(), || format!("invalid signal({name})"))?;
                enum_map.insert(value, name);
            }

            let mux_flag = match dbcppp_SignalMultiplexerIndicator(raw) {
                0 => SignalMuxFlag::NoMux,
                1 => SignalMuxFlag::Switch,
                2 => SignalMuxFlag::Value,
                _ => return Err(Error::msg(format!("invalid signal({name})")))
            };

            let mux_value = dbcppp_SignalMultiplexerSwitchValue(raw);

            let ex_mux_count = dbcppp_SignalMultiplexerValues_Size(raw);
            let ex_mux_parent = if ex_mux_count > 1 {
                return Err(Error::msg("signal has more than one extended multiplexer parents"));
            } else if ex_mux_count == 1 {
                let mux_parent = dbcppp_SignalMultiplexerValues_Get(raw, 0);
                let switch = dbcppp_SignalMultiplexerValue_SwitchName(mux_parent).try_to_string()
                    .with_loc_context(here!(), || format!("invalid signal({name})"))?;
                let ranges_count = dbcppp_SignalMultiplexerValue_ValueRanges_Size(mux_parent);
                let mut ranges = Vec::with_capacity(ranges_count as _);
                for idx in 0..ranges_count {
                    let range_ptr = dbcppp_SignalMultiplexerValue_ValueRanges_Get(mux_parent, idx);
                    if range_ptr == null() {
                        return Err(Error::msg(format!("invalid signal({name})")));
                    }
                    ranges.push((&*range_ptr).clone());
                }
                Some(ExMuxInfo {
                    switch,
                    ranges,
                })
            } else {
                None
            };

            Ok(Signal {
                raw,
                name,
                unit,
                comment,
                enum_map,
                mux_flag,
                mux_value,
                ex_mux_parent,
            })
        }
    }

    pub fn decode_raw(&self, payload: &[u8]) -> u64 {
        unsafe {
            dbcppp_SignalDecode(self.raw, payload.as_ptr() as _)
        }
    }

    pub fn raw_to_phys(&self, raw: u64) -> f64 {
        unsafe {
            dbcppp_SignalRawToPhys(self.raw, raw)
        }
    }
}