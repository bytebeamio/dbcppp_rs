use std::collections::HashMap;
use std::ffi::CString;
use std::ptr::null;
use dbcppp_rs_sys::*;
use anyhow::{Context, Error, Result};
use crate::utils::TryToString;

mod utils;

#[derive(Debug)]
pub struct CanProcessor {
    /// lifetime tied to this struct
    pub inner: *const dbcppp_Network,
    pub message_processors: HashMap<u64, MessageProcessor>,
}

impl CanProcessor {
    pub fn from_dbc(dbc: &str) -> Result<CanProcessor> {
        unsafe {
            CString::new(dbc)?;
            let inner = dbcppp_NetworkLoadDBCFromMemory(CString::new(dbc)?.as_ptr());
            if inner == null() {
                // TODO: extract syntax error from libdbcppp
                return Err(Error::msg("Provided dbc file is invalid"));
            }
            let mut message_processors = HashMap::new();
            for msg in (0..dbcppp_NetworkMessages_Size(inner))
                .map(|idx| dbcppp_NetworkMessages_Get(inner, idx)) {
                message_processors.insert(
                    dbcppp_MessageId(msg),
                    MessageProcessor::new(msg)
                        .context(format!("Failed to initialize processor for message: {:?} | {}", dbcppp_MessageName(msg).try_to_string(), dbcppp_MessageId(msg)))?,
                );
            }
            Ok(CanProcessor {
                inner,
                message_processors,
            })
        }
    }

    pub fn parse_frame(&self, id: u64, payload: &[u8]) -> Result<HashMap<String, CanValue>> {
        self.message_processors.get(&id)
            .ok_or(Error::msg("Invalid can id"))
            .map(|processor| processor.parse_frame(payload))
    }
}

impl Drop for CanProcessor {
    fn drop(&mut self) {
        unsafe { dbcppp_NetworkFree(self.inner); }
    }
}

#[derive(Debug)]
pub struct MessageProcessor {
    /// lifetime tied to parent struct, DO NOT COPY
    inner: *const dbcppp_Message,
    pub name: String,
    /// parser implementation identifies signals using indexes in this vector
    signals: Vec<*const dbcppp_Signal>,
    /// To be decoded unconditionally
    no_mux_signals: Vec<usize>,
    /// walk this decision tree and add all enabled signals
    top_mux_signals: Vec<MuxSignal>,
}

impl MessageProcessor {
    pub fn new(msg: *const dbcppp_Message) -> Result<MessageProcessor> {
        let name = unsafe {
            dbcppp_MessageName(msg)
        }.try_to_string()?;

        Ok(MessageProcessor {
            inner: msg,
            name
        })
    }

    pub fn parse_frame(&self, payload: &[u8]) -> HashMap<String, CanValue> {

    }
}

#[derive(Debug, Clone)]
pub struct CanValue {
    pub numeric_value: f64,
    pub unit: Option<String>,
    pub enum_repr: Option<String>,
}

#[derive(Debug)]
pub struct MuxSignal {
    /// index into the signals array of MessageProcessor owning this tree
    multiplexer_signal: usize,
    decisions: Vec<Decision>,
}

#[derive(Debug)]
pub struct Decision {
    /// inclusive range
    min_val: f64,
    max_val: f64,
    /// The signal that this decision will enable
    target_signal: MuxSignal,
}