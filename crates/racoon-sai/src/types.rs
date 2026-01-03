use crate::bindings::*;
use racoon_common::SaiOid;
use std::fmt;

/// SAI Object Types
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SaiObjectType {
    Switch,
    Port,
    Vlan,
    VlanMember,
    FdbEntry,
    Lag,
    LagMember,
    RouterInterface,
    RouteEntry,
    NeighborEntry,
    NextHop,
    NextHopGroup,
    Acl,
    Hostif,
    Queue,
    Scheduler,
    Buffer,
    Mirror,
}

impl SaiObjectType {
    pub fn to_sai(&self) -> sai_object_type_t {
        match self {
            SaiObjectType::Switch => sai_object_type_t_SAI_OBJECT_TYPE_SWITCH,
            SaiObjectType::Port => sai_object_type_t_SAI_OBJECT_TYPE_PORT,
            SaiObjectType::Vlan => sai_object_type_t_SAI_OBJECT_TYPE_VLAN,
            SaiObjectType::VlanMember => sai_object_type_t_SAI_OBJECT_TYPE_VLAN_MEMBER,
            SaiObjectType::FdbEntry => sai_object_type_t_SAI_OBJECT_TYPE_FDB_ENTRY,
            SaiObjectType::Lag => sai_object_type_t_SAI_OBJECT_TYPE_LAG,
            SaiObjectType::LagMember => sai_object_type_t_SAI_OBJECT_TYPE_LAG_MEMBER,
            SaiObjectType::RouterInterface => sai_object_type_t_SAI_OBJECT_TYPE_ROUTER_INTERFACE,
            SaiObjectType::RouteEntry => sai_object_type_t_SAI_OBJECT_TYPE_ROUTE_ENTRY,
            SaiObjectType::NeighborEntry => sai_object_type_t_SAI_OBJECT_TYPE_NEIGHBOR_ENTRY,
            SaiObjectType::NextHop => sai_object_type_t_SAI_OBJECT_TYPE_NEXT_HOP,
            SaiObjectType::NextHopGroup => sai_object_type_t_SAI_OBJECT_TYPE_NEXT_HOP_GROUP,
            SaiObjectType::Acl => sai_object_type_t_SAI_OBJECT_TYPE_ACL_TABLE,
            SaiObjectType::Hostif => sai_object_type_t_SAI_OBJECT_TYPE_HOSTIF,
            SaiObjectType::Queue => sai_object_type_t_SAI_OBJECT_TYPE_QUEUE,
            SaiObjectType::Scheduler => sai_object_type_t_SAI_OBJECT_TYPE_SCHEDULER,
            SaiObjectType::Buffer => sai_object_type_t_SAI_OBJECT_TYPE_BUFFER_POOL,
            SaiObjectType::Mirror => sai_object_type_t_SAI_OBJECT_TYPE_MIRROR_SESSION,
        }
    }

    pub fn from_oid(oid: SaiOid) -> Option<Self> {
        // SAI OID encoding includes object type in upper bits
        // This is a simplified version - actual implementation would decode OID
        Some(SaiObjectType::Port) // TODO: Implement proper OID decoding
    }
}

impl fmt::Display for SaiObjectType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = match self {
            SaiObjectType::Switch => "SWITCH",
            SaiObjectType::Port => "PORT",
            SaiObjectType::Vlan => "VLAN",
            SaiObjectType::VlanMember => "VLAN_MEMBER",
            SaiObjectType::FdbEntry => "FDB_ENTRY",
            SaiObjectType::Lag => "LAG",
            SaiObjectType::LagMember => "LAG_MEMBER",
            SaiObjectType::RouterInterface => "ROUTER_INTERFACE",
            SaiObjectType::RouteEntry => "ROUTE_ENTRY",
            SaiObjectType::NeighborEntry => "NEIGHBOR_ENTRY",
            SaiObjectType::NextHop => "NEXT_HOP",
            SaiObjectType::NextHopGroup => "NEXT_HOP_GROUP",
            SaiObjectType::Acl => "ACL",
            SaiObjectType::Hostif => "HOSTIF",
            SaiObjectType::Queue => "QUEUE",
            SaiObjectType::Scheduler => "SCHEDULER",
            SaiObjectType::Buffer => "BUFFER",
            SaiObjectType::Mirror => "MIRROR",
        };
        write!(f, "{}", s)
    }
}

/// SAI Attribute wrapper
#[derive(Debug, Clone)]
pub struct SaiAttribute {
    pub id: i32,
    pub value: SaiAttributeValue,
}

#[derive(Debug, Clone)]
pub enum SaiAttributeValue {
    Bool(bool),
    U8(u8),
    U16(u16),
    U32(u32),
    U64(u64),
    I32(i32),
    OidList(Vec<SaiOid>),
    Oid(SaiOid),
    MacAddress([u8; 6]),
    IpAddress([u8; 4]),
    Ipv6Address([u8; 16]),
}

impl SaiAttribute {
    pub fn new_bool(id: i32, value: bool) -> Self {
        Self {
            id,
            value: SaiAttributeValue::Bool(value),
        }
    }

    pub fn new_u16(id: i32, value: u16) -> Self {
        Self {
            id,
            value: SaiAttributeValue::U16(value),
        }
    }

    pub fn new_u32(id: i32, value: u32) -> Self {
        Self {
            id,
            value: SaiAttributeValue::U32(value),
        }
    }

    pub fn new_u64(id: i32, value: u64) -> Self {
        Self {
            id,
            value: SaiAttributeValue::U64(value),
        }
    }

    pub fn new_i32(id: i32, value: i32) -> Self {
        Self {
            id,
            value: SaiAttributeValue::I32(value),
        }
    }

    pub fn new_oid(id: i32, value: SaiOid) -> Self {
        Self {
            id,
            value: SaiAttributeValue::Oid(value),
        }
    }

    /// Convert Rust attribute to C SAI attribute
    /// This is unsafe because it creates raw pointers
    pub unsafe fn to_c_attribute(&self) -> sai_attribute_t {
        let mut attr: sai_attribute_t = std::mem::zeroed();
        attr.id = self.id;

        match &self.value {
            SaiAttributeValue::Bool(v) => {
                attr.value.booldata = *v;
            }
            SaiAttributeValue::U8(v) => {
                attr.value.u8 = *v;
            }
            SaiAttributeValue::U16(v) => {
                attr.value.u16_ = *v;
            }
            SaiAttributeValue::U32(v) => {
                attr.value.u32_ = *v;
            }
            SaiAttributeValue::U64(v) => {
                attr.value.u64_ = *v;
            }
            SaiAttributeValue::I32(v) => {
                attr.value.s32 = *v;
            }
            SaiAttributeValue::Oid(v) => {
                attr.value.oid = *v;
            }
            SaiAttributeValue::MacAddress(mac) => {
                attr.value.mac.copy_from_slice(mac);
            }
            SaiAttributeValue::IpAddress(ip) => {
                attr.value.ipaddr.addr_family = sai_ip_addr_family_t_SAI_IP_ADDR_FAMILY_IPV4;
                attr.value.ipaddr.addr.ip4 = u32::from_be_bytes(*ip);
            }
            SaiAttributeValue::Ipv6Address(ip) => {
                attr.value.ipaddr.addr_family = sai_ip_addr_family_t_SAI_IP_ADDR_FAMILY_IPV6;
                attr.value.ipaddr.addr.ip6.copy_from_slice(ip);
            }
            SaiAttributeValue::OidList(_) => {
                // OID lists require heap allocation and special handling
                // This would need to be implemented based on specific use case
                todo!("OID list conversion not yet implemented");
            }
        }

        attr
    }
}
