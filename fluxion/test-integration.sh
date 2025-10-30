#!/bin/bash
# Integration test runner script
# Usage: ./test-integration.sh [test_name]

set -e

# Colors for output
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
RED='\033[0;31m'
NC='\033[0m' # No Color

# Check for token file
if [ ! -f ".token.txt" ]; then
  echo -e "${RED}Error: .token.txt not found${NC}"
  echo "Please create .token.txt with your Home Assistant long-lived access token"
  echo "Get token from: Home Assistant → Profile → Long-Lived Access Tokens"
  exit 1
fi

echo -e "${GREEN}✓${NC} Found .token.txt"

# Get test name from argument or default to all tests
TEST_NAME=${1:-""}

if [ -z "$TEST_NAME" ]; then
  echo -e "${YELLOW}Running all integration tests...${NC}"
  cargo test --package fluxion-integration-tests --test ha_integration -- --ignored --nocapture
else
  echo -e "${YELLOW}Running test: $TEST_NAME${NC}"
  cargo test --package fluxion-integration-tests --test ha_integration "$TEST_NAME" -- --ignored --nocapture
fi
