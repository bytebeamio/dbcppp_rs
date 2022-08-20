use std::collections::HashMap;
use std::ffi::CString;
use dbcppp_rs_sys::*;
use anyhow::{Context, Error, Result};
use crate::dbc::{Dbc, Message, Signal, SignalMuxFlag};
use crate::utils::TryToString;

pub mod utils;
pub mod dbc;
pub mod decision_tree;

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
                msg.id,
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
        let msg = self.message_processors.get(&id)
            .ok_or(Error::msg("Invalid can id"))?;
        Ok(CanResult {
            message_name: msg.inner.name.as_str(),
            signals: msg.parse_frame(payload)?,
        })
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

pub struct MessageProcessor {
    inner: Message,
}

impl MessageProcessor {
    pub fn new(inner: Message) -> Result<MessageProcessor> {
        Ok(MessageProcessor {
            inner
        })
    }

    pub fn parse_frame(&self, payload: &[u8]) -> Result<HashMap<&str, SignalValue>> {
        if self.inner.payload_size > payload.len() as _ {
            return Err(Error::msg(format!("payload size ({}) is smaller than the message size ({})", payload.len(), self.inner.payload_size)));
        }

        let mut result = HashMap::new();
        for sig in self.inner.signals.iter() {
            let mut to_insert = false;
            if sig.mux_flag != SignalMuxFlag::Value {
                to_insert = true;
            } else if self.inner.mux_sig.is_some()
                && sig.ex_mux_parent.is_none()
                && self.inner.mux_sig.as_ref().unwrap().decode_raw(payload) == sig.mux_value
            {
                to_insert = true;
            } else {
                if self.all_multiplexers_valid(sig, payload) {
                    to_insert = true;
                }
            }
            if to_insert {
                let raw = sig.decode_raw(payload);
                result.insert(sig.name.as_str(), SignalValue {
                    raw,
                    phys: sig.raw_to_phys(raw),
                    enum_repr: sig.enum_map.get(&raw)
                        .map(|s| s.as_str()),
                    unit: sig.unit.as_str(),
                });
            }
        }
        Ok(result)
    }

    fn all_multiplexers_valid(&self, sig: &Signal, payload: &[u8]) -> bool {
        match sig.ex_mux_parent.as_ref() {
            None => {
                return false;
            }
            Some(par_info) => {
                match self.inner.signals.iter().find(|sig| sig.name == par_info.switch) {
                    None => {
                        return false;
                    }
                    Some(par_sig) => {
                        let par_value = par_sig.decode_raw(payload);
                        for range in par_info.ranges.iter() {
                            if par_value >= range.from && par_value <= range.to {
                                if par_sig.ex_mux_parent.is_some() {
                                    return self.all_multiplexers_valid(par_sig, payload);
                                } else {
                                    return true;
                                }
                            } else {
                                return false;
                            }
                        }
                        return false;
                    }
                }
            }
        }
    }
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