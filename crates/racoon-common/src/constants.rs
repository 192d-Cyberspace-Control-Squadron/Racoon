/// Maximum VLAN ID
pub const MAX_VLAN_ID: u16 = 4094;

/// Minimum VLAN ID
pub const MIN_VLAN_ID: u16 = 1;

/// Default VLAN ID
pub const DEFAULT_VLAN_ID: u16 = 1;

/// Database key separators
pub const DB_KEY_SEPARATOR: &str = "|";
pub const DB_TABLE_SEPARATOR: &str = ":";

/// Port name prefix
pub const PORT_PREFIX: &str = "Ethernet";

/// VLAN name prefix
pub const VLAN_PREFIX: &str = "Vlan";

/// LAG name prefix
pub const LAG_PREFIX: &str = "PortChannel";

/// Default MTU
pub const DEFAULT_MTU: u32 = 9100;

/// SAI Object Type prefixes for ASIC_DB
pub mod sai_object_types {
    pub const SWITCH: &str = "SAI_OBJECT_TYPE_SWITCH";
    pub const PORT: &str = "SAI_OBJECT_TYPE_PORT";
    pub const VLAN: &str = "SAI_OBJECT_TYPE_VLAN";
    pub const VLAN_MEMBER: &str = "SAI_OBJECT_TYPE_VLAN_MEMBER";
    pub const FDB_ENTRY: &str = "SAI_OBJECT_TYPE_FDB_ENTRY";
    pub const LAG: &str = "SAI_OBJECT_TYPE_LAG";
    pub const LAG_MEMBER: &str = "SAI_OBJECT_TYPE_LAG_MEMBER";
    pub const ROUTER_INTERFACE: &str = "SAI_OBJECT_TYPE_ROUTER_INTERFACE";
    pub const ROUTE_ENTRY: &str = "SAI_OBJECT_TYPE_ROUTE_ENTRY";
    pub const NEIGHBOR_ENTRY: &str = "SAI_OBJECT_TYPE_NEIGHBOR_ENTRY";
}
