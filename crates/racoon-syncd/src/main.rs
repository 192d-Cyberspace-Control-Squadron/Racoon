//! Racoon SAI Synchronization Daemon
//!
//! Synchronizes database state to hardware via SAI

use anyhow::Result;
use racoon_db_client::{DbClient, DbSubscriberClient};
use racoon_sai::{SaiAdapter, VlanApi};
use racoon_syncd::{VlanSync, VlanSyncSubscriber};
use std::sync::Arc;
use tracing::{error, info, warn};

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize tracing
    tracing_subscriber::fmt()
        .with_target(false)
        .with_thread_ids(true)
        .with_level(true)
        .init();

    info!("Starting Racoon SAI Synchronization Daemon (syncd)");

    // Get database URL from environment or use default
    let db_url =
        std::env::var("RACOON_DB_URL").unwrap_or_else(|_| "redis://127.0.0.1:6379".to_string());
    info!("Connecting to database: {}", db_url);

    // Create database client
    let db_client = Arc::new(DbClient::new(&db_url).await?);
    info!("Database client connected");

    // Get SAI library path from environment
    let sai_lib_path =
        std::env::var("SAI_LIBRARY_PATH").unwrap_or_else(|_| "/usr/lib/libsai.so".to_string());

    info!("Loading SAI library from: {}", sai_lib_path);

    // Initialize SAI adapter
    let sai_adapter = match SaiAdapter::load(&sai_lib_path) {
        Ok(adapter) => {
            info!("SAI adapter initialized successfully");
            adapter
        }
        Err(e) => {
            warn!("Failed to load SAI library: {}", e);
            warn!("Running in test mode without hardware access");
            warn!("Set SAI_LIBRARY_PATH environment variable to enable hardware programming");
            // In production, we'd return an error here
            // For now, we'll continue to allow testing the database flow
            return Err(anyhow::anyhow!("Failed to load SAI library: {}", e));
        }
    };

    // Get switch ID (for real hardware, this would come from SAI initialization)
    // For now, use a dummy switch ID
    let switch_id: u64 = 0x21000000000000;
    info!("Using switch ID: 0x{:x}", switch_id);

    // Create VLAN API from the adapter's VLAN API table
    let vlan_api_table = sai_adapter.get_vlan_api() as *const _;
    let vlan_api = Arc::new(VlanApi::new(vlan_api_table));

    // Create VLAN synchronization agent
    let vlan_sync = Arc::new(VlanSync::new(db_client.clone(), vlan_api, switch_id));

    // Start VLAN synchronization (load existing VLANs from APPL_DB)
    vlan_sync.start().await?;
    info!("VLAN synchronization agent started");

    // Create subscriber for APPL_DB changes
    let subscriber_client = DbSubscriberClient::new(&db_url)?;
    let vlan_subscriber = Arc::new(VlanSyncSubscriber::new(vlan_sync.clone()));

    info!("Subscribing to APPL_DB VLAN_TABLE channel");

    // Subscribe to VLAN table changes
    // This will block and process messages
    if let Err(e) = subscriber_client
        .subscribe(vec!["VLAN_TABLE".to_string()], vlan_subscriber)
        .await
    {
        error!("Subscription error: {}", e);
        return Err(e.into());
    }

    Ok(())
}
