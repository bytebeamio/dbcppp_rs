use std::collections::HashMap;
use std::ffi::CString;
use dbcppp_rs_sys::*;
use anyhow::{Context, Error, Result};
use crate::dbc::{Dbc, Message, Signal, SignalMuxFlag};
use crate::message_processor::MessageProcessor;
use crate::utils::TryToString;

pub mod utils;
pub mod dbc;
pub mod message_processor;
pub mod decision_tree;

pub const BOTTOM_29_BITS: u64 = 0x1fffffff;

pub struct CanProcessor {
    /// lifetime tied to this struct
    inner: *const dbcppp_Network,
    dbc: Dbc,
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
        for msg in dbc.messages.iter() {
            message_processors.insert(
                msg.id & BOTTOM_29_BITS,
                MessageProcessor::new(msg.clone())
                    .context(format!("Failed to initialize processor for message: {:?} | {}", msg.name, msg.id))?,
            );
        }

        Ok(CanProcessor {
            inner,
            dbc,
            message_processors,
        })
    }

    pub fn decode_frame(&self, id: u64, payload: &[u8]) -> Result<CanResult> {
        let msg = self.message_processors.get(&(id & BOTTOM_29_BITS))
            .ok_or(Error::msg("Invalid can id"))?;
        Ok(msg.parse_frame(payload)?)
    }

    /// You can use it to query the schema of data that will be returned when a CAN frame is parsed
    pub fn schema(&self) -> &Dbc {
        &self.dbc
    }
}

impl Drop for CanProcessor {
    fn drop(&mut self) {
        unsafe { dbcppp_NetworkFree(self.inner); }
    }
}

/// Dbc files can be in a multitude of encodings
/// use this function to load them into a rust utf-8 string
pub fn load_dbc_file(data: &[u8]) -> Result<String> {
    use encoding::{DecoderTrap, Encoding};

    let encodings = [
        Box::new(encoding::all::UTF_8 as &dyn Encoding),
        Box::new(encoding::all::UTF_16BE as &dyn Encoding),
        Box::new(encoding::all::UTF_16LE as &dyn Encoding),
        Box::new(encoding::all::ISO_8859_1 as &dyn Encoding),
    ];
    for enc in encodings {
        if let Ok(res) = enc.decode(data, DecoderTrap::Strict) {
            return Ok(res);
        }
    }

    return Err(Error::msg("unknown encoding"));
}

#[derive(Debug, Clone)]
pub struct CanResult<'a> {
    pub message_name: &'a str,
    pub signals: HashMap<&'a str, SignalValue<'a>>,
}

#[derive(Debug, Clone)]
pub struct SignalValue<'a> {
    pub raw: u64,
    pub phys: f64,
    pub enum_repr: Option<&'a str>,
    pub unit: &'a str,
}