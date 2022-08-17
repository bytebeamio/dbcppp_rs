use std::collections::HashMap;
use std::ptr::null;
use anyhow::{Result, Context, Error};
use dbcppp_rs_sys::*;
use crate::TryToString;

pub struct Dbc {
    messages: Vec<Message>,
}

impl Dbc {
    pub fn new(raw: *const dbcppp_Network) -> Result<Dbc> {
        unsafe {
            let mut messages = Vec::new();
            for msg in (0..dbcppp_NetworkMessages_Size(raw))
                .map(|idx| dbcppp_NetworkMessages_Get(raw, idx)) {
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

pub struct Message {
    pub id: u64,
    pub name: String,
    pub signals: Vec<Signal>,
}

impl Message {
    pub fn new(raw: *const dbcppp_Message) -> Result<Message> {
        unsafe {
            let id = dbcppp_MessageId(raw);
            let name = dbcppp_MessageName(raw).try_to_string()
                .with_context(|| format!("Message({id}): invalid name"))?;

            let signals_count = dbcppp_MessageSignals_Size(raw);
            let mut signals = Vec::with_capacity(signals_count as _);
            for idx in 0..signals_count {
                let sig = dbcppp_MessageSignals_Get(raw, idx);
                if sig == null() {
                    return Err(Error::msg(format!("signal #{idx} is null")));
                }

                signals.push(Signal::new(sig)
                    .with_context(|| format!("signal #{idx} is invalid"))?);
            }
            Ok(Message {
                id,
                name,
                signals
            })
        }
    }
}

pub enum SignalMuxFlag {
    NoMux,
    MuxSwitch,
    MuxValue,
}

pub struct ExMuxInfo {
    pub switch: String,
    /// inclusive range
    pub min_val: u64,
    pub max_val: u64,
}

pub struct Signal {
    pub name: String,
    pub unit: String,
    pub comment: String,
    pub enum_map: HashMap<u64, String>,
    pub mux_flag: SignalMuxFlag,
    pub mux_value: u64,
    pub ex_mux_parent: Option<ExMuxInfo>,
}

impl Signal {
    pub fn new(raw: *const dbcppp_Signal) -> Result<Signal> {
        unsafe {
            let name = dbcppp_SignalName(raw)
                .try_to_string()
                .context("invalid name")?;
            let unit = dbcppp_SignalUnit(raw)
                .try_to_string()
                .context("invalid name")?;
            let comment = dbcppp_SignalComment(raw)
                .try_to_string()
                .context("invalid name")?;

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
                1 => SignalMuxFlag::MuxSwitch,
                2 => SignalMuxFlag::MuxValue,
                _ => return Err(Error::msg(format!("invalid signal({name})")))
            };

            let mux_value = dbcppp_SignalMultiplexerSwitchValue(raw);

            let ex_mux_count = dbcppp_SignalMultiplexerValues_Size(raw);
            let ex_mux_parent = if ex_mux_count > 1 {
                return Err(Error::msg("signal has more than one extended multiplexer parents"));
            } else if ex_mux_count == 1 {
                let mux_parent = dbcppp_SignalMultiplexerValues_Get(raw, 0);
                Some(ExMuxInfo {
                    switch: "".to_string(),
                    min_val: 0,
                    max_val: 0
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
                ex_mux_parent
            })
        }
    }
}