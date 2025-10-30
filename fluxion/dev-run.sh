#!/usr/bin/env bash
# FluxION ECS Development Runner
# This script ensures the correct Rust version is used for development

set -e

cd "$(dirname "$0")"

echo "🦀 FluxION ECS - Development Mode"
echo "=================================="
echo ""

# Check if we have rust-toolchain.toml
if [ ! -f "rust-toolchain.toml" ]; then
  echo "❌ Error: rust-toolchain.toml not found"
  exit 1
fi

# Show current Rust version
echo "📋 Checking Rust version..."
RUSTC_VERSION=$(rustc --version 2>&1)
CARGO_VERSION=$(cargo --version 2>&1)

echo "   rustc:  $RUSTC_VERSION"
echo "   cargo:  $CARGO_VERSION"
echo ""

# Check if we have the right version (nightly)
if ! echo "$RUSTC_VERSION" | grep -q "nightly"; then
  echo "❌ Error: Not using nightly Rust!"
  echo ""
  echo "   The project requires Rust nightly for edition 2024 support."
  echo "   Current version: $RUSTC_VERSION"
  echo ""
  echo "   To fix this:"
  echo ""
  echo "   Option 1: Exit and re-enter nix develop shell"
  echo "     exit"
  echo "     cd $(dirname $(pwd))"
  echo "     nix develop"
  echo "     cd fluxion-ecs"
  echo ""
  echo "   Option 2: Use rustup (if not in nix shell)"
  echo "     rustup show  # This will install nightly automatically"
  echo ""
  exit 1
fi

# Check if config exists
if [ ! -f "config.toml" ]; then
  echo "⚠️  Warning: config.toml not found"
  if [ -f "config.example.toml" ]; then
    echo ""
    read -p "   Create config.toml from example? (y/n) " -n 1 -r
    echo ""
    if [[ $REPLY =~ ^[Yy]$ ]]; then
      cp config.example.toml config.toml
      echo "   ✅ Created config.toml from example"
      echo ""
      echo "   📝 Please edit config.toml with your settings:"
      echo "      - Set ha_base_url (e.g., http://homeassistant.local:8123)"
      echo "      - Set ha_token (your long-lived access token)"
      echo "      - Configure your inverter(s)"
      echo ""
      exit 1
    else
      echo "   Continuing without config.toml (will use defaults + env vars)"
    fi
  else
    echo "   ❌ Error: config.example.toml not found"
    exit 1
  fi
else
  echo "✅ Configuration file found: config.toml"

  # Check if HA token is configured
  if grep -q "^ha_token = " config.toml; then
    echo "✅ HA token configured in config.toml"
  elif [ -n "$HA_TOKEN" ]; then
    echo "✅ HA token found in environment variable"
  else
    echo "⚠️  Warning: No HA token configured"
    echo "   Either:"
    echo "   1. Add ha_token to config.toml [system] section"
    echo "   2. Set HA_TOKEN environment variable"
  fi
fi

echo ""
echo "🚀 Running FluxION ECS..."
echo "=================================="
echo ""

# Set RUST_LOG if not already set
if [ -z "$RUST_LOG" ]; then
  export RUST_LOG="info"
fi

exec cargo run "$@"
