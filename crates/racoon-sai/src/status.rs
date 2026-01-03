use crate::bindings::*;
use crate::constants::*;
use racoon_common::RacoonError;
use std::fmt;

/// Safe Rust wrapper for SAI status codes
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SaiStatus(pub sai_status_t);

impl SaiStatus {
    pub const SUCCESS: Self = SaiStatus(SAI_STATUS_SUCCESS as sai_status_t);
    pub const FAILURE: Self = SaiStatus(SAI_STATUS_FAILURE as sai_status_t);
    pub const NOT_SUPPORTED: Self = SaiStatus(SAI_STATUS_NOT_SUPPORTED as sai_status_t);
    pub const NO_MEMORY: Self = SaiStatus(SAI_STATUS_NO_MEMORY as sai_status_t);
    pub const INVALID_PARAMETER: Self = SaiStatus(SAI_STATUS_INVALID_PARAMETER as sai_status_t);
    pub const ITEM_ALREADY_EXISTS: Self = SaiStatus(SAI_STATUS_ITEM_ALREADY_EXISTS as sai_status_t);
    pub const ITEM_NOT_FOUND: Self = SaiStatus(SAI_STATUS_ITEM_NOT_FOUND as sai_status_t);
    pub const TABLE_FULL: Self = SaiStatus(SAI_STATUS_TABLE_FULL as sai_status_t);

    pub fn is_success(&self) -> bool {
        self.0 == SAI_STATUS_SUCCESS as sai_status_t
    }

    pub fn is_error(&self) -> bool {
        !self.is_success()
    }

    pub fn to_result(self) -> Result<(), RacoonError> {
        if self.is_success() {
            Ok(())
        } else {
            Err(RacoonError::Sai(self.to_string()))
        }
    }
}

impl From<sai_status_t> for SaiStatus {
    fn from(status: sai_status_t) -> Self {
        SaiStatus(status)
    }
}

impl fmt::Display for SaiStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let msg = match self.0 {
            x if x == SAI_STATUS_SUCCESS as i32 => "SUCCESS",
            x if x == SAI_STATUS_FAILURE => "FAILURE",
            x if x == SAI_STATUS_NOT_SUPPORTED => "NOT_SUPPORTED",
            x if x == SAI_STATUS_NO_MEMORY => "NO_MEMORY",
            x if x == SAI_STATUS_INSUFFICIENT_RESOURCES => "INSUFFICIENT_RESOURCES",
            x if x == SAI_STATUS_INVALID_PARAMETER => "INVALID_PARAMETER",
            x if x == SAI_STATUS_ITEM_ALREADY_EXISTS => "ITEM_ALREADY_EXISTS",
            x if x == SAI_STATUS_ITEM_NOT_FOUND => "ITEM_NOT_FOUND",
            x if x == SAI_STATUS_BUFFER_OVERFLOW => "BUFFER_OVERFLOW",
            x if x == SAI_STATUS_INVALID_PORT_NUMBER => "INVALID_PORT_NUMBER",
            x if x == SAI_STATUS_INVALID_PORT_MEMBER => "INVALID_PORT_MEMBER",
            x if x == SAI_STATUS_INVALID_VLAN_ID => "INVALID_VLAN_ID",
            x if x == SAI_STATUS_UNINITIALIZED => "UNINITIALIZED",
            x if x == SAI_STATUS_TABLE_FULL => "TABLE_FULL",
            x if x == SAI_STATUS_MANDATORY_ATTRIBUTE_MISSING => "MANDATORY_ATTRIBUTE_MISSING",
            x if x == SAI_STATUS_NOT_IMPLEMENTED => "NOT_IMPLEMENTED",
            x if x == SAI_STATUS_ADDR_NOT_FOUND => "ADDR_NOT_FOUND",
            x if x == SAI_STATUS_OBJECT_IN_USE => "OBJECT_IN_USE",
            x if x == SAI_STATUS_INVALID_OBJECT_TYPE => "INVALID_OBJECT_TYPE",
            x if x == SAI_STATUS_INVALID_OBJECT_ID => "INVALID_OBJECT_ID",
            x if x == SAI_STATUS_INVALID_NV_STORAGE => "INVALID_NV_STORAGE",
            x if x == SAI_STATUS_NV_STORAGE_FULL => "NV_STORAGE_FULL",
            x if x == SAI_STATUS_INVALID_ATTRIBUTE_0 => "INVALID_ATTRIBUTE_0",
            _ => "UNKNOWN_STATUS",
        };
        write!(f, "SAI_{} ({})", msg, self.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_status_success() {
        let status = SaiStatus::SUCCESS;
        assert!(status.is_success());
        assert!(!status.is_error());
        assert!(status.to_result().is_ok());
    }

    #[test]
    fn test_status_error() {
        let status = SaiStatus::FAILURE;
        assert!(!status.is_success());
        assert!(status.is_error());
        assert!(status.to_result().is_err());
    }
}
