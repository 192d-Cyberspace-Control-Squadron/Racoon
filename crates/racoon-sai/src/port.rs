use crate::bindings::*;
use crate::status::SaiStatus;
use crate::types::SaiAttribute;
use racoon_common::{Result, SaiOid};

pub struct PortApi {
    api_table: *const sai_port_api_t,
}

unsafe impl Send for PortApi {}
unsafe impl Sync for PortApi {}

impl PortApi {
    pub fn new(api_table: *const sai_port_api_t) -> Self {
        Self { api_table }
    }

    /// Set port attribute
    pub fn set_attribute(&self, port_id: SaiOid, attribute: &SaiAttribute) -> Result<()> {
        let c_attr = unsafe { attribute.to_c_attribute() };

        let status = unsafe {
            let api = &*self.api_table;
            if let Some(set_fn) = api.set_port_attribute {
                set_fn(port_id, &c_attr)
            } else {
                SAI_STATUS_NOT_IMPLEMENTED as sai_status_t
            }
        };

        SaiStatus::from(status).to_result()
    }

    /// Get port attribute
    pub fn get_attribute(&self, port_id: SaiOid, attr_id: i32) -> Result<SaiAttribute> {
        let mut c_attr: sai_attribute_t = unsafe { std::mem::zeroed() };
        c_attr.id = attr_id;

        let status = unsafe {
            let api = &*self.api_table;
            if let Some(get_fn) = api.get_port_attribute {
                get_fn(port_id, 1, &mut c_attr)
            } else {
                SAI_STATUS_NOT_IMPLEMENTED as sai_status_t
            }
        };

        SaiStatus::from(status).to_result()?;

        // TODO: Properly convert based on attribute type
        Ok(SaiAttribute::new_u32(attr_id, unsafe { c_attr.value.u32_ }))
    }

    /// Get port statistics
    pub fn get_stats(&self, port_id: SaiOid, counter_ids: &[sai_port_stat_t]) -> Result<Vec<u64>> {
        let mut counters = vec![0u64; counter_ids.len()];

        let status = unsafe {
            let api = &*self.api_table;
            if let Some(get_stats_fn) = api.get_port_stats {
                get_stats_fn(
                    port_id,
                    counter_ids.len() as u32,
                    counter_ids.as_ptr(),
                    counters.as_mut_ptr(),
                )
            } else {
                SAI_STATUS_NOT_IMPLEMENTED as sai_status_t
            }
        };

        SaiStatus::from(status).to_result()?;
        Ok(counters)
    }

    /// Clear port statistics
    pub fn clear_stats(&self, port_id: SaiOid, counter_ids: &[sai_port_stat_t]) -> Result<()> {
        let status = unsafe {
            let api = &*self.api_table;
            if let Some(clear_stats_fn) = api.clear_port_stats {
                clear_stats_fn(port_id, counter_ids.len() as u32, counter_ids.as_ptr())
            } else {
                SAI_STATUS_NOT_IMPLEMENTED as sai_status_t
            }
        };

        SaiStatus::from(status).to_result()
    }
}
