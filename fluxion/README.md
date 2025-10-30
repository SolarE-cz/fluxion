# FluxION - Solar Plant Automation System

[![License: AGPL v3](https://img.shields.io/badge/License-AGPL%20v3-blue.svg)](https://www.gnu.org/licenses/agpl-3.0)
[![Rust](https://img.shields.io/badge/rust-1.75%2B-orange.svg)](https://www.rust-lang.org/)

A Rust-based solar plant automation system using Bevy ECS framework. Supports Home Assistant integration for vendor-agnostic inverter control and comprehensive sensor monitoring.

**Status:** ✅ MVP Complete - Production ready for Home Assistant integration

## Features

- **ECS Architecture**: Built on Bevy's Entity Component System for clean separation of data and
  logic
- **Vendor-Agnostic Design**: Generic sensor abstraction layer supporting multiple inverter brands
- **Home Assistant Integration**: Primary communication through HA API with existing vendor
  integrations
- **Internationalization (i18n)**: Multi-language support (Czech, English) with Mozilla Fluent
  framework
- **Comprehensive Monitoring**: 30+ optional sensor types including:
  - Extended PV strings (PV1-4 individual powers)
  - Three-phase measurements (L1/L2/L3 voltage, current, power)
  - Battery extended metrics (SOH, BMS limits, energy totals)
  - Grid totals (import/export, today/total yield)
  - Temperature sensors (inverter, battery, board, boost)
  - EPS status (Emergency Power Supply)
  - Fault/diagnostic data
- **Spot Price Automation**: Intelligent scheduling based on electricity spot prices
- **Channel-Based Async**: Non-blocking async operations using crossbeam channels
- **Optional Modbus Support**: Feature-gated direct Modbus TCP communication

## Internationalization

FluxION supports multiple languages with complete translations:

- 🇬🇧 **English** (default)
- 🇨🇿 **Czech** (Čeština)

Features:

- 134+ translation keys covering all UI elements
- Variable interpolation for dynamic values (prices, counts)
- Correct pluralization for each language
- Embedded translations in binary (no external files needed)

See [docs/guides/I18N.md](docs/guides/I18N.md) for detailed documentation on:

- Using translations in code and templates
- Adding new languages
- Translation file format and best practices

## System Requirements

- Rust 1.75+
- Tokio runtime for async operations
- Network access to Solax inverter (Modbus TCP port 502)

## Quick Start

### Prerequisites
- Rust 1.75 or later
- Home Assistant instance with inverter integration
- (Optional) Direct Modbus TCP access to inverter

### Installation

1. **Clone and build the project:**

```bash
git clone https://github.com/yourusername/fluxion.git
cd fluxion
cargo build --release
```

2. **Configure the system:**

Copy the example configuration and edit it:

```bash
cp config.example.toml config.toml
# Edit config.toml with your settings
```

See [docs/guides/CONFIGURATION.md](docs/guides/CONFIGURATION.md) for detailed configuration options.

3. **Run the application:**

```bash
cargo run --release
```

For deployment options (Docker, Home Assistant addon), see [docs/guides/DEPLOYMENT.md](docs/guides/DEPLOYMENT.md).

## Documentation

📚 **[Complete Documentation →](docs/README.md)**

### Quick Links
- **[Configuration Guide](docs/guides/CONFIGURATION.md)** - How to configure FluxION
- **[Deployment Guide](docs/guides/DEPLOYMENT.md)** - Docker, Home Assistant addon
- **[Architecture Overview](docs/architecture/ARCHITECTURE.md)** - System design and ECS architecture
- **[Testing Guide](docs/guides/TESTING.md)** - Running tests
- **[Internationalization](docs/guides/I18N.md)** - Adding translations

## Architecture Overview

FluxION uses the **Bevy ECS (Entity Component System)** framework for clean separation of data and logic.

### Vendor-Agnostic Design

```
Solax/Other Inverter → GenericInverterState → FluxION business logic
         ↓
   VendorEntityMapper
         ↓
  Home Assistant API / Modbus
```

- **Generic Components**: Vendor-agnostic data structures (30+ optional fields)
- **Entity Mappers**: Brand-specific HA entity name mappings
- **Data Sources**: Abstract interfaces for reading/writing inverter data
- **Adapters**: Concrete implementations (HomeAssistant, Modbus)

For detailed architecture information, see [docs/architecture/ARCHITECTURE.md](docs/architecture/ARCHITECTURE.md).

### Key Features

- **30+ Sensor Types**: Comprehensive monitoring including PV strings, three-phase power, battery metrics, temperatures
- **Economic Optimization**: Multiple strategy system for intelligent charge/discharge scheduling
- **Seasonal Adaptation**: Different optimization strategies for summer vs winter
- **Web UI**: Real-time monitoring dashboard with charts and controls
- **Prometheus Metrics**: Integration with monitoring tools
- **Channel-Based Async**: Non-blocking operations for responsive system

## Web Interface

FluxION includes a built-in web UI accessible at `http://localhost:3000` (configurable):

- Real-time system status and metrics
- Interactive charts showing power flows
- Schedule visualization
- Manual mode control
- Multi-language support (English, Czech)

## Supported Inverters

Currently tested with:
- **Solax X3-Hybrid G4** (via Home Assistant integration or Modbus)

The vendor-agnostic architecture makes it easy to add support for other brands. See [docs/architecture/ARCHITECTURE.md](docs/architecture/ARCHITECTURE.md) for implementation details.

## Troubleshooting

Common issues and solutions:

- **Connection Issues**: Verify Home Assistant API access and authentication
- **Missing Sensors**: Check that your inverter exposes the required entities in HA
- **Scheduling Problems**: Verify spot price entity is available and updating

For detailed troubleshooting, see [docs/guides/DEPLOYMENT.md](docs/guides/DEPLOYMENT.md).

## Project Status

**Current Version:** MVP Complete (v0.1.0)

**Status:**
- ✅ Core ECS architecture implemented
- ✅ Home Assistant integration working
- ✅ Economic optimization strategies (8 strategies)
- ✅ Web UI with real-time monitoring
- ✅ Multi-language support (EN, CZ)
- ✅ Comprehensive sensor support (30+ types)
- 🔄 Additional inverter brand support (planned)
- 🔄 Advanced forecasting integration (planned)

## Contributing

Contributions are welcome! Please:

1. Read the [Architecture Overview](docs/architecture/ARCHITECTURE.md)
2. Follow Rust best practices (`cargo fmt`, `cargo clippy`)
3. Maintain the ECS paradigm (separation of data and logic)
4. Add tests for new features
5. Update documentation as needed

## License

**GNU Affero General Public License v3.0 (AGPL-3.0)**

This program is free software: you can redistribute it and/or modify it under the terms of the GNU Affero General Public License as published by the Free Software Foundation, either version 3 of the License, or (at your option) any later version.

For commercial licensing inquiries, contact: info@solare.cz

See [LICENSE](LICENSE) for full license text.

## Acknowledgments

- Built with [Bevy ECS](https://bevyengine.org/)
- Web framework: [Axum](https://github.com/tokio-rs/axum)
- Internationalization: [Fluent](https://projectfluent.org/)
- Templating: [Askama](https://github.com/djc/askama)
