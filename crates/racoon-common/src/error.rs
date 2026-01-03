use thiserror::Error;

#[derive(Error, Debug)]
pub enum RacoonError {
    #[error("SAI error: {0}")]
    Sai(String),

    #[error("Database error: {0}")]
    Database(String),

    #[error("Configuration error: {0}")]
    Config(String),

    #[error("Port not found: {0}")]
    PortNotFound(String),

    #[error("VLAN {0} already exists")]
    VlanExists(u16),

    #[error("VLAN {0} not found")]
    VlanNotFound(u16),

    #[error("Invalid VLAN ID: {0} (must be 1-4094)")]
    InvalidVlanId(u16),

    #[error("FDB entry not found: {0}")]
    FdbNotFound(String),

    #[error("LAG {0} not found")]
    LagNotFound(String),

    #[error("Invalid MAC address: {0}")]
    InvalidMacAddress(String),

    #[error("Dependency not satisfied: {0}")]
    DependencyNotSatisfied(String),

    #[error("OID not found: {0}")]
    OidNotFound(String),

    #[error("Invalid attribute: {0}")]
    InvalidAttribute(String),

    #[error("Library loading error: {0}")]
    LibraryLoad(String),

    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),

    #[error("TOML parsing error: {0}")]
    TomlParse(#[from] toml::de::Error),

    #[error("Internal error: {0}")]
    Internal(String),
}

pub type Result<T> = std::result::Result<T, RacoonError>;
