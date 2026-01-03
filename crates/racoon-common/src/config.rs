use crate::error::{RacoonError, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::Path;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub platform: PlatformConfig,
    pub database: DatabaseConfig,
    pub logging: LoggingConfig,
    pub services: ServicesConfig,
    pub management: ManagementConfig,
    #[serde(default)]
    pub features: FeaturesConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlatformConfig {
    pub name: String,
    pub sai_library: String,
    #[serde(default = "default_config_db_path")]
    pub config_db_path: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DatabaseConfig {
    #[serde(default = "default_db_host")]
    pub host: String,
    #[serde(default = "default_db_port")]
    pub port: u16,
    #[serde(default = "default_db_socket")]
    pub socket: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoggingConfig {
    #[serde(default = "default_log_level")]
    pub level: String,
    #[serde(default = "default_log_format")]
    pub format: String,
    #[serde(default = "default_log_output")]
    pub output: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServicesConfig {
    pub enabled: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ManagementConfig {
    #[serde(default = "default_rest_port")]
    pub rest_api_port: u16,
    #[serde(default = "default_cli_socket")]
    pub cli_socket: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct FeaturesConfig {
    #[serde(default)]
    pub warm_boot: bool,
    #[serde(default)]
    pub fast_reboot: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HardwareConfig {
    pub port_count: u32,
    pub port_lanes: u32,
    pub max_speed: u32,
    pub buffer_size: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CapabilitiesConfig {
    pub max_vlans: u32,
    pub max_fdb_entries: u32,
    pub max_routes: u32,
    pub max_ecmp_groups: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlatformDetailsConfig {
    pub name: String,
    pub asic_type: String,
    pub sai_library: String,
    pub hardware: HardwareConfig,
    pub port_mapping: HashMap<String, (u32, u32)>,
    pub capabilities: CapabilitiesConfig,
}

// Default value functions
fn default_config_db_path() -> String {
    "/etc/racoon/config_db.json".to_string()
}

fn default_db_host() -> String {
    "127.0.0.1".to_string()
}

fn default_db_port() -> u16 {
    6379
}

fn default_db_socket() -> String {
    "/var/run/racoon/database.sock".to_string()
}

fn default_log_level() -> String {
    "info".to_string()
}

fn default_log_format() -> String {
    "json".to_string()
}

fn default_log_output() -> String {
    "/var/log/racoon/racoon.log".to_string()
}

fn default_rest_port() -> u16 {
    8080
}

fn default_cli_socket() -> String {
    "/var/run/racoon/cli.sock".to_string()
}

impl Config {
    pub fn load<P: AsRef<Path>>(path: P) -> Result<Self> {
        let content = std::fs::read_to_string(path)
            .map_err(|e| RacoonError::Config(format!("Failed to read config file: {}", e)))?;

        let config: Config = toml::from_str(&content)?;
        Ok(config)
    }

    pub fn load_platform<P: AsRef<Path>>(path: P) -> Result<PlatformDetailsConfig> {
        let content = std::fs::read_to_string(path)
            .map_err(|e| RacoonError::Config(format!("Failed to read platform config: {}", e)))?;

        let platform: PlatformDetailsConfig = toml::from_str(&content)?;
        Ok(platform)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_values() {
        let config = r#"
            [platform]
            name = "test"
            sai_library = "/usr/lib/libsai.so"

            [database]

            [logging]

            [services]
            enabled = ["database", "syncd"]

            [management]
        "#;

        let parsed: Config = toml::from_str(config).unwrap();
        assert_eq!(parsed.database.port, 6379);
        assert_eq!(parsed.logging.level, "info");
        assert_eq!(parsed.management.rest_api_port, 8080);
    }
}
