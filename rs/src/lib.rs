use std::collections::HashMap;
use std::ffi::CString;
use std::ptr::null;
use dbcppp_rs_sys::*;
use anyhow::{Context, Error, Result};
use bitfield::bitfield;
use crate::dbc::{Dbc, Message, Signal, SignalMuxFlag};
use crate::decision_tree::create_decision_tree;
use crate::utils::TryToString;

pub mod utils;
pub mod dbc;
pub mod decision_tree;

pub struct CanProcessor {
    /// lifetime tied to this struct
    inner: *const dbcppp_Network,
    /// This field can be used to query the schema of data that will be generated when a can frame is processed
    pub dbc: Dbc,
    message_processors: HashMap<u64, MessageProcessor>,
}

impl CanProcessor {
    /// Parses the provided dbc file and prepares to receive frames for decoding
    /// The only dbcppp method that will be called after the initialization is `dbcppp_SignalDecode`
    pub fn from_dbc(dbc: &str) -> Result<CanProcessor> {
        let inner = unsafe {
            dbcppp_NetworkLoadDBCFromMemory(CString::new(dbc)?.as_ptr())
        };
        let dbc = Dbc::new(inner)?;

        let mut message_processors = HashMap::new();
        for msg in dbc.messages {
            message_processors.insert(
                msg.id,
                MessageProcessor::new(msg)
                    .context(format!("Failed to initialize processor for message: {:?} | {}", msg.name, msg.id))?,
            );
        }

        Ok(CanProcessor {
            inner,
            dbc,
            message_processors,
        })
    }

    pub fn decode_frame(&self, id: u64, payload: &[u8]) -> Result<HashMap<String, CanValue>> {
        let msg = self.message_processors.get(&id)
            .ok_or(Error::msg("Invalid can id"))?;
        msg.parse_frame(payload)
    }
}

impl Drop for CanProcessor {
    fn drop(&mut self) {
        unsafe { dbcppp_NetworkFree(self.inner); }
    }
}

pub struct MessageProcessor {
    inner: Message,
}

impl MessageProcessor {
    pub fn new(inner: Message) -> Result<MessageProcessor> {
        let decision_tree = create_decision_tree(&inner);
        Ok(MessageProcessor {
            inner
        })
    }

    pub fn parse_frame(&self, payload: &[u8]) -> Result<HashMap<String, CanValue>> {
        if self.inner.payload_size > payload.len() as _ {
            return Err(Error::msg(format!("payload size ({}) is smaller than the message size ({})", payload.len(), self.inner.payload_size)));
        }

        let mux_sig = self.inner.mux_sig();

        let mut result = HashMap::new();
        for sig in self.inner.signals.iter() {
            if sig.mux_flag != SignalMuxFlag::Value {
                result.insert(sig.name.clone(), self.decode_signal(sig, payload));
            } else if mux_sig.is_some() && self.decode_signal(mux_sig.unwrap(), payload) == sig.mux_value {
                result.insert(sig.name.clone(), self.decode_signal(sig, payload));
            } else {
                // TODO: decode ex mux
            }
        }
        Ok(result)
    }

    fn decode_signal(&self, sig: &Signal, payload: &[u8]) -> CanValue {
        0
    }
}

type CanValue = u64;

// #[derive(Debug, Clone, Eq, PartialEq)]
// pub struct CanValue<'a> {
//     pub numeric_value: f64,
//     pub unit: Option<&'a str>,
//     pub enum_repr: Option<&'a str>,
// }

// #[derive(Debug)]
// pub struct MessageProcessor {
//     /// lifetime tied to parent struct, DO NOT COPY
//     inner: *const dbcppp_Message,
//     pub name: String,
//     /// parser implementation identifies signals using indexes in these vectors
//     signals: Vec<*const dbcppp_Signal>,
//     signal_names: Vec<String>,
//     signal_units: Vec<String>,
//     signal_comments: Vec<String>,
//     /// To be decoded unconditionally
//     no_mux_signals: Vec<usize>,
//     /// walk this decision tree and add all enabled signals
//     top_mux_signals: Vec<MuxSignal>,
// }
//
// impl MessageProcessor {
//     pub unsafe fn new(msg: *const dbcppp_Message) -> Result<MessageProcessor> {
//         let name = dbcppp_MessageName(msg).try_to_string()?;
//
//         let signals_count = dbcppp_MessageSignals_Size(msg);
//         let mut signals = Vec::with_capacity(signals_count as usize);
//         let mut signal_multiplexers = Vec::with_capacity(signals_count as usize);
//         let mut signals_revindex = HashMap::new();
//         let mut no_mux_signals = Vec::new();
//         for idx in 0..signals_count {
//             let sig = dbcppp_MessageSignals_Get(msg, idx);
//             let name = dbcppp_SignalName(sig).try_to_string()
//                 .context(format!("signal #{idx} in message {:?} is malformed", name))?;
//             signals.push(sig);
//             signals_revindex.insert(name, idx);
//         }
//
//         Ok(MessageProcessor {
//             inner: msg,
//             name,
//         })
//     }
//
//     pub fn parse_frame(&self, payload: &[u8]) -> HashMap<String, CanValue> {
//
//     }
// }
//
// #[derive(Debug, Clone)]
// pub struct CanValue<'a> {
//     pub numeric_value: f64,
//     pub unit: Option<&'a str>,
//     pub enum_repr: Option<&'a str>,
// }
//
// #[derive(Debug)]
// pub struct MuxSignal {
//     /// index into the signals array of MessageProcessor owning this tree
//     multiplexer_signal: usize,
//     decisions: Vec<Decision>,
// }
//
// #[derive(Debug)]
// pub struct Decision {
//     /// inclusive range
//     min_val: f64,
//     max_val: f64,
//     /// The signal that this decision will enable
//     target_signal: MuxSignal,
// }
//
// #[derive(Debug)]
// pub struct SignalProcessor {
//     inner: *const dbcppp_Signal,
// }
//
// impl SignalProcessor {
//     pub unsafe fn new(sig: *const dbcppp_Signal) -> Result<SignalProcessor> {
//         let name = dbcppp_SignalName(sig).try_to_string()?;
//         Ok(SignalProcessor {
//             inner: sig,
//         })
//     }
// }
