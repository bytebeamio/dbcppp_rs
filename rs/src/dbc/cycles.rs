use crate::Signal;
use crate::dbc::ExMuxInfo;

pub fn find_cycle(signals: &[Signal]) -> Option<Vec<&str>> {
    for sig in signals {
        if let Some(stack) = find_cycle_containing_signal(sig, signals) {
            return Some(stack);
        }
    }
    None
}

pub fn find_cycle_containing_signal<'a>(sig: &'a Signal, all: &'a [Signal]) -> Option<Vec<&'a str>> {
    let mut curr = sig;
    let mut stack = vec![sig.name.as_str()];
    loop {
        match curr.ex_mux_parent.as_ref() {
            None => {
                return None;
            }
            Some(ExMuxInfo { switch, ..}) => {
                if stack.iter().find(|&&s| s == switch.as_str()).is_some() {
                    stack.push(switch.as_str());
                    return Some(stack);
                } else {
                    stack.push(switch.as_str());
                    curr = all.iter().find(|s| &s.name == switch).unwrap();
                }
            }
        }
    }
}