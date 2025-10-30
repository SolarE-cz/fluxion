# FluxION ECS - Home Assistant Add-on

Energy Control System for PV plant automation - Rust implementation.

## About

FluxION ECS is a Rust-based energy control system that optimizes your PV (photovoltaic) plant
operations through intelligent automation. It integrates seamlessly with Home Assistant to provide
real-time control and monitoring of your solar energy system.

## Features

- **Multi-inverter Support**: Works with Solax inverters (with support for Fronius and SMA planned)
- **Spot Price Integration**: Automatically adjusts energy usage based on current electricity prices
- **15-minute Time Block Scheduling**: Fine-grained control over energy management
- **Debug Mode**: Safe testing environment without affecting actual hardware
- **ECS Architecture**: Built on Bevy ECS for efficient, modular system design
- **Home Assistant Integration**: Native integration with Home Assistant ecosystem

## Installation

1. Add this repository to Home Assistant:

   - Navigate to: Settings → Add-ons → Add-on Store → ⋮ (menu) → Repositories
   - Add URL: `https://gitlab.com/SolarE-cz/fluxion`

2. Find "FluxION ECS" in the add-on store and click Install

3. Configure the add-on (see Configuration section below)

4. Start the add-on

5. Check the logs to verify everything is working correctly

## Configuration

The add-on is configured through Home Assistant's UI. Here's an example configuration:

```yaml
# FluxION ECS Configuration Example
# Energy Control System for PV plant automation
  #
  # Copy this file to config.toml and customize for your setup

  # Inverter Configuration
  # You can configure multiple inverters with master/slave topology
  [[inverters]]
  id = "main_inverter"
  vendor = "solax"
  entity_prefix = "solax"  # Prefix for Home Assistant entities
  topology = "independent" # Options: independent, master, slave

  # Example: Multi-inverter setup (commented out)
  # [[inverters]]
  # id = "master_inverter"
  # vendor = "solax"
  # entity_prefix = "solax_1"
  # topology = "master"
  # slaves = ["slave_1", "slave_2"]
  #
  # [[inverters]]
  # id = "slave_1"
  # vendor = "solax"
  # entity_prefix = "solax_2"
  # topology = "slave"
  # master = "master_inverter"

  # Pricing Configuration
  [pricing]
  spot_price_entity = "sensor.current_spot_electricity_price_15min" # Your HA spot price sensor
  # Optional: separate sensor for tomorrow's prices (if your integration splits today/tomorrow)
  tomorrow_price_entity = "sensor.tomorrow_spot_electricity_15min_order"
  use_spot_prices_to_buy = true
  use_spot_prices_to_sell = true

  # Fixed hourly prices (24 values) - fallback when spot prices disabled
  # These are in CZK/kWh (or your local currency)
  fixed_buy_prices = [
  0.05,
  0.05,
  0.05,
  0.05,
  0.05,
  0.05, # 00:00 - 05:59
  0.06,
  0.07,
  0.08,
  0.08,
  0.07,
  0.06, # 06:00 - 11:59
  0.06,
  0.07,
  0.08,
  0.08,
  0.09,
  0.10, # 12:00 - 17:59
  0.09,
  0.08,
  0.07,
  0.06,
  0.05,
  0.05, # 18:00 - 23:59
]
  fixed_sell_prices = [
  0.08,
  0.08,
  0.08,
  0.08,
  0.08,
  0.08, # 00:00 - 05:59
  0.09,
  0.10,
  0.11,
  0.11,
  0.10,
  0.09, # 06:00 - 11:59
  0.09,
  0.10,
  0.11,
  0.11,
  0.12,
  0.13, # 12:00 - 17:59
  0.12,
  0.11,
  0.10,
  0.09,
  0.08,
  0.08, # 18:00 - 23:59
]

  # Control Configuration
  [control]
  maximum_export_power_w = 5000 # Maximum grid export power in watts
  force_charge_hours = 4        # Number of cheapest hours to force-charge battery
  force_discharge_hours = 2     # Number of most expensive hours to force-discharge
  min_battery_soc = 10.0        # Minimum battery state of charge (%)
  max_battery_soc = 100.0       # Maximum battery state of charge (%)

  # Mode change debounce interval to prevent rapid switching
  # Default: 300 seconds (5 minutes), Minimum: 60 seconds (1 minute)
  min_mode_change_interval_secs = 300

  # Average household power consumption in kW (used for battery SOC predictions)
  # This is used as a fallback when actual load sensor data is not available
  # Typical values: 0.3-1.0 kW depending on household size and time of day
  # Default: 0.5 kW (500W)
  average_household_load_kw = 0.5

  # Minimum number of consecutive 15-minute blocks for force-charge/discharge operations
  # This protects inverter EEPROM from excessive writes due to frequent mode switching
  # Default: 2 blocks (30 minutes), Set to 1 to allow single blocks (not recommended)
  min_consecutive_force_blocks = 2

  # System Configuration
  [system]
  debug_mode = true         # Safe default - logs actions without making actual hardware changes
  update_interval_secs = 60 # How often to update (minimum 10 seconds)
  log_level = "info"        # Options: error, warn, info, debug, trace
  display_currency = "CZK"  # Display currency for web UI: EUR, USD, or CZK

  # Optional: Home Assistant connection
  # If not set, will use SUPERVISOR_TOKEN for HA addon, or fail for standalone
  # For development/testing outside HA addon, uncomment and set these:
  # ha_base_url = "http://homeassistant.local:8123"
  # ha_token = "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9..."  # Your long-lived access token
```

### Configuration Options

#### General Settings

- `debug_mode` (boolean, optional): Enable debug mode for testing without affecting hardware.
  Default: `false`
- `log_level` (string, optional): Logging verbosity. Options: `error`, `warn`, `info`, `debug`,
  `trace`. Default: `info`

#### Inverters

Configure your solar inverters:

- `type` (string, required): Inverter type. Currently supported: `solax`
- `host` (string, required): IP address or hostname of the inverter
- `serial_number` (string, required): Serial number of the inverter
- `register_prefix` (integer, optional): Modbus register prefix. Default: `0`

#### Pricing

Configure electricity pricing data:

- `provider` (string, required): Pricing data provider
- `api_url` (string, required): API endpoint for price data
- `update_interval` (integer, optional): How often to fetch prices (in seconds). Default: `900` (15
  minutes)

#### Control Settings

Fine-tune energy control parameters:

- `max_battery_soc` (integer, optional): Maximum battery state of charge (%). Default: `100`
- `min_battery_soc` (integer, optional): Minimum battery state of charge (%). Default: `10`
- `time_blocks` (integer, optional): Number of time blocks per day. Default: `96` (15-minute blocks)

## Support

For issues, feature requests, or contributions, please visit the
[GitHub repository](https://github.com/SolarE-cz/fluxion).

## License

Licensed under the Creative Commons Attribution-NonCommercial-NoDerivatives 4.0 International (CC
BY-NC-ND 4.0). You may use and share this file for non-commercial purposes only and you may not
create derivatives. See <https://creativecommons.org/licenses/by-nc-nd/4.0/>.
