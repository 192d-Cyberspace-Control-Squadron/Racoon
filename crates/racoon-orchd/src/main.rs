//! Racoon Orchestration Daemon
//!
//! Translates configuration from CONFIG_DB to application-level entries

use anyhow::Result;
use racoon_db_client::{DbClient, DbSubscriberClient};
use racoon_orchd::{VlanOrch, VlanOrchSubscriber};
use std::sync::Arc;
use tracing::{error, info};

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize tracing
    tracing_subscriber::fmt()
        .with_target(false)
        .with_thread_ids(true)
        .with_level(true)
        .init();

    info!("Starting Racoon Orchestration Daemon (orchd)");

    // Get database URL from environment or use default
    let db_url =
        std::env::var("RACOON_DB_URL").unwrap_or_else(|_| "redis://127.0.0.1:6379".to_string());
    info!("Connecting to database: {}", db_url);

    // Create database client
    let db_client = Arc::new(DbClient::new(&db_url).await?);
    info!("Database client connected");

    // Create VLAN orchestration agent
    let vlan_orch = Arc::new(VlanOrch::new(db_client.clone()));

    // Start VLAN orchestration (load existing VLANs)
    vlan_orch.start().await?;
    info!("VLAN orchestration agent started");

    // Create subscriber for CONFIG_DB changes
    let subscriber_client = DbSubscriberClient::new(&db_url)?;
    let vlan_subscriber = Arc::new(VlanOrchSubscriber::new(vlan_orch.clone()));

    info!("Subscribing to CONFIG_DB VLAN channel");

    // Subscribe to VLAN configuration changes
    // This will block and process messages
    if let Err(e) = subscriber_client
        .subscribe(vec!["CONFIG_DB:VLAN".to_string()], vlan_subscriber)
        .await
    {
        error!("Subscription error: {}", e);
        return Err(e.into());
    }

    Ok(())
}
