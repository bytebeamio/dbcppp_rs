use anyhow::{Result, Error};
use crate::dbc::{Signal, SignalMuxFlag};

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

pub fn create_decision_tree(signals: &[Signal]) -> Result<Vec<MuxSignal>> {
    let extended_mux = signals.iter().find(|sig| sig.ex_mux_parent.is_some()).is_some();

    let top_mux_signals = if extended_mux {
        // build nested decision tree
        vec![]
    } else {
        let mut switch_idx = None;
        let mut decisions = vec![];
        for (idx, sig) in signals.iter().enumerate() {
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
                        }
                    });
                }
                SignalMuxFlag::NoMux => {}
            }
        }
        match switch_idx {
            None => {
                if decisions.is_empty() {
                    vec![]
                } else {
                    return Err(Error::msg("found multiplexed signals but no multiplexer signals"));
                }
            }
            Some(multiplexer_signal) => {
                vec![
                    MuxSignal {
                        multiplexer_signal,
                        decisions
                    }
                ]
            }
        }
    };

    Ok(top_mux_signals)
}