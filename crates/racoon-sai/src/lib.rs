pub mod adapter;
pub mod bindings;
pub mod constants;
pub mod fdb;
pub mod lag;
pub mod port;
pub mod status;
pub mod switch;
pub mod types;
pub mod vlan;

pub use adapter::SaiAdapter;
pub use status::SaiStatus;
pub use types::{SaiAttribute, SaiObjectType};
pub use vlan::VlanApi;

// Re-export bindings for convenient access
pub use bindings::*;
// Re-export constants
pub use constants::*;
