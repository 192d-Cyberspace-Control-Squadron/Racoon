use crate::bindings::*;
use crate::constants::*;
use crate::status::SaiStatus;
use crate::types::SaiAttribute;
use racoon_common::{Result, SaiOid};

pub struct LagApi {
    api_table: *const sai_lag_api_t,
}

unsafe impl Send for LagApi {}
unsafe impl Sync for LagApi {}

impl LagApi {
    pub fn new(api_table: *const sai_lag_api_t) -> Self {
        Self { api_table }
    }

    /// Create a LAG (Link Aggregation Group / Port Channel)
    pub fn create_lag(&self, switch_id: SaiOid, attributes: &[SaiAttribute]) -> Result<SaiOid> {
        let mut lag_oid: SaiOid = 0;

        let c_attrs: Vec<sai_attribute_t> = attributes
            .iter()
            .map(|attr| unsafe { attr.to_c_attribute() })
            .collect();

        let status = unsafe {
            let api = &*self.api_table;
            if let Some(create_fn) = api.create_lag {
                create_fn(
                    &mut lag_oid,
                    switch_id,
                    c_attrs.len() as u32,
                    c_attrs.as_ptr(),
                )
            } else {
                SAI_STATUS_NOT_IMPLEMENTED as sai_status_t
            }
        };

        SaiStatus::from(status).to_result()?;
        Ok(lag_oid)
    }

    /// Remove a LAG
    pub fn remove_lag(&self, lag_oid: SaiOid) -> Result<()> {
        let status = unsafe {
            let api = &*self.api_table;
            if let Some(remove_fn) = api.remove_lag {
                remove_fn(lag_oid)
            } else {
                SAI_STATUS_NOT_IMPLEMENTED as sai_status_t
            }
        };

        SaiStatus::from(status).to_result()
    }

    /// Create a LAG member (add port to LAG)
    pub fn create_lag_member(
        &self,
        switch_id: SaiOid,
        lag_id: SaiOid,
        port_id: SaiOid,
    ) -> Result<SaiOid> {
        let mut member_oid: SaiOid = 0;

        let attrs = [
            SaiAttribute::new_oid(SAI_LAG_MEMBER_ATTR_LAG_ID, lag_id),
            SaiAttribute::new_oid(SAI_LAG_MEMBER_ATTR_PORT_ID, port_id),
        ];

        let c_attrs: Vec<sai_attribute_t> = attrs
            .iter()
            .map(|attr| unsafe { attr.to_c_attribute() })
            .collect();

        let status = unsafe {
            let api = &*self.api_table;
            if let Some(create_fn) = api.create_lag_member {
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

    /// Remove a LAG member
    pub fn remove_lag_member(&self, member_oid: SaiOid) -> Result<()> {
        let status = unsafe {
            let api = &*self.api_table;
            if let Some(remove_fn) = api.remove_lag_member {
                remove_fn(member_oid)
            } else {
                SAI_STATUS_NOT_IMPLEMENTED as sai_status_t
            }
        };

        SaiStatus::from(status).to_result()
    }

    /// Set LAG attribute
    pub fn set_attribute(&self, lag_oid: SaiOid, attribute: &SaiAttribute) -> Result<()> {
        let c_attr = unsafe { attribute.to_c_attribute() };

        let status = unsafe {
            let api = &*self.api_table;
            if let Some(set_fn) = api.set_lag_attribute {
                set_fn(lag_oid, &c_attr)
            } else {
                SAI_STATUS_NOT_IMPLEMENTED as sai_status_t
            }
        };

        SaiStatus::from(status).to_result()
    }
}
