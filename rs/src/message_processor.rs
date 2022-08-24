use std::collections::HashMap;
use crate::{CanResult, Message, Signal, SignalMuxFlag, SignalValue};
use anyhow::{Result, Error};

#[derive(Debug)]
pub struct MessageProcessor {
    inner: Message,
}

impl MessageProcessor {
    pub(crate) fn new(msg: &Message) -> Result<MessageProcessor> {
        Ok(MessageProcessor {
            inner: msg.clone()
        })
    }

    pub fn parse_frame(&self, payload: &[u8]) -> Result<CanResult> {
        if self.inner.payload_size > payload.len() as _ {
            return Err(Error::msg(format!("payload size ({}) is smaller than the message size ({})", payload.len(), self.inner.payload_size)));
        }

        if self.inner.is_ex_mux {
            self.parse_frame_ex(payload)
        } else {
            self.parse_frame_simple(payload)
        }
    }

    fn parse_frame_simple(&self, payload: &[u8]) -> Result<CanResult> {
        let mut result = HashMap::new();
        for sig in self.inner.signals.iter() {
            let to_insert = match sig.mux_flag {
                SignalMuxFlag::NoMux | SignalMuxFlag::Switch => {
                    true
                }
                SignalMuxFlag::Value => {
                    self.inner.mux_sig.as_ref().unwrap().decode_raw(payload) == sig.mux_value
                }
            };

            if to_insert {
                let raw = sig.decode_raw(payload);
                result.insert(
                    sig.name.as_str(),
                    SignalValue {
                        raw,
                        phys: sig.raw_to_phys(raw),
                        enum_repr: sig.enum_map.get(&raw)
                            .map(|s| s.as_str()),
                        unit: sig.unit.as_str(),
                    },
                );
            }
        }
        Ok(CanResult {
            message_name: self.inner.name.as_str(),
            signals: result,
        })
    }

    fn parse_frame_ex(&self, payload: &[u8]) -> Result<CanResult> {
        let mut result = HashMap::new();

        for sig in self.inner.signals.iter() {
            if self.all_multiplexers_valid(sig, payload) {
                let raw = sig.decode_raw(payload);
                result.insert(
                    sig.name.as_str(),
                    SignalValue {
                        raw,
                        phys: sig.raw_to_phys(raw),
                        enum_repr: sig.enum_map.get(&raw)
                            .map(|s| s.as_str()),
                        unit: sig.unit.as_str(),
                    },
                );
            }
        }

        Ok(CanResult {
            message_name: self.inner.name.as_str(),
            signals: result,
        })
    }

    fn all_multiplexers_valid(&self, sig: &Signal, payload: &[u8]) -> bool {
        match sig.ex_mux_parent.as_ref() {
            None => {
                return true;
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

