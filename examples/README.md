# Racoon NOS Examples

This directory contains examples and test scripts for Racoon NOS.

## VLAN Creation Test

The `vlan_create_test.sh` script demonstrates the complete end-to-end VLAN creation flow.

### Prerequisites

1. **Valkey/Redis**: Running on `localhost:6379`
   ```bash
   # Install and start Valkey (Redis fork)
   docker run -d -p 6379:6379 valkey/valkey:latest
   # Or use Redis
   redis-server
   ```

2. **Build Racoon NOS**:
   ```bash
   cargo build --release
   ```

### Running the Test

The test can be run in two modes:

#### Mode 1: Manual Database Test (No Daemons)

This mode tests database operations without running the daemons:

```bash
./examples/vlan_create_test.sh
```

This will:
- Write a VLAN configuration to CONFIG_DB
- Check if the entry exists
- Provide instructions for running the full test

#### Mode 2: Full End-to-End Test (With Daemons)

This mode tests the complete flow with all daemons running:

1. **Terminal 1 - Start Valkey/Redis**:
   ```bash
   redis-server
   # or
   docker run -p 6379:6379 valkey/valkey:latest
   ```

2. **Terminal 2 - Start orchd**:
   ```bash
   cargo run --bin racoon-orchd
   # Or with release build:
   cargo run --release --bin racoon-orchd
   ```

3. **Terminal 3 - Start syncd**:
   ```bash
   # Note: syncd requires SAI library, will fail gracefully without it
   cargo run --bin racoon-syncd
   # Or with custom SAI library path:
   SAI_LIBRARY_PATH=/path/to/libsai.so cargo run --bin racoon-syncd
   ```

4. **Terminal 4 - Run the test**:
   ```bash
   ./examples/vlan_create_test.sh
   ```

### Expected Output

When running the full test with daemons:

```
=== Racoon NOS VLAN Creation Test ===

✓ Redis/Valkey is running

Test 1: Creating VLAN 100 in CONFIG_DB
✓ CONFIG_DB entry created:
  {"vlanid": 100, "description": "Test VLAN"}

Test 2: Checking APPL_DB (orchd output)
✓ APPL_DB entry found (orchd processed it):
  {"vlanid": 100, "description": "Test VLAN"}

Test 3: Checking ASIC_DB (syncd output)
✓ ASIC_DB entries found (syncd programmed hardware):
  ASIC_STATE:SAI_OBJECT_TYPE_VLAN:0x260000000004d2 = {"vlanid":100,"oid":"0x260000000004d2"}

=== Summary ===
Data flow: CONFIG_DB → orchd → APPL_DB → syncd → ASIC_DB → Hardware
```

### Data Flow

The test demonstrates the complete Racoon NOS architecture:

```
┌─────────────┐
│ CONFIG_DB   │  Database 4: User configuration
│  VLAN|Vlan100 │
└──────┬──────┘
       │
       ▼
┌─────────────┐
│   orchd     │  Orchestration daemon
│  (VlanOrch) │  Translates config to application entries
└──────┬──────┘
       │
       ▼
┌─────────────┐
│  APPL_DB    │  Database 0: Application state
│ VLAN_TABLE:*│
└──────┬──────┘
       │
       ▼
┌─────────────┐
│   syncd     │  Synchronization daemon
│ (VlanSync)  │  Programs hardware via SAI
└──────┬──────┘
       │
       ▼
┌─────────────┐
│  ASIC_DB    │  Database 1: Hardware state
│ ASIC_STATE:*│
└──────┬──────┘
       │
       ▼
┌─────────────┐
│  Hardware   │  Physical switch ASIC
│  (via SAI)  │
└─────────────┘
```

### Environment Variables

- `RACOON_DB_URL`: Database connection URL (default: `redis://127.0.0.1:6379`)
- `SAI_LIBRARY_PATH`: Path to SAI library for syncd (default: `/usr/lib/libsai.so`)

### Troubleshooting

**orchd not creating APPL_DB entries:**
- Check that orchd is running and connected to the database
- Look for errors in orchd output
- Verify CONFIG_DB entry format matches expected schema

**syncd not creating ASIC_DB entries:**
- Check that syncd is running
- Verify SAI library is loaded (syncd will warn if not available)
- Without a SAI library, syncd will fail to start
- For testing without hardware, you can use a mock SAI implementation

**Database connection issues:**
- Verify Valkey/Redis is running: `redis-cli ping`
- Check connection URL matches your setup
- Ensure no firewall blocking port 6379

### Cleaning Up

To remove test data:

```bash
redis-cli -n 4 DEL "VLAN|Vlan100"
redis-cli -n 0 DEL "VLAN_TABLE:Vlan100"
redis-cli -n 1 KEYS "ASIC_STATE:SAI_OBJECT_TYPE_VLAN:*" | xargs redis-cli -n 1 DEL
```

Or use the FLUSHALL command (⚠️ this removes ALL data):

```bash
redis-cli FLUSHALL
```
