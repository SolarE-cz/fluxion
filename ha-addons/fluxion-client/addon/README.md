# Home Assistant Add-on: FluxION ECS

FluxION ECS (Energy Control System) is a Home Assistant add-on for intelligent PV plant automation
based on electricity spot prices.

## About

This add-on provides automated control for solar inverters based on real-time electricity prices. It
optimizes battery charging and discharging schedules to maximize savings and revenue from your solar
installation.

### Features

- **Spot Price Integration**: Automatically fetch electricity spot prices from Home Assistant
  entities
- **Smart Scheduling**: Generate optimal charge/discharge schedules for 15-minute time blocks
- **Multi-Inverter Support**: Control multiple inverters with master/slave topology
- **Debug Mode**: Test and validate without making real changes to your system
- **Multiple Vendors**: Support for Solax, Fronius, and SMA inverters (extendable)

## Installation

1. Add this repository to your Home Assistant:

   - Navigate to **Settings** → **Add-ons** → **Add-on Store**
   - Click the menu (⋮) in the top right → **Repositories**
   - Add the URL of this repository

2. Install the **FluxION ECS** add-on

3. Configure the add-on (see Configuration section below)

4. Start the add-on

## Configuration

Example configuration:

```yaml
debug_mode: true
log_level: info
inverters:
  - id: "main_inverter"
    vendor: "solax"
    entity_prefix: "solax"
    topology: "independent"
    min_battery_soc: 10
    max_battery_soc: 100
pricing:
  spot_price_entity: "sensor.current_spot_electricity_prices"
  use_spot_prices_to_buy: true
  use_spot_prices_to_sell: true
  force_charge_hours: 4
  force_discharge_hours: 2
control:
  maximum_export_power_w: 10000
  update_interval_secs: 60
```

### Option: `debug_mode`

Enable debug mode to test the system without making real changes. When enabled, all actions are
logged but not executed.

### Option: `log_level`

Set the logging verbosity. Options: `trace`, `debug`, `info`, `warn`, `error`.

### Option: `inverters`

List of inverters to control. Each inverter requires:

- `id`: Unique identifier
- `vendor`: Inverter manufacturer (`solax`, `fronius`, `sma`)
- `entity_prefix`: Home Assistant entity prefix
- `topology`: `independent`, `master`, or `slave`
- `min_battery_soc`: Minimum battery state of charge (%)
- `max_battery_soc`: Maximum battery state of charge (%)

### Option: `pricing`

Price configuration:

- `spot_price_entity`: Home Assistant entity providing spot prices
- `use_spot_prices_to_buy`: Use spot prices for charging decisions
- `use_spot_prices_to_sell`: Use spot prices for discharging decisions
- `force_charge_hours`: Number of cheapest hours for charging
- `force_discharge_hours`: Number of most expensive hours for discharging

### Option: `control`

Control parameters:

- `maximum_export_power_w`: Maximum export power limit (watts)
- `update_interval_secs`: How often to check and update (seconds)

## Support

For issues and feature requests, please visit the
[GitHub repository](https://github.com/your-org/fluxion).

## License

CC BY-NC-ND 4.0 - see LICENSE file for details.
