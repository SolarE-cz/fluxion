# FluxION

**Battery automation and optimization for Home Assistant-powered solar systems**

FluxION connects to your inverter via Home Assistant, schedules charge/discharge using spot prices
and solar forecasts, and provides a web dashboard and data export for analysis.

[![License: CC BY-NC-ND 4.0](https://img.shields.io/badge/License-CC%20BY--NC--ND%204.0-lightgrey.svg)](https://creativecommons.org/licenses/by-nc-nd/4.0/)
[![Rust](https://img.shields.io/badge/rust-2024%20edition-orange.svg)](https://www.rust-lang.org/)

[English](#english) | [Čeština](#%C4%8De%C5%A1tina)

______________________________________________________________________

## English

### What it does

FluxION continuously:

- Reads inverter telemetry through Home Assistant (Solax supported, more planned)
- Ingests spot electricity prices from your HA sensor
- Computes 15-minute schedules with multiple optimization strategies
- Sends safe control commands with configurable limits and debounce
- Serves a web dashboard, live stream, and export endpoint

**Key features:**

- **Multiple strategies** - Winter peak discharge, solar-aware charging, time-aware windows, price
  arbitrage, seasonal adaptation
- **Safety** - Hardware minimum SOC respected, configurable limits, debug mode, mode change debounce
- **Web UI** - Real-time dashboard with charts, controls, and multi-language support (English,
  Czech)
- **Data export** - JSON export for analysis with included Python toolkit
- **Home Assistant integration** - Supervisor API and REST tokens supported, addon available

### Requirements

- **Rust** - Edition 2024 toolchain (see `fluxion/rust-toolchain.toml`)
- **Home Assistant** with:
  - Inverter integration (Solax supported, others planned)
  - Spot price sensor entity
- **OS** - Linux, macOS, or container (x86_64 and ARM64 supported)

### Quick Start

#### Native

```bash
# Clone repository
git clone https://github.com/SolarE-cz/fluxion.git
cd fluxion/fluxion

# Copy and edit configuration
cp config.example.toml config.toml
# Edit config.toml with your settings

# Run
cargo run -p fluxion-main --release --bin fluxion
```

Web UI: http://localhost:8099/

#### Docker

See `fluxion/docs/guides/NIX_DOCKER_BUILDS.md` for Docker builds using Nix, or use the standard
`fluxion/Dockerfile`.

#### Home Assistant Addon

See `ha-addons/fluxion-client/` for addon installation. The addon uses Ingress and provides a
sidebar panel.

### Configuration

**Minimal `config.toml`:**

```toml
[[inverters]]
id = "solax"
vendor = "solax"
entity_prefix = "solax"
topology = "independent"

[pricing]
spot_price_entity = "sensor.current_spot_electricity_price"
use_spot_prices_to_buy = true
use_spot_prices_to_sell = true
fixed_buy_prices = [0.05; 24]
fixed_sell_prices = [0.08; 24]

[control]
maximum_export_power_w = 5000
force_charge_hours = 4
force_discharge_hours = 2
min_battery_soc = 15.0
max_battery_soc = 100.0

[system]
debug_mode = true           # Safe default - no hardware changes
update_interval_secs = 60
log_level = "info"
display_currency = "CZK"
language = "en"
```

**📚 For complete configuration reference, see:**

- **English**: [`fluxion/docs/CONFIG_README.md`](fluxion/docs/CONFIG_README.md)
- **Czech**: [`fluxion/docs/CONFIG_README.cs.md`](fluxion/docs/CONFIG_README.cs.md)

Configuration loading priority:

1. `/data/options.json` (HA addon)
2. `config.toml`
3. `config.json`
4. Environment variables
5. Built-in defaults

### Documentation

**📚 [Complete Documentation](fluxion/docs/README.md)**

Quick links:

- **[Configuration Guide](fluxion/docs/CONFIG_README.md)** - Detailed configuration reference
- **[Deployment Guide](fluxion/docs/guides/DEPLOYMENT.md)** - Docker, HA addon
- **[Architecture](fluxion/docs/architecture/ARCHITECTURE.md)** - System design
- **[Testing](fluxion/docs/guides/TESTING.md)** - Running tests
- **[i18n](fluxion/docs/guides/I18N.md)** - Adding translations
- **[Web UI](fluxion/docs/guides/WEB_UI_GUIDE.md)** - Using the dashboard

### Analysis Toolkit

FluxION includes Python tools for analyzing exports and tuning parameters:

1. Export data via Web UI or `GET /export`
2. Save JSON to `fluxion/data/`
3. Run analysis:
   ```bash
   cd fluxion
   python3 analysis/analyze_export.py data/your_export.json
   ```

See `fluxion/analysis/QUICK_START.md` for details.

### Development

**Workspace structure** (`fluxion/` is the Rust workspace root):

```
fluxion/
├── crates/
│   ├── fluxion-main/            # Binary application
│   ├── fluxion-core/            # ECS systems, strategies, scheduling
│   ├── fluxion-web/             # Axum web server, templates, SSE
│   ├── fluxion-ha/              # Home Assistant client
│   ├── fluxion-solax/           # Solax vendor mapping
│   ├── fluxion-i18n/            # Internationalization
│   └── fluxion-integration-tests/
├── docs/                        # Documentation
├── config.example.toml          # Example configuration
└── Cargo.toml                   # Workspace manifest
```

**Useful commands:**

```bash
cargo check --workspace
cargo build --workspace --release
cargo run -p fluxion-main --bin fluxion
cargo test -p fluxion-core
```

See `fluxion/rust-toolchain.toml` for toolchain details.

### Project Status

**Version:** 0.1.0 (MVP Complete)

- ✅ Core ECS architecture
- ✅ Home Assistant integration
- ✅ Economic optimization strategies (8 strategies)
- ✅ Web UI with real-time monitoring
- ✅ Multi-language support (EN, CZ)
- ✅ Comprehensive sensor support (30+ types)
- 🔄 Additional inverter brands (planned)
- 🔄 Advanced forecasting integration (planned)

### License

**Creative Commons Attribution-NonCommercial-NoDerivatives 4.0 International (CC BY-NC-ND 4.0)**

Some source files carry an AGPLv3+ header. For commercial licensing inquiries: **info@solare.cz**

See [`fluxion/LICENSE`](fluxion/LICENSE) for full license text.

### Acknowledgments

Built with:

- [Bevy ECS](https://bevyengine.org/) - Entity Component System
- [Axum](https://github.com/tokio-rs/axum) - Web framework
- [Fluent](https://projectfluent.org/) - Internationalization
- [Askama](https://github.com/djc/askama) - Templating

______________________________________________________________________

## Čeština

### Co to dělá

FluxION průběžně:

- Čte telemetrii střídače přes Home Assistant (podporován Solax, další plánováno)
- Získává spotové ceny elektřiny z vašeho HA senzoru
- Počítá 15minutové rozvrhy s více optimalizačními strategiemi
- Posílá bezpečné řídicí příkazy s konfigurovatelnými limity a "debounce"
- Poskytuje webový dashboard, živý stream a export dat

**Klíčové funkce:**

- **Více strategií** - Zimní vybíjení ve špičce, nabíjení s ohledem na slunce, časová okna, cenová
  arbitráž, sezónní adaptace
- **Bezpečnost** - Respektuje bateriové minimální SOC, konfigurovatelné limity, debug režim
- **Webové UI** - Real-time dashboard s grafy, ovládáním a vícejazyčnou podporou (angličtina,
  čeština)
- **Export dat** - JSON export pro analýzu s dodanými Python nástroji
- **Integrace Home Assistant** - Podporuje Supervisor API i REST tokeny, dostupné jako addon

### Požadavky

- **Rust** - Toolchain edice 2024 (viz `fluxion/rust-toolchain.toml`)
- **Home Assistant** s:
  - Integrací střídače (podporován Solax, další plánováno)
  - Entitou senzoru spotové ceny
- **OS** - Linux, macOS nebo kontejner (podporovány x86_64 a ARM64)

### Rychlý start

#### Nativní

```bash
# Klonovat repozitář
git clone https://github.com/SolarE-cz/fluxion.git
cd fluxion/fluxion

# Zkopírovat a upravit konfiguraci
cp config.example.toml config.toml
# Upravte config.toml podle svého nastavení

# Spustit
cargo run -p fluxion-main --release --bin fluxion
```

Webové UI: http://localhost:8099/

#### Docker

Viz `fluxion/docs/guides/NIX_DOCKER_BUILDS.md` pro Docker buildy pomocí Nix, nebo použijte
standardní `fluxion/Dockerfile`.

#### Home Assistant Addon

Viz `ha-addons/fluxion-client/` pro instalaci addonu. Addon používá Ingress a poskytuje panel v
postranní liště.

### Konfigurace

**Minimální `config.toml`:**

```toml
[[inverters]]
id = "solax"
vendor = "solax"
entity_prefix = "solax"
topology = "independent"

[pricing]
spot_price_entity = "sensor.current_spot_electricity_price"
use_spot_prices_to_buy = true
use_spot_prices_to_sell = true
fixed_buy_prices = [1.5; 24]
fixed_sell_prices = [2.0; 24]

[control]
maximum_export_power_w = 5000
force_charge_hours = 4
force_discharge_hours = 2
min_battery_soc = 15.0
max_battery_soc = 100.0

[system]
debug_mode = true           # Bezpečný výchozí - žádné změny v hardware
update_interval_secs = 60
log_level = "info"
display_currency = "CZK"
language = "cs"
```

**📚 Pro kompletní referenci konfigurace viz:**

- **Anglicky**: [`fluxion/docs/CONFIG_README.md`](fluxion/docs/CONFIG_README.md)
- **Česky**: [`fluxion/docs/CONFIG_README.cs.md`](fluxion/docs/CONFIG_README.cs.md)

Priorita načítání konfigurace:

1. `/data/options.json` (HA addon)
2. `config.toml`
3. `config.json`
4. Proměnné prostředí
5. Vestavěné výchozí hodnoty

### Dokumentace

**📚 [Kompletní dokumentace](fluxion/docs/README.md)**

Rychlé odkazy:

- **[Průvodce konfigurací](fluxion/docs/CONFIG_README.cs.md)** - Detailní reference konfigurace
- **[Průvodce nasazením](fluxion/docs/guides/DEPLOYMENT.md)** - Docker, HA addon
- **[Architektura](fluxion/docs/architecture/ARCHITECTURE.md)** - Návrh systému
- **[Testování](fluxion/docs/guides/TESTING.md)** - Spouštění testů
- **[i18n](fluxion/docs/guides/I18N.md)** - Přidávání překladů
- **[Webové UI](fluxion/docs/guides/WEB_UI_GUIDE.md)** - Používání dashboardu

### Analytické nástroje

FluxION obsahuje Python nástroje pro analýzu exportů a ladění parametrů:

1. Exportujte data přes Webové UI nebo `GET /export`
2. Uložte JSON do `fluxion/data/`
3. Spusťte analýzu:
   ```bash
   cd fluxion
   python3 analysis/analyze_export.py data/vas_export.json
   ```

Viz `fluxion/analysis/QUICK_START.md` pro detaily.

### Vývoj

**Struktura workspace** (`fluxion/` je root Rust workspace):

```
fluxion/
├── crates/
│   ├── fluxion-main/            # Binární aplikace
│   ├── fluxion-core/            # ECS systémy, strategie, plánování
│   ├── fluxion-web/             # Axum web server, šablony, SSE
│   ├── fluxion-ha/              # Home Assistant klient
│   ├── fluxion-solax/           # Solax mapování střídače
│   ├── fluxion-i18n/            # Překlady
│   └── fluxion-integration-tests/
├── docs/                        # Dokumentace
├── config.example.toml          # Ukázková konfigurace
└── Cargo.toml                   # Workspace manifest
```

**Užitečné příkazy:**

```bash
cargo check --workspace
cargo build --workspace --release
cargo run -p fluxion-main --bin fluxion
cargo test -p fluxion-core
```

Viz `fluxion/rust-toolchain.toml` pro detaily toolchainu.

### Stav projektu

**Verze:** 0.1.0 (MVP dokončeno)

- ✅ Základní ECS architektura
- ✅ Integrace s Home Assistant
- ✅ Ekonomické optimalizační strategie (8 strategií)
- ✅ Webové UI s real-time monitorováním
- ✅ Podpora více jazyků (EN, CZ)
- ✅ Komplexní podpora senzorů (30+ typů)
- 🔄 Další značky střídačů (plánováno)
- 🔄 Pokročilá integrace předpovědí (plánováno)

### Licence

**Creative Commons Attribution-NonCommercial-NoDerivatives 4.0 International (CC BY-NC-ND 4.0)**

Některé zdrojové soubory obsahují AGPLv3+ hlavičku. Pro komerční licencování kontaktujte:
**info@solare.cz**

Viz [`fluxion/LICENSE`](fluxion/LICENSE) pro úplný text licence.

### Poděkování

Postaveno s:

- [Bevy ECS](https://bevyengine.org/) - Entity Component System
- [Axum](https://github.com/tokio-rs/axum) - Web framework
- [Fluent](https://projectfluent.org/) - Překlady
- [Askama](https://github.com/djc/askama) - Šablonování

______________________________________________________________________

**Last Updated / Poslední aktualizace**: 2025-10-31
