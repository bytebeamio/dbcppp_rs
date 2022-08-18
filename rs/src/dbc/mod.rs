use std::collections::HashMap;
use std::ptr::null;
use anyhow::{Context, Error, Result};
use dbcppp_rs_sys::*;
use crate::decision_tree::{create_decision_tree, Decision, MuxSignal};
use crate::TryToString;

#[derive(Debug)]
pub struct Dbc {
    messages: Vec<Message>,
}

#[derive(Debug)]
pub struct Message {
    pub id: u64,
    pub name: String,
    pub signals: Vec<Signal>,
    pub no_mux_signals: Vec<u64>,
    pub decision_tree: Vec<MuxSignal>,
}

#[derive(Debug)]
pub struct Signal {
    pub name: String,
    pub unit: String,
    pub comment: String,
    pub enum_map: HashMap<u64, String>,
    pub mux_flag: SignalMuxFlag,
    pub mux_value: u64,
    pub ex_mux_parent: Option<ExMuxInfo>,
}

#[derive(Debug, Eq, PartialEq)]
pub enum SignalMuxFlag {
    NoMux,
    Switch,
    Value,
}

#[derive(Debug)]
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
                    .with_context(|| format!("message #{idx} is invalid"))?);
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
                .with_context(|| format!("Message({id}): invalid name"))?;

            let signals_count = dbcppp_MessageSignals_Size(raw);
            let mut signals = Vec::with_capacity(signals_count as _);
            let mut no_mux_signals = Vec::with_capacity(signals_count as _);
            for idx in 0..signals_count {
                let sig = dbcppp_MessageSignals_Get(raw, idx);
                if sig == null() {
                    return Err(Error::msg(format!("signal #{idx} is null")));
                }

                let sig = Signal::new(sig)
                    .with_context(|| format!("signal #{idx} is invalid"))?;
                if sig.mux_flag == SignalMuxFlag::NoMux {
                    no_mux_signals.push(idx);
                }
                signals.push(sig);
            }

            Ok(Message {
                id,
                name,
                signals,
                no_mux_signals,
                decision_tree,
            })
        }
    }
}

impl Signal {
    pub fn new(raw: *const dbcppp_Signal) -> Result<Signal> {
        unsafe {
            let name = dbcppp_SignalName(raw)
                .try_to_string()
                .context("invalid name")?;
            let unit = dbcppp_SignalUnit(raw)
                .try_to_string()
                .context("invalid unit")?;
            let comment = dbcppp_SignalComment(raw)
                .try_to_string()
                .context("invalid comment")?;

            let mut enum_map = HashMap::new();
            for idx in 0..dbcppp_SignalValueEncodingDescriptions_Size(raw) {
                let desc = dbcppp_SignalValueEncodingDescriptions_Get(raw, idx);
                if desc == null() {
                    return Err(Error::msg(format!("invalid signal({name})")));
                }
                let value = dbcppp_ValueEncodingDescriptionValue(desc);
                let name = dbcppp_ValueEncodingDescriptionDescription(desc)
                    .try_to_string()
                    .with_context(|| format!("invalid signal({name})"))?;
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
                    .with_context(|| format!("invalid signal({name})"))?;
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
}