# Racoon NOS API Reference

## Table of Contents

- [Database Client API](#database-client-api)
- [Orchestration APIs](#orchestration-apis)
- [Synchronization APIs](#synchronization-apis)
- [SAI Wrapper APIs](#sai-wrapper-apis)
- [Common Types](#common-types)

## Database Client API

### DbClient

Primary interface for database operations.

#### Creation

```rust
use racoon_db_client::DbClient;

let db_client = DbClient::new("redis://127.0.0.1:6379").await?;
```

#### Methods

##### `set<T: Serialize>`

Write a value to the database.

```rust
pub async fn set<T: Serialize>(
    &self,
    db: Database,
    key: &str,
    value: &T
) -> Result<()>
```

**Example**:
```rust
let vlan_config = VlanConfig {
    vlanid: 100,
    description: Some("Production".to_string()),
};

db_client.set(
    Database::Config,
    "VLAN|Vlan100",
    &vlan_config
).await?;
```

##### `get<T: DeserializeOwned>`

Read a value from the database.

```rust
pub async fn get<T: DeserializeOwned>(
    &self,
    db: Database,
    key: &str
) -> Result<T>
```

**Example**:
```rust
let vlan_config: VlanConfig = db_client.get(
    Database::Config,
    "VLAN|Vlan100"
).await?;
```

##### `del`

Delete a key from the database.

```rust
pub async fn del(&self, db: Database, key: &str) -> Result<()>
```

##### `exists`

Check if a key exists.

```rust
pub async fn exists(&self, db: Database, key: &str) -> Result<bool>
```

##### `keys`

Get all keys matching a pattern.

```rust
pub async fn keys(&self, db: Database, pattern: &str) -> Result<Vec<String>>
```

**Example**:
```rust
let vlan_keys = db_client.keys(Database::Config, "VLAN|Vlan*").await?;
```

##### `publish`

Publish a message to a channel.

```rust
pub async fn publish(&self, channel: &str, message: &str) -> Result<()>
```

**Example**:
```rust
let notification = serde_json::json!({
    "operation": "SET",
    "key": "Vlan100",
    "data": vlan_entry
});

db_client.publish("VLAN_TABLE", &notification.to_string()).await?;
```

### Database Enum

```rust
pub enum Database {
    Config = 4,    // User configuration
    Appl = 0,      // Application state
    Asic = 1,      // ASIC/hardware state
    State = 6,     // Runtime state
    Counters = 2,  // Statistics
}
```

### DbSubscriber Trait

Implement to receive database notifications.

```rust
#[async_trait]
pub trait DbSubscriber: Send + Sync {
    async fn on_message(&self, channel: String, message: String);
    async fn on_subscribe(&self, channel: String);
    async fn on_unsubscribe(&self, channel: String);
}
```

**Example**:
```rust
pub struct MySubscriber {
    // fields
}

#[async_trait]
impl DbSubscriber for MySubscriber {
    async fn on_message(&self, channel: String, message: String) {
        // Parse notification
        let notification: serde_json::Value = serde_json::from_str(&message)?;
        let operation = notification["operation"].as_str().unwrap_or("");
        let key = notification["key"].as_str().unwrap_or("");

        match operation {
            "SET" | "CREATE" => self.handle_create(key).await,
            "DEL" | "DELETE" => self.handle_delete(key).await,
            _ => {}
        }
    }

    async fn on_subscribe(&self, channel: String) {
        info!("Subscribed to {}", channel);
    }
}
```

### DbSubscriberClient

Client for subscribing to database channels.

```rust
let subscriber_client = DbSubscriberClient::new("redis://127.0.0.1:6379")?;
let subscriber = Arc::new(MySubscriber::new());

subscriber_client.subscribe(
    vec!["VLAN_TABLE".to_string()],
    subscriber
).await?;
```

## Orchestration APIs

### VlanOrch

VLAN orchestration agent.

#### Creation

```rust
use racoon_orchd::VlanOrch;

let vlan_orch = Arc::new(VlanOrch::new(db_client.clone()));
```

#### Methods

##### `start`

Start the orchestration agent (loads existing VLANs).

```rust
pub async fn start(&self) -> Result<()>
```

##### `stats`

Get orchestration statistics.

```rust
pub fn stats(&self) -> VlanOrchStats
```

**Returns**:
```rust
pub struct VlanOrchStats {
    pub vlan_count: usize,
}
```

### VlanOrchSubscriber

Subscriber for CONFIG_DB VLAN changes.

```rust
use racoon_orchd::VlanOrchSubscriber;

let subscriber = Arc::new(VlanOrchSubscriber::new(vlan_orch.clone()));

subscriber_client.subscribe(
    vec!["CONFIG_DB:VLAN".to_string()],
    subscriber
).await?;
```

### Data Types

#### VlanConfig (CONFIG_DB)

```rust
pub struct VlanConfig {
    pub vlanid: u16,
    pub description: Option<String>,
}
```

**Database Key**: `VLAN|Vlan{id}`

**Example JSON**:
```json
{
    "vlanid": 100,
    "description": "Production Network"
}
```

#### VlanEntry (APPL_DB)

```rust
pub struct VlanEntry {
    pub vlanid: u16,
    pub description: Option<String>,
}
```

**Database Key**: `VLAN_TABLE:Vlan{id}`

## Synchronization APIs

### VlanSync

VLAN synchronization to hardware.

#### Creation

```rust
use racoon_syncd::VlanSync;

let vlan_sync = Arc::new(VlanSync::new(
    db_client.clone(),
    vlan_api,
    switch_id,
));
```

**Parameters**:
- `db_client`: Database client
- `vlan_api`: SAI VLAN API wrapper
- `switch_id`: SAI switch object ID

#### Methods

##### `start`

Start the synchronization agent (loads existing VLANs).

```rust
pub async fn start(&self) -> Result<()>
```

##### `stats`

Get synchronization statistics.

```rust
pub fn stats(&self) -> VlanSyncStats
```

**Returns**:
```rust
pub struct VlanSyncStats {
    pub vlan_count: usize,
}
```

### VlanSyncSubscriber

Subscriber for APPL_DB VLAN changes.

```rust
use racoon_syncd::VlanSyncSubscriber;

let subscriber = Arc::new(VlanSyncSubscriber::new(vlan_sync.clone()));

subscriber_client.subscribe(
    vec!["VLAN_TABLE".to_string()],
    subscriber
).await?;
```

## SAI Wrapper APIs

### SaiAdapter

Main SAI adapter for dynamic library loading.

#### Creation

```rust
use racoon_sai::SaiAdapter;

let adapter = SaiAdapter::load("/usr/lib/libsai.so")?;
```

#### Methods

##### `get_vlan_api`

Get VLAN API table.

```rust
pub fn get_vlan_api(&self) -> &sai_vlan_api_t
```

##### `get_port_api`

Get Port API table.

```rust
pub fn get_port_api(&self) -> &sai_port_api_t
```

##### `get_fdb_api`

Get FDB API table.

```rust
pub fn get_fdb_api(&self) -> &sai_fdb_api_t
```

### VlanApi

Type-safe VLAN API wrapper.

#### Creation

```rust
use racoon_sai::VlanApi;

let vlan_api_table = adapter.get_vlan_api() as *const _;
let vlan_api = Arc::new(VlanApi::new(vlan_api_table));
```

#### Methods

##### `create_vlan`

Create a VLAN in hardware.

```rust
pub fn create_vlan(
    &self,
    switch_id: SaiOid,
    vlan_id: VlanId
) -> Result<SaiOid>
```

**Parameters**:
- `switch_id`: SAI switch object ID
- `vlan_id`: VLAN ID (validated 1-4094)

**Returns**: SAI object ID (OID) for the created VLAN

**Example**:
```rust
let vlan_id = VlanId::new(100).unwrap();
let vlan_oid = vlan_api.create_vlan(switch_id, vlan_id)?;
println!("Created VLAN with OID: 0x{:x}", vlan_oid);
```

##### `remove_vlan`

Remove a VLAN from hardware.

```rust
pub fn remove_vlan(&self, vlan_oid: SaiOid) -> Result<()>
```

**Example**:
```rust
vlan_api.remove_vlan(vlan_oid)?;
```

##### `create_vlan_member`

Add a port to a VLAN.

```rust
pub fn create_vlan_member(
    &self,
    vlan_oid: SaiOid,
    bridge_port_id: SaiOid,
    tagging_mode: VlanTaggingMode
) -> Result<SaiOid>
```

**Parameters**:
- `vlan_oid`: VLAN object ID
- `bridge_port_id`: Bridge port object ID
- `tagging_mode`: Tagged or untagged

**Returns**: VLAN member object ID

##### `remove_vlan_member`

Remove a port from a VLAN.

```rust
pub fn remove_vlan_member(&self, member_oid: SaiOid) -> Result<()>
```

### VlanTaggingMode

```rust
pub enum VlanTaggingMode {
    Untagged = 0,
    Tagged = 1,
}
```

## Common Types

### VlanId

Type-safe VLAN ID with validation.

```rust
pub struct VlanId(u16);

impl VlanId {
    /// Create a new VLAN ID (validates range 1-4094)
    pub fn new(id: u16) -> Option<Self> {
        if (1..=4094).contains(&id) {
            Some(VlanId(id))
        } else {
            None
        }
    }

    /// Get the raw VLAN ID value
    pub fn get(&self) -> u16 {
        self.0
    }
}
```

**Example**:
```rust
// Valid VLAN ID
let vlan_id = VlanId::new(100).unwrap();
assert_eq!(vlan_id.get(), 100);

// Invalid VLAN IDs
assert!(VlanId::new(0).is_none());
assert!(VlanId::new(4095).is_none());

// Pattern matching
match VlanId::new(user_input) {
    Some(vlan_id) => println!("Valid VLAN: {}", vlan_id.get()),
    None => println!("Invalid VLAN ID"),
}

// Use with ? operator
let vlan_id = VlanId::new(config.vlanid)
    .ok_or(RacoonError::InvalidVlanId(config.vlanid))?;
```

### SaiOid

SAI object identifier.

```rust
pub type SaiOid = u64;
```

**Format**: `[object_type:8][switch_id:8][index:48]`

**Example**:
```rust
let vlan_oid: SaiOid = 0x260000000004d2;
//   0x26 = VLAN object type
//   0x00000000 = switch 0
//   0x04d2 = VLAN 1234

println!("VLAN OID: 0x{:x}", vlan_oid);
```

### Result Type

```rust
pub type Result<T> = std::result::Result<T, RacoonError>;
```

### RacoonError

```rust
pub enum RacoonError {
    Database(String),
    Serialization(#[from] serde_json::Error),
    InvalidVlanId(u16),
    LibraryLoad(String),
    SaiError(i32),
}

impl std::fmt::Display for RacoonError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            RacoonError::Database(msg) => write!(f, "Database error: {}", msg),
            RacoonError::Serialization(e) => write!(f, "Serialization error: {}", e),
            RacoonError::InvalidVlanId(id) => write!(f, "Invalid VLAN ID: {}", id),
            RacoonError::LibraryLoad(msg) => write!(f, "Library load error: {}", msg),
            RacoonError::SaiError(code) => write!(f, "SAI error: {}", code),
        }
    }
}
```

**Example**:
```rust
use racoon_common::{Result, RacoonError};

fn process_vlan(vlan_id: u16) -> Result<()> {
    let vlan_id = VlanId::new(vlan_id)
        .ok_or(RacoonError::InvalidVlanId(vlan_id))?;

    // Process VLAN
    Ok(())
}

match process_vlan(100) {
    Ok(()) => println!("Success"),
    Err(RacoonError::InvalidVlanId(id)) => {
        eprintln!("VLAN ID {} is out of range (1-4094)", id);
    }
    Err(e) => eprintln!("Error: {}", e),
}
```

### SaiStatus

SAI status code wrapper.

```rust
pub struct SaiStatus(pub i32);

impl SaiStatus {
    pub fn to_result(self) -> Result<()> {
        if self.0 == SAI_STATUS_SUCCESS {
            Ok(())
        } else {
            Err(RacoonError::SaiError(self.0))
        }
    }

    pub fn is_success(&self) -> bool {
        self.0 == SAI_STATUS_SUCCESS
    }
}

impl From<i32> for SaiStatus {
    fn from(code: i32) -> Self {
        SaiStatus(code)
    }
}
```

**Example**:
```rust
let status = SaiStatus::from(sai_status_code);
status.to_result()?; // Convert to Result
```

## Database Schema Reference

### CONFIG_DB (Database 4)

User configuration.

| Key Pattern | Type | Description |
|------------|------|-------------|
| `VLAN\|Vlan{id}` | VlanConfig | VLAN configuration |
| `VLAN_MEMBER\|Vlan{id}\|{port}` | VlanMemberConfig | VLAN membership |
| `PORT\|{name}` | PortConfig | Port configuration |
| `LAG\|PortChannel{id}` | LagConfig | LAG configuration |

### APPL_DB (Database 0)

Application state.

| Key Pattern | Type | Description |
|------------|------|-------------|
| `VLAN_TABLE:{name}` | VlanEntry | VLAN entry |
| `VLAN_MEMBER_TABLE:{vlan}:{port}` | VlanMemberEntry | VLAN membership |
| `PORT_TABLE:{name}` | PortEntry | Port entry |
| `FDB_TABLE:{vlan}:{mac}` | FdbEntry | FDB entry |

### ASIC_DB (Database 1)

Hardware state.

| Key Pattern | Type | Description |
|------------|------|-------------|
| `ASIC_STATE:SAI_OBJECT_TYPE_VLAN:{oid}` | JSON | VLAN state |
| `ASIC_STATE:SAI_OBJECT_TYPE_BRIDGE_PORT:{oid}` | JSON | Bridge port state |

**Example ASIC_DB Entry**:
```json
{
    "vlanid": 100,
    "oid": "0x260000000004d2"
}
```

## Notification Format

### Standard Notification

```json
{
    "operation": "SET" | "CREATE" | "DEL" | "DELETE",
    "table": "VLAN_TABLE",
    "key": "Vlan100",
    "data": {
        "vlanid": 100,
        "description": "Production Network"
    }
}
```

### Channel Names

- `CONFIG_DB:VLAN` - CONFIG_DB VLAN changes
- `VLAN_TABLE` - APPL_DB VLAN changes
- `PORT_TABLE` - APPL_DB port changes
- `FDB_TABLE` - APPL_DB FDB changes

## Error Handling Patterns

### Using ? Operator

```rust
async fn process_vlan(db_client: &DbClient) -> Result<()> {
    let config: VlanConfig = db_client
        .get(Database::Config, "VLAN|Vlan100")
        .await?;

    let vlan_id = VlanId::new(config.vlanid)
        .ok_or(RacoonError::InvalidVlanId(config.vlanid))?;

    Ok(())
}
```

### Pattern Matching

```rust
match db_client.get(Database::Config, key).await {
    Ok(config) => process_config(config).await,
    Err(RacoonError::Database(msg)) if msg.contains("key not found") => {
        warn!("Key not found: {}", key);
        Ok(())
    }
    Err(e) => Err(e),
}
```

### Logging Errors

```rust
if let Err(e) = process_vlan(vlan_name).await {
    error!("Failed to process VLAN {}: {}", vlan_name, e);
    // Continue processing other VLANs
}
```

## Concurrency Patterns

### Shared State with Arc

```rust
let vlan_orch = Arc::new(VlanOrch::new(db_client.clone()));

// Clone Arc for different tasks
let orch1 = vlan_orch.clone();
let orch2 = vlan_orch.clone();
```

### DashMap for Concurrent Access

```rust
use dashmap::DashMap;

let vlans: DashMap<VlanId, VlanEntry> = DashMap::new();

// Insert
vlans.insert(vlan_id, entry);

// Read
if let Some(entry) = vlans.get(&vlan_id) {
    println!("VLAN: {:?}", entry);
}

// Remove
vlans.remove(&vlan_id);

// Iterate
for entry in vlans.iter() {
    println!("{:?}", entry);
}
```

## Best Practices

### Always Validate Input

```rust
let vlan_id = VlanId::new(user_input)
    .ok_or(RacoonError::InvalidVlanId(user_input))?;
```

### Use Type-Safe Wrappers

```rust
// Good: Type-safe
let vlan_id = VlanId::new(100)?;

// Bad: Raw integers
let vlan_id: u16 = 100;
```

### Handle All Error Cases

```rust
match operation {
    "SET" | "CREATE" => handle_create(key).await?,
    "DEL" | "DELETE" => handle_delete(key).await?,
    _ => {
        warn!("Unknown operation: {}", operation);
        return Ok(()); // Don't fail on unknown operations
    }
}
```

### Log Important Events

```rust
info!("Creating VLAN {} in hardware", vlan_id.get());
debug!("SAI OID: 0x{:x}", vlan_oid);
warn!("VLAN {} already exists", vlan_id.get());
error!("Failed to program VLAN: {}", e);
```
