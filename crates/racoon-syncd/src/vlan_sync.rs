//! VLAN Synchronization
//!
//! Synchronizes VLAN entries from APPL_DB to hardware via SAI

use async_trait::async_trait;
use dashmap::DashMap;
use racoon_common::{Result, SaiOid, VlanId};
use racoon_db_client::{Database, DbClient, DbSubscriber};
use racoon_sai::VlanApi;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tracing::{debug, error, info, warn};

/// VLAN entry from APPL_DB
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VlanEntry {
    pub vlanid: u16,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
}

/// VLAN synchronization state
#[derive(Debug, Clone)]
struct VlanState {
    _vlan_id: VlanId,
    /// SAI object ID for the VLAN
    sai_oid: SaiOid,
}

/// VLAN Synchronization Agent
pub struct VlanSync {
    db_client: Arc<DbClient>,
    vlan_api: Arc<VlanApi>,
    switch_id: SaiOid,
    /// Track VLANs we've programmed
    vlans: DashMap<VlanId, VlanState>,
}

impl VlanSync {
    /// Create new VLAN sync agent
    pub fn new(db_client: Arc<DbClient>, vlan_api: Arc<VlanApi>, switch_id: SaiOid) -> Self {
        Self {
            db_client,
            vlan_api,
            switch_id,
            vlans: DashMap::new(),
        }
    }

    /// Start the sync agent
    pub async fn start(&self) -> Result<()> {
        info!("Starting VLAN synchronization agent");

        // Load existing VLANs from APPL_DB
        self.sync_vlans().await?;

        info!("VLAN synchronization agent started");
        Ok(())
    }

    /// Sync all VLANs from APPL_DB to SAI
    async fn sync_vlans(&self) -> Result<()> {
        info!("Syncing VLANs from APPL_DB to SAI");

        let keys = self.db_client.keys(Database::Appl, "VLAN_TABLE:*").await?;

        for key in keys {
            if let Some(vlan_name) = key.strip_prefix("VLAN_TABLE:") {
                match self.create_vlan(vlan_name).await {
                    Ok(_) => debug!("Synced VLAN: {}", vlan_name),
                    Err(e) => warn!("Failed to sync VLAN {}: {}", vlan_name, e),
                }
            }
        }

        info!("Synced {} VLANs to SAI", self.vlans.len());
        Ok(())
    }

    /// Create VLAN in hardware via SAI
    async fn create_vlan(&self, vlan_name: &str) -> Result<()> {
        let appl_key = format!("VLAN_TABLE:{}", vlan_name);

        // Get VLAN entry from APPL_DB
        let entry: VlanEntry = self.db_client.get(Database::Appl, &appl_key).await?;

        let vlan_id = VlanId::new(entry.vlanid)
            .ok_or(racoon_common::RacoonError::InvalidVlanId(entry.vlanid))?;

        // Check if already created
        if self.vlans.contains_key(&vlan_id) {
            debug!("VLAN {} already exists in SAI", vlan_id.get());
            return Ok(());
        }

        // Create VLAN via SAI
        info!(
            "Creating VLAN {} in hardware (switch_id: 0x{:x})",
            vlan_id.get(),
            self.switch_id
        );
        let vlan_oid = self.vlan_api.create_vlan(self.switch_id, vlan_id)?;

        info!(
            "Created VLAN {} in SAI with OID: 0x{:x}",
            vlan_id.get(),
            vlan_oid
        );

        // Store state
        let state = VlanState {
            _vlan_id: vlan_id,
            sai_oid: vlan_oid,
        };
        self.vlans.insert(vlan_id, state.clone());

        // Write to ASIC_DB
        let asic_key = format!("ASIC_STATE:SAI_OBJECT_TYPE_VLAN:0x{:x}", vlan_oid);
        let asic_value = serde_json::json!({
            "vlanid": entry.vlanid,
            "oid": format!("0x{:x}", vlan_oid)
        });

        self.db_client
            .set(Database::Asic, &asic_key, &asic_value)
            .await?;

        info!(
            "Programmed VLAN {} to hardware (OID: 0x{:x})",
            vlan_id.get(),
            vlan_oid
        );

        Ok(())
    }

    /// Delete VLAN from hardware
    async fn delete_vlan(&self, vlan_name: &str) -> Result<()> {
        // Parse VLAN ID from name (Vlan100 -> 100)
        let vlan_id_str = vlan_name.strip_prefix("Vlan").unwrap_or(vlan_name);
        let vlan_id_num = vlan_id_str
            .parse::<u16>()
            .map_err(|_| racoon_common::RacoonError::InvalidVlanId(0))?;
        let vlan_id = VlanId::new(vlan_id_num)
            .ok_or(racoon_common::RacoonError::InvalidVlanId(vlan_id_num))?;

        // Get state
        let state = match self.vlans.get(&vlan_id) {
            Some(s) => s.clone(),
            None => {
                warn!("VLAN {} not found in tracking", vlan_id.get());
                return Ok(());
            }
        };

        // Delete from SAI
        info!("Deleting VLAN {} from hardware", vlan_id.get());
        self.vlan_api.remove_vlan(state.sai_oid)?;

        // Remove from tracking
        self.vlans.remove(&vlan_id);

        // Remove from ASIC_DB
        let asic_key = format!("ASIC_STATE:SAI_OBJECT_TYPE_VLAN:0x{:x}", state.sai_oid);
        self.db_client.del(Database::Asic, &asic_key).await?;

        info!("Deleted VLAN {} from hardware", vlan_id.get());

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
                if let Err(e) = self.create_vlan(key).await {
                    error!("Failed to create VLAN {}: {}", key, e);
                }
            }
            "DEL" | "DELETE" => {
                if let Err(e) = self.delete_vlan(key).await {
                    error!("Failed to delete VLAN {}: {}", key, e);
                }
            }
            _ => {
                warn!("Unknown operation: {}", operation);
            }
        }
    }

    /// Get statistics
    pub fn stats(&self) -> VlanSyncStats {
        VlanSyncStats {
            vlan_count: self.vlans.len(),
        }
    }
}

/// VLAN sync statistics
#[derive(Debug, Clone, Serialize)]
pub struct VlanSyncStats {
    pub vlan_count: usize,
}

/// Database subscriber implementation for VlanSync
pub struct VlanSyncSubscriber {
    vlan_sync: Arc<VlanSync>,
}

impl VlanSyncSubscriber {
    pub fn new(vlan_sync: Arc<VlanSync>) -> Self {
        Self { vlan_sync }
    }
}

#[async_trait]
impl DbSubscriber for VlanSyncSubscriber {
    async fn on_message(&self, channel: String, message: String) {
        self.vlan_sync.handle_notification(&channel, &message).await;
    }

    async fn on_subscribe(&self, channel: String) {
        info!("VlanSync subscribed to channel: {}", channel);
    }
}
