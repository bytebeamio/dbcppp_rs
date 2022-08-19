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
        Ok(MessageProcessor {
            inner
        })
    }

    pub fn parse_frame(&self, payload: &[u8]) -> Result<HashMap<String, CanValue>> {
        if self.inner.payload_size > payload.len() as _ {
            return Err(Error::msg(format!("payload size ({}) is smaller than the message size ({})", payload.len(), self.inner.payload_size)));
        }

        let mut result = HashMap::new();
        for sig in self.inner.signals.iter() {
            if sig.mux_flag != SignalMuxFlag::Value {
                result.insert(sig.name.clone(), sig.decode(payload));
            } else if self.inner.mux_sig.is_some()
                && sig.ex_mux_parent.is_none()
                && self.inner.mux_sig.as_ref().unwrap().decode(payload) == sig.mux_value
            {
                result.insert(sig.name.clone(), sig.decode(payload));
            } else {
                // TODO: decode ex mux
                if self.all_multiplexers_valid(sig, payload) {
                    result.insert(sig.name.clone(), sig.decode(payload));
                }
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
                        let par_value = par_sig.decode(payload);
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

type CanValue = u64;
