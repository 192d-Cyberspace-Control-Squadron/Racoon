//! Racoon SAI Synchronization Daemon
//!
//! Synchronizes database state to hardware via SAI

pub mod vlan_sync;

pub use vlan_sync::{VlanSync, VlanSyncSubscriber};
