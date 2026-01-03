use crate::bindings::*;
use crate::constants::*;
use crate::status::SaiStatus;
use crate::types::SaiAttribute;
use racoon_common::{MacAddress, Result, SaiOid};

pub struct FdbApi {
    api_table: *const sai_fdb_api_t,
}

unsafe impl Send for FdbApi {}
unsafe impl Sync for FdbApi {}

impl FdbApi {
    pub fn new(api_table: *const sai_fdb_api_t) -> Self {
        Self { api_table }
    }

    /// Create an FDB entry
    pub fn create_fdb_entry(
        &self,
        switch_id: SaiOid,
        mac: MacAddress,
        bv_id: SaiOid,
        bridge_port_id: SaiOid,
        entry_type: FdbEntryType,
    ) -> Result<()> {
        let mut fdb_entry: sai_fdb_entry_t = unsafe { std::mem::zeroed() };
        fdb_entry.switch_id = switch_id;
        fdb_entry.mac_address.copy_from_slice(mac.as_bytes());
        fdb_entry.bv_id = bv_id;

        let attrs = vec![
            SaiAttribute::new_i32(SAI_FDB_ENTRY_ATTR_TYPE, entry_type as i32),
            SaiAttribute::new_oid(SAI_FDB_ENTRY_ATTR_BRIDGE_PORT_ID, bridge_port_id),
            SaiAttribute::new_i32(
                SAI_FDB_ENTRY_ATTR_PACKET_ACTION,
                SAI_PACKET_ACTION_FORWARD as i32,
            ),
        ];

        let c_attrs: Vec<sai_attribute_t> = attrs
            .iter()
            .map(|attr| unsafe { attr.to_c_attribute() })
            .collect();

        let status = unsafe {
            let api = &*self.api_table;
            if let Some(create_fn) = api.create_fdb_entry {
                create_fn(&fdb_entry, c_attrs.len() as u32, c_attrs.as_ptr())
            } else {
                SAI_STATUS_NOT_IMPLEMENTED as sai_status_t
            }
        };

        SaiStatus::from(status).to_result()
    }

    /// Remove an FDB entry
    pub fn remove_fdb_entry(
        &self,
        switch_id: SaiOid,
        mac: MacAddress,
        bv_id: SaiOid,
    ) -> Result<()> {
        let mut fdb_entry: sai_fdb_entry_t = unsafe { std::mem::zeroed() };
        fdb_entry.switch_id = switch_id;
        fdb_entry.mac_address.copy_from_slice(mac.as_bytes());
        fdb_entry.bv_id = bv_id;

        let status = unsafe {
            let api = &*self.api_table;
            if let Some(remove_fn) = api.remove_fdb_entry {
                remove_fn(&fdb_entry)
            } else {
                SAI_STATUS_NOT_IMPLEMENTED as sai_status_t
            }
        };

        SaiStatus::from(status).to_result()
    }

    /// Flush FDB entries
    pub fn flush_fdb_entries(&self, switch_id: SaiOid, attributes: &[SaiAttribute]) -> Result<()> {
        let c_attrs: Vec<sai_attribute_t> = attributes
            .iter()
            .map(|attr| unsafe { attr.to_c_attribute() })
            .collect();

        let status = unsafe {
            let api = &*self.api_table;
            if let Some(flush_fn) = api.flush_fdb_entries {
                flush_fn(switch_id, c_attrs.len() as u32, c_attrs.as_ptr())
            } else {
                SAI_STATUS_NOT_IMPLEMENTED as sai_status_t
            }
        };

        SaiStatus::from(status).to_result()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FdbEntryType {
    Dynamic = SAI_FDB_ENTRY_TYPE_DYNAMIC as isize,
    Static = SAI_FDB_ENTRY_TYPE_STATIC as isize,
}
