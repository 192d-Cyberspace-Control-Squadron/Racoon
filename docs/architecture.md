# Racoon NOS Architecture

## Overview

Racoon NOS is a network operating system written in Rust, following the SONiC (Software for Open Networking in the Cloud) architecture. It provides a modern, type-safe implementation of network switch functionality with a database-centric microservices architecture.

## Core Architecture

### Database-Centric Design

Racoon NOS uses Valkey (Redis fork) as a central state database, divided into multiple logical databases:

```
┌─────────────────────────────────────────────────────┐
│                  Valkey Database                     │
├─────────────────────────────────────────────────────┤
│ DB 4: CONFIG_DB    - User configuration             │
│ DB 0: APPL_DB      - Application state              │
│ DB 1: ASIC_DB      - Hardware/ASIC state            │
│ DB 6: STATE_DB     - Runtime state and status       │
│ DB 2: COUNTERS_DB  - Statistics and counters        │
└─────────────────────────────────────────────────────┘
```

### Microservices Architecture

```
┌────────────────────────────────────────────────────────────┐
│                       User Space                           │
├────────────────────────────────────────────────────────────┤
│                                                            │
│  ┌──────────┐  ┌──────────┐  ┌──────────┐  ┌──────────┐ │
│  │   CLI    │  │  mgmtd   │  │ configd  │  │  eventd  │ │
│  └────┬─────┘  └────┬─────┘  └────┬─────┘  └────┬─────┘ │
│       │             │              │              │        │
│       └─────────────┴──────────────┴──────────────┘        │
│                          │                                  │
│                          ▼                                  │
│                  ┌──────────────┐                          │
│                  │  CONFIG_DB   │                          │
│                  └──────┬───────┘                          │
│                         │                                   │
│                         ▼                                   │
│  ┌──────────────────────────────────────────────┐         │
│  │            Orchestration Layer               │         │
│  │  ┌──────────┐  ┌──────────┐  ┌──────────┐  │         │
│  │  │  orchd   │  │  portd   │  │  fdbsyncd│  │         │
│  │  │(VlanOrch)│  │          │  │          │  │         │
│  │  └────┬─────┘  └────┬─────┘  └────┬─────┘  │         │
│  └───────┼─────────────┼─────────────┼─────────┘         │
│          │             │             │                     │
│          └─────────────┴─────────────┘                     │
│                        │                                    │
│                        ▼                                    │
│                  ┌──────────────┐                          │
│                  │   APPL_DB    │                          │
│                  └──────┬───────┘                          │
│                         │                                   │
│                         ▼                                   │
│  ┌──────────────────────────────────────────────┐         │
│  │         Synchronization Layer                │         │
│  │  ┌──────────┐  ┌──────────┐  ┌──────────┐  │         │
│  │  │  syncd   │  │  portd   │  │  fdbsyncd│  │         │
│  │  │(VlanSync)│  │          │  │          │  │         │
│  │  └────┬─────┘  └────┬─────┘  └────┬─────┘  │         │
│  └───────┼─────────────┼─────────────┼─────────┘         │
│          │             │             │                     │
│          └─────────────┴─────────────┘                     │
│                        │                                    │
│                        ▼                                    │
│                  ┌──────────────┐                          │
│                  │   ASIC_DB    │                          │
│                  └──────┬───────┘                          │
│                         │                                   │
└─────────────────────────┼───────────────────────────────────┘
                          │
                          ▼
┌─────────────────────────────────────────────────────────────┐
│                    Hardware Layer                           │
├─────────────────────────────────────────────────────────────┤
│                         SAI                                 │
│            (Switch Abstraction Interface)                   │
│                          │                                   │
│                          ▼                                   │
│                  ┌──────────────┐                           │
│                  │ Switch ASIC  │                           │
│                  │  (Hardware)  │                           │
│                  └──────────────┘                           │
└─────────────────────────────────────────────────────────────┘
```

## Component Layers

### 1. Configuration Layer

**Purpose**: User-facing interfaces for network configuration

**Components**:
- **CLI** (`racoon-cli`): Command-line interface
- **mgmtd** (`racoon-mgmtd`): Management daemon
- **configd** (`racoon-configd`): Configuration daemon
- **eventd** (`racoon-eventd`): Event handling daemon

**Database**: CONFIG_DB (DB 4)

### 2. Orchestration Layer

**Purpose**: Translates user configuration into application-level state

**Components**:
- **orchd** (`racoon-orchd`): Main orchestration daemon
  - `VlanOrch`: VLAN configuration translation
  - Future: `PortOrch`, `LagOrch`, `RouteOrch`, etc.

**Flow**: CONFIG_DB → Orchestration → APPL_DB

**Key Features**:
- Schema validation
- Dependency resolution
- State normalization
- Pub/sub notifications

### 3. Synchronization Layer

**Purpose**: Programs hardware state via SAI

**Components**:
- **syncd** (`racoon-syncd`): SAI synchronization daemon
  - `VlanSync`: VLAN hardware programming
  - Future: `PortSync`, `FdbSync`, etc.

**Flow**: APPL_DB → Synchronization → SAI → Hardware

**Key Features**:
- SAI interaction
- Hardware state tracking
- ASIC_DB state reflection
- Error recovery

### 4. Hardware Abstraction Layer

**Purpose**: Vendor-independent hardware interface

**Components**:
- **SAI Adapter** (`racoon-sai`): Rust SAI bindings
  - `VlanApi`: VLAN operations
  - `PortApi`: Port operations
  - `FdbApi`: FDB operations
  - `LagApi`: LAG operations
  - `BridgeApi`: Bridge operations

**Features**:
- Dynamic library loading
- Type-safe API wrappers
- Auto-generated bindings (bindgen)

## Data Flow Example: VLAN Creation

### Step-by-Step Flow

```
1. User Input
   ↓
   CLI: vlan create 100 "Production Network"
   ↓

2. CONFIG_DB Write
   ↓
   Key: VLAN|Vlan100
   Value: {"vlanid": 100, "description": "Production Network"}
   ↓

3. Pub/Sub Notification
   ↓
   Channel: CONFIG_DB:VLAN
   Message: {"operation": "SET", "key": "VLAN|Vlan100", ...}
   ↓

4. Orchestration (orchd)
   ↓
   VlanOrch receives notification
   - Validates VLAN ID (1-4094)
   - Creates APPL_DB entry
   ↓
   APPL_DB Write
   Key: VLAN_TABLE:Vlan100
   Value: {"vlanid": 100, "description": "Production Network"}
   ↓

5. Pub/Sub Notification
   ↓
   Channel: VLAN_TABLE
   Message: {"operation": "SET", "key": "Vlan100", ...}
   ↓

6. Synchronization (syncd)
   ↓
   VlanSync receives notification
   - Calls SAI VLAN create API
   - Receives SAI OID (Object ID)
   ↓
   SAI Call
   create_vlan(switch_id, vlan_id) → vlan_oid
   ↓
   ASIC_DB Write
   Key: ASIC_STATE:SAI_OBJECT_TYPE_VLAN:0x260000000004d2
   Value: {"vlanid": 100, "oid": "0x260000000004d2"}
   ↓

7. Hardware Programming
   ↓
   SAI driver programs switch ASIC
   VLAN 100 now exists in hardware
```

### Reverse Flow: VLAN Deletion

```
1. User Input: vlan delete 100
   ↓
2. CONFIG_DB: DEL VLAN|Vlan100
   ↓
3. orchd: DEL VLAN_TABLE:Vlan100
   ↓
4. syncd: SAI remove_vlan(oid)
   ↓
5. ASIC_DB: DEL ASIC_STATE:...
   ↓
6. Hardware: VLAN removed from ASIC
```

## Communication Patterns

### Pub/Sub Architecture

All inter-daemon communication uses Redis pub/sub:

```rust
// Subscriber pattern
#[async_trait]
pub trait DbSubscriber: Send + Sync {
    async fn on_message(&self, channel: String, message: String);
    async fn on_subscribe(&self, channel: String);
    async fn on_unsubscribe(&self, channel: String);
}

// Notification format
{
    "operation": "SET" | "DEL" | "CREATE" | "DELETE",
    "table": "VLAN_TABLE",
    "key": "Vlan100",
    "data": { /* entry data */ }
}
```

### State Tracking

Each daemon maintains in-memory state using concurrent data structures:

```rust
// VlanOrch state
vlans: DashMap<VlanId, VlanEntry>

// VlanSync state
vlans: DashMap<VlanId, VlanState>
```

## Type Safety

### VLAN ID Validation

```rust
pub struct VlanId(u16);

impl VlanId {
    pub fn new(id: u16) -> Option<Self> {
        if (1..=4094).contains(&id) {
            Some(VlanId(id))
        } else {
            None
        }
    }
}
```

### SAI Object IDs

```rust
pub type SaiOid = u64;

// OID format: [object_type:8][switch_id:8][index:48]
// Example: 0x260000000004d2
//   0x26 = VLAN object type
//   0x00000000 = switch 0
//   0x04d2 = VLAN 1234
```

## Error Handling

### Error Types

```rust
pub enum RacoonError {
    Database(String),
    Serialization(#[from] serde_json::Error),
    InvalidVlanId(u16),
    LibraryLoad(String),
    SaiError(i32),
}
```

### Recovery Strategy

1. **Transient Errors**: Retry with exponential backoff
2. **State Errors**: Log and continue (don't crash daemon)
3. **Fatal Errors**: Cleanup and exit gracefully
4. **SAI Errors**: Translate to RacoonError with context

## Concurrency Model

### Tokio Async Runtime

All daemons use Tokio for async I/O:

```rust
#[tokio::main]
async fn main() -> Result<()> {
    // Database operations are async
    let entry = db_client.get(Database::Appl, key).await?;

    // SAI calls are sync (hardware operations)
    let oid = vlan_api.create_vlan(switch_id, vlan_id)?;
}
```

### Thread Safety

- **DashMap**: Concurrent HashMap for state tracking
- **Arc**: Shared ownership across async tasks
- **Mutex/RwLock**: Fine-grained locking where needed

## Configuration

### Environment Variables

- `RACOON_DB_URL`: Database connection (default: `redis://127.0.0.1:6379`)
- `SAI_LIBRARY_PATH`: SAI library path (default: `/usr/lib/libsai.so`)
- `RUST_LOG`: Logging level (default: `info`)

### Database Schema

Following SONiC naming conventions:

```
CONFIG_DB:
  VLAN|Vlan{id}                    → VlanConfig
  VLAN_MEMBER|Vlan{id}|{port}     → VlanMemberConfig
  PORT|{name}                       → PortConfig
  LAG|PortChannel{id}              → LagConfig

APPL_DB:
  VLAN_TABLE:{name}                → VlanEntry
  VLAN_MEMBER_TABLE:{vlan}:{port} → VlanMemberEntry
  PORT_TABLE:{name}                → PortEntry
  FDB_TABLE:{vlan}:{mac}           → FdbEntry

ASIC_DB:
  ASIC_STATE:SAI_OBJECT_TYPE_VLAN:{oid}
  ASIC_STATE:SAI_OBJECT_TYPE_BRIDGE_PORT:{oid}
```

## Performance Considerations

### Database Optimization

- **Connection Pooling**: One connection per database number
- **Batching**: Batch related operations when possible
- **Selective Subscribe**: Only subscribe to relevant channels

### Memory Management

- **Bounded State**: Limit in-memory state size
- **Lazy Loading**: Load state on-demand
- **Periodic Cleanup**: Remove stale entries

### SAI Efficiency

- **Bulk Operations**: Use SAI bulk APIs where available
- **Attribute Caching**: Cache frequently-read attributes
- **Async SAI**: Future: async SAI wrapper for better concurrency

## Security

### Current State

- Database authentication via URL credentials
- No encryption (localhost communication)
- No authorization (trusted environment)

### Future Enhancements

- TLS for database connections
- SAI operation auditing
- Role-based access control (RBAC)
- Secure credential storage

## Monitoring and Observability

### Logging

Structured logging with `tracing`:

```rust
info!("Creating VLAN {} in hardware", vlan_id);
debug!("SAI OID: 0x{:x}", vlan_oid);
warn!("VLAN {} not found", vlan_id);
error!("Failed to program VLAN: {}", e);
```

### Metrics

Future: Export metrics to Prometheus:
- VLAN creation latency
- Database operation counts
- SAI call success/failure rates
- Memory usage per daemon

### Health Checks

Future: Health check endpoints:
- Database connectivity
- SAI library status
- Daemon uptime
- Last successful operation timestamp

## Testing Strategy

### Unit Tests

```rust
#[cfg(test)]
mod tests {
    #[test]
    fn test_vlan_id_validation() {
        assert!(VlanId::new(1).is_some());
        assert!(VlanId::new(0).is_none());
        assert!(VlanId::new(4095).is_none());
    }
}
```

### Integration Tests

See [examples/README.md](../examples/README.md) for end-to-end testing.

### Mock SAI

Future: Mock SAI implementation for testing without hardware.

## Future Architecture Enhancements

### Planned Features

1. **L3 Routing**: Route orchestration and synchronization
2. **ACLs**: Access control list management
3. **QoS**: Quality of service configuration
4. **Tunneling**: VXLAN, GRE support
5. **BGP Integration**: FRR routing daemon integration

### Scalability Improvements

1. **Multi-ASIC Support**: Handle multiple ASICs per switch
2. **Distributed Database**: Scale beyond single Redis instance
3. **Event Batching**: Reduce pub/sub overhead
4. **State Compression**: Optimize memory usage

### High Availability

1. **Daemon Supervision**: systemd integration
2. **Graceful Restart**: Preserve state across restarts
3. **Failover Support**: Active-standby configuration
4. **State Reconciliation**: Detect and fix state drift
