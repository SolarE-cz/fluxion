# FluxION Strategy Benchmarking Suite

Comprehensive testing framework for analyzing and comparing FluxION battery optimization strategies using real historical electricity prices and consumption data.

## Overview

This crate provides tools to:

- **Fetch historical price data** from OTE-CR (Czech electricity market)
- **Load real consumption data** from Excel/CSV files
- **Simulate battery operation** with different strategies
- **Analyze performance** across financial, battery health, and efficiency metrics
- **Generate detailed reports** in Markdown, JSON, and terminal-friendly formats
- **Run regression tests** to ensure strategies don't degrade over time

## Quick Start

### Running Comprehensive Benchmarks

```bash
# Run all strategy benchmarks (requires historical data)
cargo test --package fluxion-strategy-bench --test strategy_benchmark -- --ignored --test-threads=1

# With manual CSV price data
PRICE_DATA_CSV=/path/to/prices.csv cargo test --package fluxion-strategy-bench --test strategy_benchmark -- --ignored --test-threads=1
```

### Running Regression Tests

```bash
# Run regression tests with synthetic data (fast, no external data needed)
cargo test --package fluxion-strategy-bench --test regression
```

### Running Specific Scenarios

```bash
# Test only price arbitrage strategy
cargo test --package fluxion-strategy-bench --test strategy_benchmark benchmark_price_arbitrage_only -- --ignored

# Test predefined scenarios
cargo test --package fluxion-strategy-bench --test strategy_benchmark benchmark_predefined_scenarios -- --ignored
```

## Features

### 1. Data Sources

#### Price Data Fetching

- **OTE-CR Integration**: Fetches historical day-ahead market prices
- **Automatic Retry Logic**: Exponential backoff for network reliability
- **Caching**: Temporary caching during test runs to avoid repeated requests
- **Manual CSV Support**: Load from manually exported CSV files

```rust
use fluxion_strategy_bench::data_sources::PriceDataCache;

let mut cache = PriceDataCache::new()?;

// Option 1: Automatic fetching
let prices = cache.get_price_data(start_date, end_date)?;

// Option 2: Manual CSV
cache.set_manual_csv("/path/to/prices.csv".into());
let prices = cache.get_price_data(start_date, end_date)?;
```

#### Consumption Data

- **Excel Support**: Load from `.xlsx` files with timestamp + consumption columns
- **CSV Support**: Standard CSV format
- **Synthetic Profiles**: Generate realistic consumption patterns
  - `TypicalHousehold`: 4-person household, 0.5 kW average
  - `SmallHousehold`: 1-2 people, 0.3 kW average
  - `LargeHousehold`: 5+ people, 0.8 kW average

```rust
use fluxion_strategy_bench::data_sources::{load_from_excel, ConsumptionProfile};

// Load from Excel
let consumption = load_from_excel("consumption.xlsx")?;

// Or generate synthetic
let consumption = ConsumptionProfile::TypicalHousehold.generate(start, end);
```

### 2. Strategy Simulation

Simulates real battery operation using FluxION's actual strategy implementations:

```rust
use fluxion_strategy_bench::{BenchmarkConfig, Simulator};

let config = BenchmarkConfig {
    battery_capacity_kwh: 20.0,
    battery_efficiency: 0.95,
    battery_wear_cost_czk_per_kwh: 0.125,
    // ... other config
    ..Default::default()
};

let mut simulator = Simulator::new(config);
simulator.set_price_data(price_data);
simulator.set_consumption_data(consumption_data);

// Run economic optimizer (all strategies)
let result = simulator.run_economic_optimizer()?;

// Or run individual strategy
let result = simulator.run_single_strategy("Price-Arbitrage")?;

// Or compare all strategies
let results = simulator.run_all_strategies_individually()?;
```

### 3. Comprehensive Metrics

#### Financial Metrics
- Net profit (CZK)
- Daily average profit
- Grid import/export costs and revenues
- Battery degradation costs
- ROI and payback period calculations

#### Battery Health Metrics
- Total charge/discharge cycles
- Average depth of discharge
- SOC distribution analysis
- Estimated lifespan impact
- Health warnings

#### Efficiency Metrics
- Self-consumption rate (%)
- Autarky rate (%)
- Grid independence score
- Solar utilization efficiency
- Battery utilization rate

#### Strategy-Specific Metrics
- Win rate (% of blocks each strategy was chosen)
- Profit contribution per strategy
- Average profit per block
- Success rate

### 4. Report Generation

Generate beautiful, informative reports:

```rust
use fluxion_strategy_bench::reports::{BenchmarkReport, ReportFormat};

let report = BenchmarkReport::new(results, "December 2024 Benchmark");

// Print to console
report.print_summary();

// Export to files
report.export_to_file("report.md", ReportFormat::Markdown)?;
report.export_to_file("report.json", ReportFormat::Json)?;
```

**Example Console Output:**

```
================================================================================
        FluxION Strategy Benchmark - December 2024
            Generated: 2024-12-15 10:30:00 UTC
================================================================================

EXECUTIVE SUMMARY
────────────────────────────────────────────────────────────────────────────────

  Baseline Profit:       1,650.00 CZK
  Optimized Profit:      2,650.00 CZK
  Improvement:              60.6%
  Duration:                 31.0 days

STRATEGY PERFORMANCE RANKINGS
────────────────────────────────────────────────────────────────────────────────

┌──────┬───────────────────────┬─────────────┬────────┬─────────────┬───────┐
│ Rank │ Strategy              │ Profit (CZK)│ Cycles │ Self-Use %  │ Grade │
├──────┼───────────────────────┼─────────────┼────────┼─────────────┼───────┤
│ 1    │ Economic-Optimizer    │ 2,650.00    │ 45.2   │ 82.3        │ A     │
│ 2    │ Price-Arbitrage       │ 2,450.00    │ 48.1   │ 72.0        │ B     │
│ 3    │ Time-Aware-Charge     │ 2,380.00    │ 38.7   │ 78.5        │ B     │
│ 4    │ Solar-First           │ 2,120.00    │ 42.1   │ 85.2        │ A     │
│ 5    │ Self-Use              │ 1,650.00    │ 28.3   │ 65.1        │ C     │
└──────┴───────────────────────┴─────────────┴────────┴─────────────┴───────┘

KEY INSIGHTS
────────────────────────────────────────────────────────────────────────────────

  ✓ Best performer: Economic-Optimizer (2,650.00 CZK)
  ✓ Healthy battery usage: 45.2 cycles
  ✓ Excellent self-consumption: 82.3%
  ✓ High grid independence: 78.5% autarky
```

### 5. Predefined Scenarios

Test common use cases:

```rust
use fluxion_strategy_bench::scenarios::predefined;

// Winter with high prices and low solar
let scenario = predefined::winter_high_price();

// Summer with abundant solar
let scenario = predefined::summer_high_solar();

// Extreme price volatility
let scenario = predefined::price_spike();

// Small system (10kWh battery, 5kW solar)
let scenario = predefined::small_system();

// Large system (30kWh battery, 15kW solar)
let scenario = predefined::large_system();

// Run scenario
let simulator = scenario.run(&mut price_cache)?;
let result = simulator.run_economic_optimizer()?;
```

### 6. Custom Scenarios

Build your own test scenarios:

```rust
use fluxion_strategy_bench::scenarios::ScenarioBuilder;

let scenario = ScenarioBuilder::new("Custom Test")
    .description("Testing specific conditions")
    .date_range(start_date, end_date)
    .battery_capacity(25.0)  // 25 kWh
    .solar_capacity(12.0)    // 12 kW peak
    .consumption_profile("LargeHousehold")
    .expect("Price arbitrage should activate frequently")
    .build();

let simulator = scenario.run(&mut price_cache)?;
```

## Price Data Format

### OTE-CR Automatic Fetching

The system automatically fetches from:
```
https://www.ote-cr.cz/en/short-term-markets/electricity/day-ahead-market?time_resolution=PT15M&date=YYYY-MM-DD
```

### Manual CSV Format

If OTE-CR blocks automated access, export manually and use this CSV format:

```csv
Timestamp,Price
2024-12-01T00:00:00Z,0.45
2024-12-01T00:15:00Z,0.43
2024-12-01T00:30:00Z,0.42
...
```

Supported timestamp formats:
- ISO 8601: `2024-12-01T00:00:00Z`
- Standard: `2024-12-01 00:00:00`
- European: `01.12.2024 00:00`

Then use:
```bash
PRICE_DATA_CSV=/path/to/prices.csv cargo test ...
```

## Consumption Data Format

### Excel Format

Two columns:
- Column A: Timestamp
- Column B: Consumption (kW)

```
Timestamp             | Consumption
2024-12-01 00:00:00  | 0.45
2024-12-01 00:15:00  | 0.43
...
```

### CSV Format

```csv
Timestamp,Consumption
2024-12-01 00:00:00,0.45
2024-12-01 00:15:00,0.43
...
```

## Understanding Results

### Financial Performance

- **Net Profit**: Total revenue minus all costs (grid import, battery wear)
- **Daily Average**: Profit per day (useful for comparing different time periods)
- **ROI**: Return on investment (requires specifying battery cost)
- **Payback Period**: Years to recoup battery investment at current profit rate

### Battery Health

- **Cycles**: Full charge/discharge cycles (charge_kwh / battery_capacity_kwh)
  - <30 cycles/month: Conservative (long lifespan)
  - 30-50 cycles/month: Moderate (good balance)
  - >50 cycles/month: Aggressive (monitor degradation)

- **DoD (Depth of Discharge)**: Average SOC change per cycle
  - <50%: Very conservative
  - 50-80%: Optimal
  - >80%: May accelerate aging

- **SOC Range**: Minimum and maximum SOC reached
  - Should respect configured limits (default: 15%-100%)

### Efficiency

- **Self-Consumption Rate**: % of solar energy used directly (not exported)
  - >80%: Excellent
  - 60-80%: Good
  - <60%: Room for improvement

- **Autarky Rate**: % of consumption covered by solar (not from grid)
  - >80%: Highly independent
  - 60-80%: Good independence
  - <60%: Grid-dependent

- **Grid Independence Score**: Average of self-consumption and autarky
  - 90-100: A grade
  - 80-90: B grade
  - 70-80: C grade

## Regression Testing

Regression tests use synthetic data to ensure consistent performance:

```bash
cargo test --package fluxion-strategy-bench --test regression
```

Tests verify:
- ✅ Economic optimizer generates positive profit
- ✅ Strategies respect battery SOC limits
- ✅ Battery cycles stay within healthy ranges
- ✅ Financial metrics are internally consistent
- ✅ Optimizer outperforms baseline (self-use only)

## Architecture

```
fluxion-strategy-bench/
├── src/
│   ├── data_sources/       # Price & consumption data loading
│   │   ├── price_fetcher.rs   # OTE-CR integration
│   │   ├── consumption.rs     # Excel/CSV loading & profiles
│   │   └── cache.rs           # Temporary caching
│   ├── simulator/          # Strategy simulation engine
│   │   └── runner.rs          # Main simulation logic
│   ├── metrics/            # Performance analysis
│   │   ├── financial.rs       # Financial metrics
│   │   ├── battery.rs         # Battery health metrics
│   │   ├── efficiency.rs      # Efficiency metrics
│   │   └── strategy_stats.rs  # Per-strategy statistics
│   ├── scenarios/          # Test scenario definitions
│   │   └── mod.rs             # Builders & predefined scenarios
│   └── reports/            # Report generation
│       └── mod.rs             # Markdown/JSON/Text output
└── tests/
    ├── strategy_benchmark.rs  # Main benchmark tests
    └── regression.rs          # Regression tests
```

## Best Practices

### For Development

1. **Always run regression tests** before committing strategy changes
2. **Use synthetic data** for quick iteration
3. **Compare against baseline** (self-use only) to measure improvements
4. **Monitor battery health** metrics to avoid over-optimization

### For Production Testing

1. **Use real historical data** (OTE-CR) for at least 30 days
2. **Test multiple seasons** (winter vs summer behavior differs)
3. **Include price spike periods** to test strategy robustness
4. **Validate with real consumption** data from actual households
5. **Check all battery health warnings** before deploying

### For Performance Analysis

1. **Focus on net profit** as primary metric
2. **Balance profit vs battery health** (cycles, DoD)
3. **Consider self-consumption** for sustainability goals
4. **Analyze strategy win rates** to understand behavior patterns
5. **Review detailed logs** for unexpected strategy decisions

## Troubleshooting

### "Price data not loaded"
- Ensure you call `simulator.set_price_data()` before running
- Check if OTE-CR is accessible or use manual CSV

### "Access denied (403)" from OTE-CR
- Use manual CSV export: `PRICE_DATA_CSV=/path/to/prices.csv`
- Visit OTE-CR website and export data manually

### Tests are slow
- Use shorter date ranges for development
- Use regression tests with synthetic data (fast)
- Run with `--test-threads=1` for accurate timing

### Unexpected negative profit
- Check price data units (should be CZK/kWh, not EUR/MWh)
- Verify battery wear cost is reasonable (default: 0.125 CZK/kWh)
- Review solar generation settings (seasonal profile enabled?)

## Contributing

When adding new strategies or modifying existing ones:

1. **Add regression test** to ensure baseline performance
2. **Document expected behavior** in scenario expectations
3. **Run full benchmark suite** before submitting PR
4. **Include benchmark report** showing improvements
5. **Check for battery health** issues (over-cycling, low SOC)

## License

AGPL-3.0-or-later

Copyright (c) 2025 SOLARE S.R.O.

For commercial licensing, please contact: info@solare.cz
