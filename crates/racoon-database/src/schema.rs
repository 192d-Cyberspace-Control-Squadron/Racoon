//! Database schema definitions for Racoon NOS
//!
//! Following SONiC database architecture with multiple logical databases:
//! - CONFIG_DB: User configuration
//! - APPL_DB: Application state
//! - ASIC_DB: ASIC/SAI state
//! - STATE_DB: Runtime state
//! - COUNTERS_DB: Statistics and counters

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Database identifiers (Valkey database numbers)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Database {
    /// User configuration (DB 4)
    Config = 4,
    /// Application state (DB 0)
    Appl = 0,
    /// ASIC/SAI programming state (DB 1)
    Asic = 1,
    /// Runtime state and status (DB 6)
    State = 6,
    /// Statistics and counters (DB 2)
    Counters = 2,
}

impl Database {
    pub fn id(&self) -> i64 {
        *self as i64
    }
}

/// Table names following SONiC naming conventions
pub mod tables {
    // CONFIG_DB tables
    pub const VLAN: &str = "VLAN";
    pub const VLAN_MEMBER: &str = "VLAN_MEMBER";
    pub const PORT: &str = "PORT";
    pub const LAG: &str = "LAG";
    pub const LAG_MEMBER: &str = "LAG_MEMBER";
    pub const INTERFACE: &str = "INTERFACE";

    // APPL_DB tables
    pub const VLAN_TABLE: &str = "VLAN_TABLE";
    pub const VLAN_MEMBER_TABLE: &str = "VLAN_MEMBER_TABLE";
    pub const PORT_TABLE: &str = "PORT_TABLE";
    pub const LAG_TABLE: &str = "LAG_TABLE";
    pub const LAG_MEMBER_TABLE: &str = "LAG_MEMBER_TABLE";
    pub const FDB_TABLE: &str = "FDB_TABLE";

    // ASIC_DB tables
    pub const ASIC_STATE: &str = "ASIC_STATE";

    // STATE_DB tables
    pub const PORT_STATE: &str = "PORT_STATE";
    pub const VLAN_STATE: &str = "VLAN_STATE";

    // COUNTERS_DB tables
    pub const COUNTERS: &str = "COUNTERS";
    pub const RATES: &str = "RATES";
}

/// VLAN configuration entry (CONFIG_DB)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VlanConfig {
    pub vlanid: u16,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
}

/// VLAN member configuration entry (CONFIG_DB)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VlanMemberConfig {
    pub tagging_mode: String, // "tagged" or "untagged"
}

/// Port configuration entry (CONFIG_DB)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PortConfig {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub speed: Option<String>, // "10000", "25000", "40000", "100000"
    #[serde(skip_serializing_if = "Option::is_none")]
    pub mtu: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub admin_status: Option<String>, // "up" or "down"
    #[serde(skip_serializing_if = "Option::is_none")]
    pub alias: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
}

/// LAG configuration entry (CONFIG_DB)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LagConfig {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub mtu: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub admin_status: Option<String>,
}

/// FDB entry (APPL_DB)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FdbEntry {
    pub port: String,
    #[serde(rename = "type")]
    pub entry_type: String, // "static" or "dynamic"
}

/// Port state (STATE_DB)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PortState {
    pub oper_status: String, // "up" or "down"
    #[serde(skip_serializing_if = "Option::is_none")]
    pub speed: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub mtu: Option<u32>,
}

/// Counter entry (COUNTERS_DB)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Counters {
    #[serde(flatten)]
    pub values: HashMap<String, u64>,
}

/// Key format helpers following SONiC conventions
pub mod keys {
    use racoon_common::VlanId;

    /// Format VLAN key: "Vlan{id}"
    pub fn vlan(vlan_id: VlanId) -> String {
        format!("Vlan{}", vlan_id.get())
    }

    /// Format VLAN member key: "Vlan{id}|{port}"
    pub fn vlan_member(vlan_id: VlanId, port: &str) -> String {
        format!("Vlan{}|{}", vlan_id.get(), port)
    }

    /// Format port key: "Ethernet{id}" or custom name
    pub fn port(port_name: &str) -> String {
        port_name.to_string()
    }

    /// Format LAG key: "PortChannel{id}"
    pub fn lag(lag_id: u32) -> String {
        format!("PortChannel{}", lag_id)
    }

    /// Format LAG member key: "PortChannel{id}|{port}"
    pub fn lag_member(lag_id: u32, port: &str) -> String {
        format!("PortChannel{}|{}", lag_id, port)
    }

    /// Format FDB key: "Vlan{id}:{mac}"
    pub fn fdb(vlan_id: VlanId, mac: &str) -> String {
        format!("Vlan{}:{}", vlan_id.get(), mac)
    }

    /// Format ASIC state key: "{object_type}:{oid}"
    pub fn asic_state(object_type: &str, oid: u64) -> String {
        format!("{}:{}", object_type, oid)
    }
}

/// Database operations result type
pub type DbResult<T> = Result<T, DbError>;

/// Database operation errors
#[derive(Debug, thiserror::Error)]
pub enum DbError {
    #[error("Connection error: {0}")]
    Connection(String),

    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),

    #[error("Key not found: {0}")]
    NotFound(String),

    #[error("Invalid data format: {0}")]
    InvalidFormat(String),

    #[error("Operation failed: {0}")]
    Operation(String),
}

impl From<redis::RedisError> for DbError {
    fn from(err: redis::RedisError) -> Self {
        DbError::Operation(err.to_string())
    }
}
