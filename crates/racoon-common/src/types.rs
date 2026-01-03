use serde::{Deserialize, Serialize};
use std::fmt;
use std::str::FromStr;

/// MAC address representation
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct MacAddress([u8; 6]);

impl MacAddress {
    pub fn new(bytes: [u8; 6]) -> Self {
        Self(bytes)
    }

    pub fn as_bytes(&self) -> &[u8; 6] {
        &self.0
    }
}

impl FromStr for MacAddress {
    type Err = &'static str;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let s = s.replace([':', '-', '.'], "");
        if s.len() != 12 {
            return Err("MAC address must be 12 hex digits");
        }

        let mut bytes = [0u8; 6];
        for i in 0..6 {
            bytes[i] = u8::from_str_radix(&s[i * 2..i * 2 + 2], 16)
                .map_err(|_| "Invalid hex digit in MAC address")?;
        }

        Ok(Self(bytes))
    }
}

impl fmt::Display for MacAddress {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{:02x}:{:02x}:{:02x}:{:02x}:{:02x}:{:02x}",
            self.0[0], self.0[1], self.0[2], self.0[3], self.0[4], self.0[5]
        )
    }
}

/// VLAN ID (1-4094)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct VlanId(u16);

impl VlanId {
    pub fn new(id: u16) -> Option<Self> {
        if (1..=4094).contains(&id) {
            Some(Self(id))
        } else {
            None
        }
    }

    pub fn get(&self) -> u16 {
        self.0
    }
}

impl fmt::Display for VlanId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// VLAN tagging mode
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum VlanTaggingMode {
    Untagged,
    Tagged,
    Priority,
}

/// Port operational status
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum PortOperStatus {
    Up,
    Down,
    Testing,
    Unknown,
}

/// Port admin status
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum PortAdminStatus {
    Up,
    Down,
}

/// FDB entry type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum FdbEntryType {
    Dynamic,
    Static,
}

/// Port speed in Mbps
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum PortSpeed {
    Speed1G = 1000,
    Speed10G = 10000,
    Speed25G = 25000,
    Speed40G = 40000,
    Speed50G = 50000,
    Speed100G = 100000,
    Speed200G = 200000,
    Speed400G = 400000,
}

impl PortSpeed {
    pub fn from_mbps(mbps: u32) -> Option<Self> {
        match mbps {
            1000 => Some(Self::Speed1G),
            10000 => Some(Self::Speed10G),
            25000 => Some(Self::Speed25G),
            40000 => Some(Self::Speed40G),
            50000 => Some(Self::Speed50G),
            100000 => Some(Self::Speed100G),
            200000 => Some(Self::Speed200G),
            400000 => Some(Self::Speed400G),
            _ => None,
        }
    }

    pub fn as_mbps(&self) -> u32 {
        *self as u32
    }
}

/// SAI Object ID (opaque 64-bit identifier)
pub type SaiOid = u64;

/// Database table names
pub mod db_tables {
    pub const CONFIG_DB: &str = "CONFIG_DB";
    pub const APPL_DB: &str = "APPL_DB";
    pub const ASIC_DB: &str = "ASIC_DB";
    pub const STATE_DB: &str = "STATE_DB";
    pub const COUNTER_DB: &str = "COUNTER_DB";
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mac_address() {
        let mac = "00:11:22:33:44:55".parse::<MacAddress>().unwrap();
        assert_eq!(mac.to_string(), "00:11:22:33:44:55");

        let mac2 = "00-11-22-33-44-55".parse::<MacAddress>().unwrap();
        assert_eq!(mac, mac2);
    }

    #[test]
    fn test_vlan_id() {
        assert!(VlanId::new(0).is_none());
        assert!(VlanId::new(1).is_some());
        assert!(VlanId::new(4094).is_some());
        assert!(VlanId::new(4095).is_none());
    }

    #[test]
    fn test_port_speed() {
        let speed = PortSpeed::from_mbps(100000).unwrap();
        assert_eq!(speed, PortSpeed::Speed100G);
        assert_eq!(speed.as_mbps(), 100000);
    }
}
