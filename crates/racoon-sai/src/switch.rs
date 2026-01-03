use crate::bindings::*;
use crate::constants::*;
use crate::status::SaiStatus;
use crate::types::SaiAttribute;
use racoon_common::{Result, SaiOid};

pub struct SwitchApi {
    api_table: *const sai_switch_api_t,
}

unsafe impl Send for SwitchApi {}
unsafe impl Sync for SwitchApi {}

impl SwitchApi {
    pub fn new(api_table: *const sai_switch_api_t) -> Self {
        Self { api_table }
    }

    /// Create and initialize a switch
    pub fn create_switch(&self, attributes: &[SaiAttribute]) -> Result<SaiOid> {
        let mut switch_id: SaiOid = 0;

        // Convert Rust attributes to C attributes
        let c_attrs: Vec<sai_attribute_t> = attributes
            .iter()
            .map(|attr| unsafe { attr.to_c_attribute() })
            .collect();

        let status = unsafe {
            let api = &*self.api_table;
            if let Some(create_fn) = api.create_switch {
                create_fn(&mut switch_id, c_attrs.len() as u32, c_attrs.as_ptr())
            } else {
                SAI_STATUS_NOT_IMPLEMENTED as sai_status_t
            }
        };

        SaiStatus::from(status).to_result()?;
        Ok(switch_id)
    }

    /// Remove a switch
    pub fn remove_switch(&self, switch_id: SaiOid) -> Result<()> {
        let status = unsafe {
            let api = &*self.api_table;
            if let Some(remove_fn) = api.remove_switch {
                remove_fn(switch_id)
            } else {
                SAI_STATUS_NOT_IMPLEMENTED as sai_status_t
            }
        };

        SaiStatus::from(status).to_result()
    }

    /// Set switch attribute
    pub fn set_attribute(&self, switch_id: SaiOid, attribute: &SaiAttribute) -> Result<()> {
        let c_attr = unsafe { attribute.to_c_attribute() };

        let status = unsafe {
            let api = &*self.api_table;
            if let Some(set_fn) = api.set_switch_attribute {
                set_fn(switch_id, &c_attr)
            } else {
                SAI_STATUS_NOT_IMPLEMENTED as sai_status_t
            }
        };

        SaiStatus::from(status).to_result()
    }

    /// Get switch attribute
    pub fn get_attribute(&self, switch_id: SaiOid, attr_id: u32) -> Result<SaiAttribute> {
        let mut c_attr: sai_attribute_t = unsafe { std::mem::zeroed() };
        c_attr.id = attr_id;

        let status = unsafe {
            let api = &*self.api_table;
            if let Some(get_fn) = api.get_switch_attribute {
                get_fn(switch_id, 1, &mut c_attr)
            } else {
                SAI_STATUS_NOT_IMPLEMENTED as sai_status_t
            }
        };

        SaiStatus::from(status).to_result()?;

        // Convert C attribute back to Rust (simplified for now)
        // TODO: Properly convert based on attribute type
        Ok(SaiAttribute::new_u32(attr_id, unsafe { c_attr.value.u32_ }))
    }
}
