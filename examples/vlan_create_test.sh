#!/bin/bash
# Test script for VLAN creation end-to-end flow
#
# Prerequisites:
# - Valkey/Redis running on localhost:6379
# - racoon-orchd and racoon-syncd built
#
# This script demonstrates the complete VLAN creation flow:
# 1. Write VLAN config to CONFIG_DB
# 2. orchd translates it to APPL_DB
# 3. syncd programs it to hardware via SAI

set -e

# Colors for output
GREEN='\033[0;32m'
BLUE='\033[0;34m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

REDIS_CLI=${REDIS_CLI:-redis-cli}

echo -e "${BLUE}=== Racoon NOS VLAN Creation Test ===${NC}\n"

# Check if Redis is running
if ! $REDIS_CLI ping > /dev/null 2>&1; then
    echo -e "${YELLOW}Error: Redis/Valkey is not running on localhost:6379${NC}"
    echo "Please start Redis/Valkey before running this test"
    exit 1
fi

echo -e "${GREEN}✓ Redis/Valkey is running${NC}\n"

# Clean up any existing VLAN data
echo -e "${BLUE}Cleaning up existing VLAN data...${NC}"
$REDIS_CLI -n 4 DEL "VLAN|Vlan100" > /dev/null 2>&1 || true
$REDIS_CLI -n 0 DEL "VLAN_TABLE:Vlan100" > /dev/null 2>&1 || true
$REDIS_CLI -n 1 KEYS "ASIC_STATE:SAI_OBJECT_TYPE_VLAN:*" | xargs -r $REDIS_CLI -n 1 DEL > /dev/null 2>&1 || true

# Test 1: Create VLAN in CONFIG_DB
echo -e "${BLUE}Test 1: Creating VLAN 100 in CONFIG_DB${NC}"
$REDIS_CLI -n 4 SET "VLAN|Vlan100" '{"vlanid": 100, "description": "Test VLAN"}' > /dev/null

# Verify CONFIG_DB entry
CONFIG_ENTRY=$($REDIS_CLI -n 4 GET "VLAN|Vlan100")
echo -e "${GREEN}✓ CONFIG_DB entry created:${NC}"
echo "  $CONFIG_ENTRY"
echo

# Wait a moment for orchd to process (if running)
sleep 1

# Check if APPL_DB entry was created by orchd
echo -e "${BLUE}Test 2: Checking APPL_DB (orchd output)${NC}"
if $REDIS_CLI -n 0 EXISTS "VLAN_TABLE:Vlan100" > /dev/null; then
    APPL_ENTRY=$($REDIS_CLI -n 0 GET "VLAN_TABLE:Vlan100")
    echo -e "${GREEN}✓ APPL_DB entry found (orchd processed it):${NC}"
    echo "  $APPL_ENTRY"
else
    echo -e "${YELLOW}⚠ APPL_DB entry not found${NC}"
    echo "  orchd may not be running or hasn't processed the update yet"
    echo "  To manually test: Start orchd with 'cargo run --bin racoon-orchd'"
fi
echo

# Wait for syncd to process
sleep 1

# Check if ASIC_DB entry was created by syncd
echo -e "${BLUE}Test 3: Checking ASIC_DB (syncd output)${NC}"
ASIC_KEYS=$($REDIS_CLI -n 1 KEYS "ASIC_STATE:SAI_OBJECT_TYPE_VLAN:*")
if [ -n "$ASIC_KEYS" ]; then
    echo -e "${GREEN}✓ ASIC_DB entries found (syncd programmed hardware):${NC}"
    for key in $ASIC_KEYS; do
        VALUE=$($REDIS_CLI -n 1 GET "$key")
        echo "  $key = $VALUE"
    done
else
    echo -e "${YELLOW}⚠ ASIC_DB entry not found${NC}"
    echo "  syncd may not be running or SAI library may not be loaded"
    echo "  To manually test: Start syncd with 'cargo run --bin racoon-syncd'"
fi
echo

# Summary
echo -e "${BLUE}=== Summary ===${NC}"
echo "Data flow: CONFIG_DB → orchd → APPL_DB → syncd → ASIC_DB → Hardware"
echo
echo "To run the full end-to-end test:"
echo "  1. Terminal 1: Start Valkey/Redis"
echo "  2. Terminal 2: cargo run --bin racoon-orchd"
echo "  3. Terminal 3: cargo run --bin racoon-syncd"
echo "  4. Terminal 4: Run this script again"
echo
echo -e "${GREEN}Test completed!${NC}"
