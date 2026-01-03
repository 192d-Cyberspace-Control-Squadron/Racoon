use crate::bindings::*;
use crate::status::SaiStatus;
use crate::types::SaiAttribute;
use racoon_common::{Result, SaiOid, VlanId};

pub struct VlanApi {
    api_table: *const sai_vlan_api_t,
}

unsafe impl Send for VlanApi {}
unsafe impl Sync for VlanApi {}

impl VlanApi {
    pub fn new(api_table: *const sai_vlan_api_t) -> Self {
        Self { api_table }
    }

    /// Create a VLAN
    pub fn create_vlan(&self, switch_id: SaiOid, vlan_id: VlanId) -> Result<SaiOid> {
        let mut vlan_oid: SaiOid = 0;

        let attr = SaiAttribute::new_u16(SAI_VLAN_ATTR_VLAN_ID as i32, vlan_id.get());
        let c_attr = unsafe { attr.to_c_attribute() };

        let status = unsafe {
            let api = &*self.api_table;
            if let Some(create_fn) = api.create_vlan {
                create_fn(&mut vlan_oid, switch_id, 1, &c_attr)
            } else {
                SAI_STATUS_NOT_IMPLEMENTED as sai_status_t
            }
        };

        SaiStatus::from(status).to_result()?;
        Ok(vlan_oid)
    }

    /// Remove a VLAN
    pub fn remove_vlan(&self, vlan_oid: SaiOid) -> Result<()> {
        let status = unsafe {
            let api = &*self.api_table;
            if let Some(remove_fn) = api.remove_vlan {
                remove_fn(vlan_oid)
            } else {
                SAI_STATUS_NOT_IMPLEMENTED as sai_status_t
            }
        };

        SaiStatus::from(status).to_result()
    }

    /// Create a VLAN member (add port to VLAN)
    pub fn create_vlan_member(
        &self,
        switch_id: SaiOid,
        vlan_oid: SaiOid,
        bridge_port_id: SaiOid,
        tagging_mode: VlanTaggingMode,
    ) -> Result<SaiOid> {
        let mut member_oid: SaiOid = 0;

        let attrs = vec![
            SaiAttribute::new_oid(SAI_VLAN_MEMBER_ATTR_VLAN_ID as i32, vlan_oid),
            SaiAttribute::new_oid(SAI_VLAN_MEMBER_ATTR_BRIDGE_PORT_ID as i32, bridge_port_id),
            SaiAttribute::new_i32(
                SAI_VLAN_MEMBER_ATTR_VLAN_TAGGING_MODE as i32,
                tagging_mode as i32,
            ),
        ];

        let c_attrs: Vec<sai_attribute_t> = attrs
            .iter()
            .map(|attr| unsafe { attr.to_c_attribute() })
            .collect();

        let status = unsafe {
            let api = &*self.api_table;
            if let Some(create_fn) = api.create_vlan_member {
                create_fn(
                    &mut member_oid,
                    switch_id,
                    c_attrs.len() as u32,
                    c_attrs.as_ptr(),
                )
            } else {
                SAI_STATUS_NOT_IMPLEMENTED as sai_status_t
            }
        };

        SaiStatus::from(status).to_result()?;
        Ok(member_oid)
    }

    /// Remove a VLAN member
    pub fn remove_vlan_member(&self, member_oid: SaiOid) -> Result<()> {
        let status = unsafe {
            let api = &*self.api_table;
            if let Some(remove_fn) = api.remove_vlan_member {
                remove_fn(member_oid)
            } else {
                SAI_STATUS_NOT_IMPLEMENTED as sai_status_t
            }
        };

        SaiStatus::from(status).to_result()
    }

    /// Set VLAN attribute
    pub fn set_attribute(&self, vlan_oid: SaiOid, attribute: &SaiAttribute) -> Result<()> {
        let c_attr = unsafe { attribute.to_c_attribute() };

        let status = unsafe {
            let api = &*self.api_table;
            if let Some(set_fn) = api.set_vlan_attribute {
                set_fn(vlan_oid, &c_attr)
            } else {
                SAI_STATUS_NOT_IMPLEMENTED as sai_status_t
            }
        };

        SaiStatus::from(status).to_result()
    }

    /// Get VLAN attribute
    pub fn get_attribute(&self, vlan_oid: SaiOid, attr_id: i32) -> Result<SaiAttribute> {
        let mut c_attr: sai_attribute_t = unsafe { std::mem::zeroed() };
        c_attr.id = attr_id;

        let status = unsafe {
            let api = &*self.api_table;
            if let Some(get_fn) = api.get_vlan_attribute {
                get_fn(vlan_oid, 1, &mut c_attr)
            } else {
                SAI_STATUS_NOT_IMPLEMENTED as sai_status_t
            }
        };

        SaiStatus::from(status).to_result()?;

        // TODO: Properly convert based on attribute type
        Ok(SaiAttribute::new_u16(attr_id, unsafe { c_attr.value.u16_ }))
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VlanTaggingMode {
    Untagged = SAI_VLAN_TAGGING_MODE_UNTAGGED as isize,
    Tagged = SAI_VLAN_TAGGING_MODE_TAGGED as isize,
    Priority = SAI_VLAN_TAGGING_MODE_PRIORITY_TAGGED as isize,
}
