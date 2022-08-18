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
pub mod decision_tree;

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

