use std::collections::HashMap;
use std::ffi::CString;
use std::ptr::null;
use dbcppp_rs_sys::*;
use anyhow::{Context, Error, Result};
use bitfield::bitfield;
use crate::dbc::Dbc;
use crate::utils::TryToString;

pub mod utils;
pub mod dbc;

#[derive(Debug)]
pub struct CanProcessor {
    /// lifetime tied to this struct
    inner: *const dbcppp_Network,
    /// This field can be used to query the schema of data that will be generated when a can frame is processed
    pub dbc: Dbc,
}

impl CanProcessor {
    pub fn from_dbc(dbc: &str) -> Result<CanProcessor> {
        let inner = unsafe {
            dbcppp_NetworkLoadDBCFromMemory(CString::new(dbc)?.as_ptr())
        };
        let dbc = Dbc::new(inner)?;
        Ok(CanProcessor {
            inner,
            dbc
        })
    }
}

// impl CanProcessor {
//     pub fn from_dbc(dbc: &str) -> Result<CanProcessor> {
//         unsafe {
//             let inner = dbcppp_NetworkLoadDBCFromMemory(CString::new(dbc)?.as_ptr());
//             if inner == null() {
//                 // TODO: extract syntax error from libdbcppp
//                 return Err(Error::msg("Provided dbc file is invalid"));
//             }
//
//             let mut message_processors = HashMap::new();
//             for msg in (0..dbcppp_NetworkMessages_Size(inner))
//                 .map(|idx| dbcppp_NetworkMessages_Get(inner, idx)) {
//                 let id = dbcppp_MessageId(msg);
//                 message_processors.insert(
//                     id,
//                     MessageProcessor::new(msg)
//                         .context(format!("Failed to initialize processor for message: {:?} | {}", dbcppp_MessageName(msg).try_to_string(), id))?,
//                 );
//             }
//
//             Ok(CanProcessor {
//                 inner,
//                 message_processors,
//             })
//         }
//     }
//
//     pub fn parse_frame(&self, id: u64, payload: &[u8]) -> Result<HashMap<String, CanValue>> {
//         self.message_processors.get(&id)
//             .ok_or(Error::msg("Invalid can id"))
//             .map(|processor| processor.parse_frame(payload))
//     }
// }
//
// impl Drop for CanProcessor {
//     fn drop(&mut self) {
//         unsafe { dbcppp_NetworkFree(self.inner); }
//     }
// }
//
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