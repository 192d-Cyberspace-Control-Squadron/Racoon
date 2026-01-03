//! Database client for Racoon NOS
//!
//! Provides async interface to Valkey database with pub/sub support

use async_trait::async_trait;
use futures::StreamExt;
use racoon_common::Result;
use redis::{AsyncCommands, Client, aio::ConnectionManager};
use serde::{Serialize, de::DeserializeOwned};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{debug, info};

/// Database identifiers
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Database {
    Config = 4,
    Appl = 0,
    Asic = 1,
    State = 6,
    Counters = 2,
}

/// Database client with connection pooling
pub struct DbClient {
    client: Client,
    connections: Arc<RwLock<HashMap<Database, ConnectionManager>>>,
}

impl DbClient {
    /// Create new database client
    pub async fn new(url: &str) -> Result<Self> {
        info!("Connecting to Valkey database at {}", url);
        let client =
            Client::open(url).map_err(|e| racoon_common::RacoonError::Database(e.to_string()))?;

        Ok(Self {
            client,
            connections: Arc::new(RwLock::new(HashMap::new())),
        })
    }

    /// Get connection for specific database
    async fn get_connection(&self, db: Database) -> Result<ConnectionManager> {
        // Check if we already have a connection
        {
            let connections = self.connections.read().await;
            if let Some(conn) = connections.get(&db) {
                return Ok(conn.clone());
            }
        }

        // Create new connection
        debug!("Creating new connection for database {:?}", db);
        let mut conn = ConnectionManager::new(self.client.clone())
            .await
            .map_err(|e| racoon_common::RacoonError::Database(e.to_string()))?;

        // Select database
        let _: () = redis::cmd("SELECT")
            .arg(db as i64)
            .query_async(&mut conn)
            .await
            .map_err(|e| racoon_common::RacoonError::Database(e.to_string()))?;

        // Store connection
        let mut connections = self.connections.write().await;
        connections.insert(db, conn.clone());

        Ok(conn)
    }

    /// Set a value in the database
    pub async fn set<T: Serialize>(&self, db: Database, key: &str, value: &T) -> Result<()> {
        let json = serde_json::to_string(value)?;

        let mut conn = self.get_connection(db).await?;
        let _: () = conn
            .set(key, json)
            .await
            .map_err(|e| racoon_common::RacoonError::Database(e.to_string()))?;

        debug!("SET {} in {:?}: {}", key, db, std::any::type_name::<T>());
        Ok(())
    }

    /// Get a value from the database
    pub async fn get<T: DeserializeOwned>(&self, db: Database, key: &str) -> Result<T> {
        let mut conn = self.get_connection(db).await?;
        let json: String = conn
            .get(key)
            .await
            .map_err(|e| racoon_common::RacoonError::Database(e.to_string()))?;

        let value = serde_json::from_str(&json)?;

        debug!("GET {} from {:?}: {}", key, db, std::any::type_name::<T>());
        Ok(value)
    }

    /// Delete a key from the database
    pub async fn del(&self, db: Database, key: &str) -> Result<()> {
        let mut conn = self.get_connection(db).await?;
        let _: () = conn
            .del(key)
            .await
            .map_err(|e| racoon_common::RacoonError::Database(e.to_string()))?;

        debug!("DEL {} from {:?}", key, db);
        Ok(())
    }

    /// Check if key exists
    pub async fn exists(&self, db: Database, key: &str) -> Result<bool> {
        let mut conn = self.get_connection(db).await?;
        let exists: bool = conn
            .exists(key)
            .await
            .map_err(|e| racoon_common::RacoonError::Database(e.to_string()))?;

        Ok(exists)
    }

    /// Get all keys matching a pattern
    pub async fn keys(&self, db: Database, pattern: &str) -> Result<Vec<String>> {
        let mut conn = self.get_connection(db).await?;
        let keys: Vec<String> = conn
            .keys(pattern)
            .await
            .map_err(|e| racoon_common::RacoonError::Database(e.to_string()))?;

        Ok(keys)
    }

    /// Set multiple hash fields
    pub async fn hset_multiple(
        &self,
        db: Database,
        key: &str,
        fields: &HashMap<String, String>,
    ) -> Result<()> {
        let mut conn = self.get_connection(db).await?;
        for (field, value) in fields {
            let _: () = conn
                .hset(key, field, value)
                .await
                .map_err(|e| racoon_common::RacoonError::Database(e.to_string()))?;
        }

        debug!("HSET {} in {:?}: {} fields", key, db, fields.len());
        Ok(())
    }

    /// Get all hash fields
    pub async fn hgetall(&self, db: Database, key: &str) -> Result<HashMap<String, String>> {
        let mut conn = self.get_connection(db).await?;
        let fields: HashMap<String, String> = conn
            .hgetall(key)
            .await
            .map_err(|e| racoon_common::RacoonError::Database(e.to_string()))?;

        Ok(fields)
    }

    /// Publish a message to a channel
    pub async fn publish(&self, channel: &str, message: &str) -> Result<()> {
        let mut conn = self.get_connection(Database::Appl).await?;
        let _: () = conn
            .publish(channel, message)
            .await
            .map_err(|e| racoon_common::RacoonError::Database(e.to_string()))?;

        debug!("PUBLISH to {}: {}", channel, message);
        Ok(())
    }
}

/// Subscriber trait for database pub/sub
#[async_trait]
pub trait DbSubscriber: Send + Sync {
    /// Handle incoming message
    async fn on_message(&self, channel: String, message: String);

    /// Handle subscription confirmation
    async fn on_subscribe(&self, channel: String) {
        info!("Subscribed to channel: {}", channel);
    }

    /// Handle unsubscription confirmation
    async fn on_unsubscribe(&self, channel: String) {
        info!("Unsubscribed from channel: {}", channel);
    }
}

/// Database subscriber client
pub struct DbSubscriberClient {
    client: Client,
}

impl DbSubscriberClient {
    /// Create new subscriber client
    pub fn new(url: &str) -> Result<Self> {
        let client =
            Client::open(url).map_err(|e| racoon_common::RacoonError::Database(e.to_string()))?;

        Ok(Self { client })
    }

    /// Subscribe to channels and process messages
    pub async fn subscribe<S: DbSubscriber>(
        &self,
        channels: Vec<String>,
        subscriber: Arc<S>,
    ) -> Result<()> {
        let mut pubsub = self
            .client
            .get_async_pubsub()
            .await
            .map_err(|e| racoon_common::RacoonError::Database(e.to_string()))?;

        // Subscribe to all channels
        for channel in &channels {
            pubsub
                .subscribe(channel)
                .await
                .map_err(|e| racoon_common::RacoonError::Database(e.to_string()))?;
            info!("Subscribing to channel: {}", channel);
        }

        // Process messages
        loop {
            let msg = pubsub.on_message().next().await.ok_or_else(|| {
                racoon_common::RacoonError::Database("Subscription closed".into())
            })?;

            let channel = msg.get_channel_name().to_string();
            let payload: String = msg
                .get_payload()
                .map_err(|e| racoon_common::RacoonError::Database(e.to_string()))?;

            subscriber.on_message(channel, payload).await;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    #[ignore] // Requires running Valkey/Redis instance
    async fn test_db_client() {
        let client = DbClient::new("redis://127.0.0.1:6379").await.unwrap();

        // Test set/get
        client
            .set(Database::Config, "test_key", &"test_value")
            .await
            .unwrap();
        let value: String = client.get(Database::Config, "test_key").await.unwrap();
        assert_eq!(value, "test_value");

        // Test delete
        client.del(Database::Config, "test_key").await.unwrap();
        assert!(!client.exists(Database::Config, "test_key").await.unwrap());
    }
}
