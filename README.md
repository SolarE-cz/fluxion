# FluxION

PV plant automation and battery optimization for Home Assistant-powered systems. FluxION connects to
your inverter via Home Assistant, schedules charge/discharge using spot prices and solar forecasts,
and provides a lightweight web dashboard and data export for analysis.

- Web UI: http://localhost:8099/ (standalone) or via Home Assistant Ingress
- Home Assistant integration: Supervisor API and REST tokens supported
- Strategies: winter peak discharge, solar-aware charging, time-aware charge, price arbitrage, and
  optimizer
- Built with Bevy ECS, Axum, and Tokio (Rust edition 2024)

## Contents

- What it does
- Requirements
- Quick start (native)
- Quick start (Docker)
- Configuration
- Running and observing
- Home Assistant addon notes
- Analysis toolkit
- Development
- License

## What it does

FluxION continuously:

- Reads inverter telemetry through Home Assistant (Solax supported in MVP)
- Ingests spot electricity prices (e.g., `sensor.current_spot_electricity_price`)
- Computes 15-minute schedules and strategy actions
- Sends safe control commands with configurable debounce and SOC limits
- Serves a dashboard, live stream, and export endpoint for offline analysis

Key features:

- Multiple strategies that can cooperate:
  - Winter peak discharge: discharge before expensive evening peaks with solar-aware safety rails
  - Solar-aware charging: leave headroom to capture midday PV
  - Time-aware charge/discharge windows
  - Price arbitrage: buy low, use/sell high (includes wear and efficiency)
  - Optimizer components for future scenarios
- Safety and stability:
  - Hardware minimum SOC respected
  - Configurable min/max SOC and minimum interval between mode changes
  - Debug mode that makes no hardware changes
- Web UI and APIs:
  - Dashboard at `/`
  - Server-Sent Events at `/stream`
  - Chart data at `/chart-data`
  - Data export at `/export`
  - Health at `/health`
- Internationalization (i18n) with language selection in config

## Requirements

- Rust toolchain supporting edition 2024 (nightly recommended). See `fluxion/rust-toolchain.toml`.
- Home Assistant with:
  - Inverter entities (Solax supported now)
  - Spot price sensor entity (e.g., `sensor.current_spot_electricity_price`)
- OS: Linux/macOS/Container. x86_64 and ARM64 targets supported.

## Quick start (native)

1. Clone and enter the workspace

```bash
git clone https://github.com/SolarE-cz/fluxion.git
cd fluxion/fluxion
```

2. Create a configuration file `config.toml` in the working directory (see template below)

3. Run FluxION

```bash
# Release build recommended on low-power devices
cargo run -p fluxion-main --release --bin fluxion
```

- Web UI: http://localhost:8099/
- Logs: controlled by `RUST_LOG` (e.g., `RUST_LOG=info`)

## Quick start (Docker)

See `fluxion/docs/guides/NIX_DOCKER_BUILDS.md` for Docker build instructions using Nix, or use the
standard Dockerfile in `fluxion/Dockerfile`.

## Configuration

FluxION loads configuration in this order:

1. Home Assistant addon options JSON: `/data/options.json` (when running as an addon)
2. `config.toml` in the current working directory
3. `config.json` in the current working directory
4. Environment variables for selected keys
5. Built-in defaults (safe, debug mode enabled)

Minimal example `config.toml`:

```toml
# Inverters
[[inverters]]
id = "solax"
vendor = "solax"
entity_prefix = "solax"
topology = "independent"

[pricing]
spot_price_entity = "sensor.current_spot_electricity_price"
use_spot_prices_to_buy = true
use_spot_prices_to_sell = true
# Optional fixed prices as 24 hourly values (used when spot is disabled)
fixed_buy_prices  = [0.05, 0.05, 0.05, 0.05, 0.05, 0.05, 0.06, 0.07, 0.08, 0.08, 0.07, 0.06, 0.06, 0.07, 0.08, 0.08, 0.09, 0.10, 0.09, 0.08, 0.07, 0.06, 0.05, 0.05]
fixed_sell_prices = [0.08, 0.08, 0.08, 0.08, 0.08, 0.08, 0.09, 0.10, 0.11, 0.11, 0.10, 0.09, 0.09, 0.10, 0.11, 0.11, 0.12, 0.13, 0.12, 0.11, 0.10, 0.09, 0.08, 0.08]

[control]
maximum_export_power_w = 5000
force_charge_hours = 4
force_discharge_hours = 2
min_battery_soc = 15.0    # Strategy minimum – stop discharge above hardware limit
max_battery_soc = 100.0
hardware_min_battery_soc = 10.0  # Absolute inverter limit
average_household_load_kw = 0.5  # Used for SOC predictions if load is unavailable

# Strategy tuning examples
[strategies.winter_peak_discharge]
enabled = true
min_spread_czk = 3.0
min_soc_to_start = 70.0
min_soc_target = 50.0
solar_window_start_hour = 9
solar_window_end_hour = 15
min_hours_to_solar = 4

[strategies.solar_aware_charging]
enabled = true
solar_window_start_hour = 9
solar_window_end_hour = 12
midday_max_soc = 90.0
min_solar_forecast_kwh = 2.0

[strategies.seasonal]
force_season = "winter"   # or "summer"

[system]
debug_mode = true                # Safe: no hardware changes when true
update_interval_secs = 60
log_level = "info"
display_currency = "CZK"
# Home Assistant connection (not needed when running inside Supervisor)
# ha_base_url = "http://homeassistant.local:8123"
# ha_token = "<long-lived-access-token>"
# language = "en"            # en, cs, ...
```

Environment variable overrides (optional): `SPOT_PRICE_ENTITY`, `DEBUG_MODE`,
`UPDATE_INTERVAL_SECS`, `HA_BASE_URL`, `HA_TOKEN`.

## Running and observing

- Start: `cargo run -p fluxion-main --release --bin fluxion`
- Web UI: http://localhost:8099/
- Ingress (HA addon): `http://homeassistant.local:8123/api/hassio_ingress/<addon-slug>/`
- Health: `GET /health`
- Live stream: `GET /stream` (SSE)
- Export data: `GET /export` then analyze with the toolkit below

The app respects `RUST_LOG` for filtering logs, e.g. `RUST_LOG=info,fluxion_core=debug`.

## Home Assistant addon notes

- When running as an addon, FluxION reads `/data/options.json` for its configuration and uses the
  Supervisor token automatically when available.
- Web UI is exposed through HA Ingress and a sidebar panel when installed as an addon.
- Addon resources live under `ha-addons/` and `fluxion/addon/` in this repository.

## Analysis toolkit

FluxION includes a Python-based analysis and optimization toolkit to evaluate exports and tune
parameters.

- Quick start: `fluxion/analysis/QUICK_START.md`
- Typical flow:
  1. Click the "Export Data" button in the web UI header or `GET /export`
  2. Save JSON to `fluxion/data/`
  3. Run:
     ```bash
     cd fluxion
     python3 analysis/analyze_export.py data/your_export.json
     ```
  4. Review recommendations and iterate on config

Advanced multi-day analysis and Claude prompt are documented in the same folder.

## Development

Workspace layout (`fluxion/` is the Rust workspace root):

- `crates/fluxion-main` – binary (`fluxion`) that wires everything together
- `crates/fluxion-core` – ECS systems, scheduling, strategies, resources
- `crates/fluxion-web` – Axum web server, templates, SSE, export
- `crates/fluxion-ha` – Home Assistant client and adapters
- `crates/fluxion-solax` – Solax vendor mapping
- `crates/fluxion-i18n` – i18n bundles and helpers
- `crates/fluxion-integration-tests` – black-box tests

Useful commands:

```bash
# Check and build
cargo check --workspace
cargo build --workspace --release

# Run main binary
cargo run -p fluxion-main --bin fluxion

# Run specific tests
cargo test -p fluxion-core
```

Rust toolchain details are specified in `fluxion/rust-toolchain.toml`. Nix integration is described
in `flake.nix` and `fluxion/flake.nix`.

## License

- The repository currently includes a Creative Commons Attribution-NonCommercial-NoDerivatives 4.0
  International license file at `fluxion/LICENSE` and license metadata in the workspace
  `Cargo.toml`.
- Some source files also carry an AGPLv3+ header with a commercial licensing contact. For commercial
  licensing, contact: info@solare.cz.

Until clarified in a future release, treat the project as non-commercial with no-derivatives per the
CC BY-NC-ND 4.0 file. If you have questions about permitted use, please reach out.
