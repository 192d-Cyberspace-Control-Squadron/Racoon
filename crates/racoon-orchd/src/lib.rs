//! Racoon Orchestration Daemon
//!
//! Translates configuration from CONFIG_DB to application-level database entries

pub mod vlan_orch;

pub use vlan_orch::{VlanOrch, VlanOrchSubscriber};
