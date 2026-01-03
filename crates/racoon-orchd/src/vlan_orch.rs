//! VLAN Orchestration Agent
//!
//! Listens to CONFIG_DB VLAN table and creates corresponding entries in APPL_DB

use async_trait::async_trait;
use dashmap::DashMap;
use racoon_common::{Result, VlanId};
use racoon_db_client::{Database, DbClient, DbSubscriber};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tracing::{debug, error, info, warn};

/// VLAN configuration from CONFIG_DB
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VlanConfig {
    pub vlanid: u16,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
}

/// VLAN entry for APPL_DB
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VlanEntry {
    pub vlanid: u16,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
}

/// VLAN Orchestration Agent
pub struct VlanOrch {
    db_client: Arc<DbClient>,
    /// Track VLANs we've processed
    vlans: DashMap<VlanId, VlanEntry>,
}

impl VlanOrch {
    /// Create new VLAN orchestration agent
    pub fn new(db_client: Arc<DbClient>) -> Self {
        Self {
            db_client,
            vlans: DashMap::new(),
        }
    }

    /// Start the orchestration agent
    pub async fn start(&self) -> Result<()> {
        info!("Starting VLAN orchestration agent");

        // Load existing VLANs from CONFIG_DB
        self.sync_vlans().await?;

        info!("VLAN orchestration agent started");
        Ok(())
    }

    /// Sync all VLANs from CONFIG_DB to APPL_DB
    async fn sync_vlans(&self) -> Result<()> {
        info!("Syncing VLANs from CONFIG_DB");

        let keys = self.db_client.keys(Database::Config, "VLAN|Vlan*").await?;

        for key in keys {
            if let Some(vlan_name) = key.strip_prefix("VLAN|") {
                match self.process_vlan_config(vlan_name).await {
                    Ok(_) => debug!("Synced VLAN: {}", vlan_name),
                    Err(e) => warn!("Failed to sync VLAN {}: {}", vlan_name, e),
                }
            }
        }

        info!("Synced {} VLANs", self.vlans.len());
        Ok(())
    }

    /// Process VLAN configuration and create APPL_DB entry
    async fn process_vlan_config(&self, vlan_name: &str) -> Result<()> {
        let config_key = format!("VLAN|{}", vlan_name);

        // Get VLAN config from CONFIG_DB
        let config: VlanConfig = self.db_client.get(Database::Config, &config_key).await?;

        let vlan_id = VlanId::new(config.vlanid)
            .ok_or(racoon_common::RacoonError::InvalidVlanId(config.vlanid))?;

        // Create APPL_DB entry
        let vlan_entry = VlanEntry {
            vlanid: config.vlanid,
            description: config.description.clone(),
        };

        let appl_key = format!("VLAN_TABLE:{}", vlan_name);
        self.db_client
            .set(Database::Appl, &appl_key, &vlan_entry)
            .await?;

        // Track the VLAN
        self.vlans.insert(vlan_id, vlan_entry.clone());

        info!(
            "Processed VLAN {} (ID: {}) -> APPL_DB",
            vlan_name, config.vlanid
        );

        // Publish notification
        let notification = serde_json::json!({
            "operation": "SET",
            "table": "VLAN_TABLE",
            "key": vlan_name,
            "data": vlan_entry
        });

        self.db_client
            .publish("VLAN_TABLE", &notification.to_string())
            .await?;

        Ok(())
    }

    /// Handle VLAN deletion
    async fn delete_vlan(&self, vlan_name: &str) -> Result<()> {
        // Parse VLAN ID from name (Vlan100 -> 100)
        let vlan_id_str = vlan_name.strip_prefix("Vlan").unwrap_or(vlan_name);
        let vlan_id_num = vlan_id_str
            .parse::<u16>()
            .map_err(|_| racoon_common::RacoonError::InvalidVlanId(0))?;
        let vlan_id = VlanId::new(vlan_id_num)
            .ok_or(racoon_common::RacoonError::InvalidVlanId(vlan_id_num))?;

        // Remove from APPL_DB
        let appl_key = format!("VLAN_TABLE:{}", vlan_name);
        self.db_client.del(Database::Appl, &appl_key).await?;

        // Remove from tracking
        self.vlans.remove(&vlan_id);

        info!("Deleted VLAN {} from APPL_DB", vlan_name);

        // Publish deletion notification
        let notification = serde_json::json!({
            "operation": "DEL",
            "table": "VLAN_TABLE",
            "key": vlan_name
        });

        self.db_client
            .publish("VLAN_TABLE", &notification.to_string())
            .await?;

        Ok(())
    }

    /// Handle database notification
    pub async fn handle_notification(&self, channel: &str, message: &str) {
        debug!("Received notification on {}: {}", channel, message);

        // Parse notification
        let notification: serde_json::Value = match serde_json::from_str(message) {
            Ok(v) => v,
            Err(e) => {
                error!("Failed to parse notification: {}", e);
                return;
            }
        };

        let operation = notification["operation"].as_str().unwrap_or("");
        let key = notification["key"].as_str().unwrap_or("");

        match operation {
            "SET" | "CREATE" => {
                if let Some(vlan_name) = key.strip_prefix("VLAN|")
                    && let Err(e) = self.process_vlan_config(vlan_name).await
                {
                    error!("Failed to process VLAN {}: {}", vlan_name, e);
                }
            }
            "DEL" | "DELETE" => {
                if let Some(vlan_name) = key.strip_prefix("VLAN|")
                    && let Err(e) = self.delete_vlan(vlan_name).await
                {
                    error!("Failed to delete VLAN {}: {}", vlan_name, e);
                }
            }
            _ => {
                warn!("Unknown operation: {}", operation);
            }
        }
    }

    /// Get statistics
    pub fn stats(&self) -> VlanOrchStats {
        VlanOrchStats {
            vlan_count: self.vlans.len(),
        }
    }
}

/// VLAN orchestration statistics
#[derive(Debug, Clone, Serialize)]
pub struct VlanOrchStats {
    pub vlan_count: usize,
}

/// Database subscriber implementation for VlanOrch
pub struct VlanOrchSubscriber {
    vlan_orch: Arc<VlanOrch>,
}

impl VlanOrchSubscriber {
    pub fn new(vlan_orch: Arc<VlanOrch>) -> Self {
        Self { vlan_orch }
    }
}

#[async_trait]
impl DbSubscriber for VlanOrchSubscriber {
    async fn on_message(&self, channel: String, message: String) {
        self.vlan_orch.handle_notification(&channel, &message).await;
    }

    async fn on_subscribe(&self, channel: String) {
        info!("VlanOrch subscribed to channel: {}", channel);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    #[ignore] // Requires running database
    async fn test_vlan_orch() {
        let db_client = Arc::new(DbClient::new("redis://127.0.0.1:6379").await.unwrap());
        let vlan_orch = VlanOrch::new(db_client.clone());

        // Create test VLAN in CONFIG_DB
        let config = VlanConfig {
            vlanid: 100,
            description: Some("Test VLAN".to_string()),
        };

        db_client
            .set(Database::Config, "VLAN|Vlan100", &config)
            .await
            .unwrap();

        // Sync VLANs
        vlan_orch.sync_vlans().await.unwrap();

        // Verify VLAN was created in APPL_DB
        let entry: VlanEntry = db_client
            .get(Database::Appl, "VLAN_TABLE:Vlan100")
            .await
            .unwrap();

        assert_eq!(entry.vlanid, 100);
        assert_eq!(entry.description, Some("Test VLAN".to_string()));
    }
}
