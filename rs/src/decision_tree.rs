use std::collections::HashSet;
use anyhow::{Result, Error};
use crate::dbc::{Signal, SignalMuxFlag};
use crate::Message;

/// All the indexes in these structs point to the signals array of owning message
#[derive(Debug)]
pub struct DecisionTree {
    /// decode these unconditionally
    pub no_mux_signals: Vec<usize>,
    /// walk this tree and decode all enabled signals
    pub top_mux_signals: Vec<MuxSignal>,
}

#[derive(Debug)]
pub struct MuxSignal {
    /// index into the signals array of Message owning this tree
    pub multiplexer_signal: usize,
    pub decisions: Vec<Decision>,
}

#[derive(Debug)]
pub struct Decision {
    /// inclusive range
    pub min_val: u64,
    pub max_val: u64,
    /// The signal that this decision will enable
    pub target_signal: MuxSignal,
}

pub fn create_decision_tree(msg: &Message) -> Result<DecisionTree> {
    let extended_mux = msg.signals.iter().find(|sig| sig.ex_mux_parent.is_some()).is_some();

    if extended_mux {
        create_extended_decision_tree(msg)
    } else {
        create_simple_decision_tree(msg)
    }
}

pub fn create_simple_decision_tree(msg: &Message) -> Result<DecisionTree> {
    let mut no_mux_signals = Vec::with_capacity(msg.signals.len());
    let mut switch_idx = None;
    let mut decisions = vec![];
    for (idx, sig) in msg.signals.iter().enumerate() {
        match sig.mux_flag {
            SignalMuxFlag::Switch => {
                if switch_idx.is_some() {
                    return Err(Error::msg("found multiple multiplexing signals without extended multiplexing"));
                } else {
                    switch_idx = Some(idx);
                }
            }
            SignalMuxFlag::Value => {
                decisions.push(Decision {
                    min_val: sig.mux_value,
                    max_val: sig.mux_value,
                    target_signal: MuxSignal {
                        multiplexer_signal: idx,
                        decisions: vec![],
                    },
                });
            }
            SignalMuxFlag::NoMux => {
                no_mux_signals.push(idx);
            }
        }
    }
    let result = match switch_idx {
        None => {
            if decisions.is_empty() {
                DecisionTree {
                    no_mux_signals,
                    top_mux_signals: vec![],
                }
            } else {
                return Err(Error::msg("found multiplexed signals but no multiplexer signals"));
            }
        }
        Some(multiplexer_signal) => {
            DecisionTree {
                no_mux_signals,
                top_mux_signals: vec![
                    MuxSignal {
                        multiplexer_signal,
                        decisions,
                    }
                ],
            }
        }
    };

    Ok(result)
}

pub fn create_extended_decision_tree(msg: &Message) -> Result<DecisionTree> {
    let multiplexers = HashSet::new();
    let
}
