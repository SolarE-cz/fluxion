// Copyright (c) 2025 SOLARE S.R.O.
//
// This file is part of FluxION.
//
// FluxION is free software: you can redistribute it and/or modify
// it under the terms of the GNU Affero General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.
//
// FluxION is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the
// GNU Affero General Public License for more details.
//
// You should have received a copy of the GNU Affero General Public License
// along with FluxION. If not, see <https://www.gnu.org/licenses/>.
//
// For commercial licensing, please contact: info@solare.cz

//! Main simulation runner for strategy benchmarking

use super::{BatteryState, SolarConfig};
use crate::data_sources::{ConsumptionData, HistoricalPriceData};
use crate::metrics::{BlockRecord, MetricsCollector, MetricsSummary};
use anyhow::{Context, Result};
use chrono::{DateTime, Duration, Utc};
use fluxion_core::{
    components::{InverterOperationMode, TimeBlockPrice},
    resources::ControlConfig,
    strategy::*,
};
use indicatif::{ProgressBar, ProgressStyle};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tracing::{debug, info};

/// Configuration for benchmark simulation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BenchmarkConfig {
    /// Battery capacity (kWh)
    pub battery_capacity_kwh: f32,

    /// Initial battery SOC (%)
    pub initial_soc_percent: f32,

    /// Minimum allowed SOC (%)
    pub min_battery_soc: f32,

    /// Maximum allowed SOC (%)
    pub max_battery_soc: f32,

    /// Battery round-trip efficiency (0.0-1.0)
    pub battery_efficiency: f32,

    /// Battery wear cost (CZK/kWh cycled)
    pub battery_wear_cost_czk_per_kwh: f32,

    /// Maximum battery charge rate (kW)
    pub max_battery_charge_rate_kw: f32,

    /// Maximum battery discharge rate (kW)
    pub max_battery_discharge_rate_kw: f32,

    /// Solar configuration
    pub solar_config: SolarConfig,

    /// Average household load (kW) - fallback if no consumption data
    pub average_household_load_kw: f32,

    /// Evening peak start hour (for evening preparation strategies)
    pub evening_peak_start_hour: u32,

    /// Evening target SOC (%)
    pub evening_target_soc: f32,
}

impl Default for BenchmarkConfig {
    fn default() -> Self {
        Self {
            battery_capacity_kwh: 20.0,
            initial_soc_percent: 50.0,
            min_battery_soc: 15.0,
            max_battery_soc: 100.0,
            battery_efficiency: 0.95,
            battery_wear_cost_czk_per_kwh: 0.125,
            max_battery_charge_rate_kw: 5.0,
            max_battery_discharge_rate_kw: 5.0,
            solar_config: SolarConfig {
                peak_capacity_kw: 10.0,
                use_seasonal_profile: true,
            },
            average_household_load_kw: 0.5,
            evening_peak_start_hour: 17,
            evening_target_soc: 90.0,
        }
    }
}

impl BenchmarkConfig {
    /// Convert to ControlConfig for strategy evaluation
    fn to_control_config(&self) -> ControlConfig {
        ControlConfig {
            max_battery_soc: self.max_battery_soc,
            min_battery_soc: self.min_battery_soc,
            battery_capacity_kwh: self.battery_capacity_kwh,
            battery_efficiency: self.battery_efficiency,
            battery_wear_cost_czk_per_kwh: self.battery_wear_cost_czk_per_kwh,
            max_battery_charge_rate_kw: self.max_battery_charge_rate_kw,
            max_battery_discharge_rate_kw: self.max_battery_discharge_rate_kw,
            average_household_load_kw: self.average_household_load_kw,
            hardware_min_battery_soc: 10.0,
            evening_peak_start_hour: self.evening_peak_start_hour,
            evening_target_soc: self.evening_target_soc,
            force_charge_hours: 4,
            force_discharge_hours: 2,
            min_mode_change_interval_secs: 0, // Not relevant for simulation
            maximum_export_power_w: 5000,
        }
    }
}

/// Result of a simulation run
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SimulationResult {
    /// Name of the strategy or combination used
    pub strategy_name: String,

    /// Configuration used
    pub config: BenchmarkConfig,

    /// Comprehensive metrics summary
    pub metrics: MetricsSummary,

    /// Start timestamp
    pub start_time: DateTime<Utc>,

    /// End timestamp
    pub end_time: DateTime<Utc>,
}

/// Main simulator for running strategies on historical data
pub struct Simulator {
    config: BenchmarkConfig,
    price_data: Option<HistoricalPriceData>,
    consumption_data: Option<ConsumptionData>,
}

impl Simulator {
    /// Create a new simulator with given configuration
    pub fn new(config: BenchmarkConfig) -> Self {
        Self {
            config,
            price_data: None,
            consumption_data: None,
        }
    }

    /// Load price data
    pub fn set_price_data(&mut self, price_data: HistoricalPriceData) {
        self.price_data = Some(price_data);
    }

    /// Load consumption data
    pub fn set_consumption_data(&mut self, consumption_data: ConsumptionData) {
        self.consumption_data = Some(consumption_data);
    }

    /// Run simulation with economic optimizer (combined strategies)
    pub fn run_economic_optimizer(&self) -> Result<SimulationResult> {
        info!("🚀 Running simulation with Economic Optimizer (all strategies)");

        let price_data = self
            .price_data
            .as_ref()
            .context("Price data not loaded")?;

        // Create all strategies
        let strategies = self.create_all_strategies();
        let optimizer = EconomicOptimizer::new(strategies);

        self.run_with_strategy_engine("Economic-Optimizer", &optimizer, price_data)
    }

    /// Run simulation with a single strategy
    pub fn run_single_strategy(&self, strategy_name: &str) -> Result<SimulationResult> {
        info!("🚀 Running simulation with strategy: {}", strategy_name);

        let price_data = self
            .price_data
            .as_ref()
            .context("Price data not loaded")?;

        let strategy = self.create_strategy_by_name(strategy_name)?;

        self.run_with_strategy_engine(strategy_name, strategy.as_ref(), price_data)
    }

    /// Run simulation with self-use only (baseline)
    pub fn run_baseline(&self) -> Result<SimulationResult> {
        info!("🚀 Running baseline simulation (Self-Use only)");

        let price_data = self
            .price_data
            .as_ref()
            .context("Price data not loaded")?;

        let strategy = SelfUseStrategy::default();

        self.run_with_strategy_engine("Baseline-SelfUse", &strategy, price_data)
    }

    /// Run all strategies individually and return results
    pub fn run_all_strategies_individually(&self) -> Result<Vec<SimulationResult>> {
        let strategy_names = vec![
            "Price-Arbitrage",
            "Solar-First",
            "Time-Aware-Charge",
            "Morning-PreCharge",
            "Day-Ahead-Planning",
            "Self-Use",
        ];

        let mut results = Vec::new();

        for name in strategy_names {
            match self.run_single_strategy(name) {
                Ok(result) => results.push(result),
                Err(e) => {
                    tracing::warn!("Failed to run strategy {}: {}", name, e);
                }
            }
        }

        // Also run the economic optimizer
        match self.run_economic_optimizer() {
            Ok(result) => results.push(result),
            Err(e) => {
                tracing::warn!("Failed to run economic optimizer: {}", e);
            }
        }

        // Also run baseline
        match self.run_baseline() {
            Ok(result) => results.push(result),
            Err(e) => {
                tracing::warn!("Failed to run baseline: {}", e);
            }
        }

        Ok(results)
    }

    /// Internal method to run simulation with a strategy engine
    fn run_with_strategy_engine(
        &self,
        strategy_name: &str,
        strategy: &dyn EconomicStrategy,
        price_data: &HistoricalPriceData,
    ) -> Result<SimulationResult> {
        let control_config = self.config.to_control_config();

        // Initialize battery state
        let mut battery_state = BatteryState::new(
            self.config.battery_capacity_kwh,
            self.config.initial_soc_percent,
        );

        // Initialize metrics collector
        let mut collector = MetricsCollector::new(
            self.config.battery_capacity_kwh,
            self.config.battery_wear_cost_czk_per_kwh,
        );

        // Setup progress bar
        let pb = ProgressBar::new(price_data.prices.len() as u64);
        pb.set_style(
            ProgressStyle::default_bar()
                .template("[{elapsed_precise}] {bar:40.cyan/blue} {pos}/{len} blocks {msg}")?
                .progress_chars("=>-"),
        );

        let start_time = price_data.start_date;
        let end_time = price_data.end_date;

        // Simulate each 15-minute block
        for price_point in &price_data.prices {
            let timestamp = price_point.timestamp;

            // Generate solar forecast
            let solar_kwh = self.config.solar_config.generate_forecast(timestamp);

            // Get consumption forecast
            let consumption_kwh = if let Some(consumption_data) = &self.consumption_data {
                consumption_data.get_consumption_at(timestamp) * 0.25 // kW * 15min = kWh
            } else {
                self.config.average_household_load_kw * 0.25
            };

            // Create price block
            let price_block = TimeBlockPrice {
                block_start: timestamp,
                duration_minutes: 15,
                price_czk_per_kwh: price_point.price_czk_per_kwh,
            };

            // Create evaluation context
            let context = EvaluationContext {
                price_block: &price_block,
                control_config: &control_config,
                current_battery_soc: battery_state.soc_percent,
                solar_forecast_kwh: solar_kwh,
                consumption_forecast_kwh: consumption_kwh,
                grid_export_price_czk_per_kwh: price_point.price_czk_per_kwh * 0.9, // Assume 90% of import price
                all_price_blocks: Some(&price_data.prices.iter().map(|p| TimeBlockPrice {
                    block_start: p.timestamp,
                    duration_minutes: 15,
                    price_czk_per_kwh: p.price_czk_per_kwh,
                }).collect::<Vec<_>>()),
            };

            // Evaluate strategy
            let evaluation = strategy.evaluate(&context);

            // Simulate the operation
            let record = self.simulate_block(
                &mut battery_state,
                evaluation.mode,
                strategy_name,
                solar_kwh,
                consumption_kwh,
                price_point.price_czk_per_kwh,
                price_point.price_czk_per_kwh * 0.9,
            );

            collector.add_record(record);

            pb.set_message(format!(
                "SOC: {:.1}%, Profit: {:.2} CZK",
                battery_state.soc_percent,
                collector.build_summary(1.0).financial.net_profit_czk
            ));
            pb.inc(1);
        }

        pb.finish_with_message("✓ Simulation complete");

        // Calculate duration in days
        let duration_days = (end_time.timestamp() - start_time.timestamp()) as f32 / 86400.0;

        // Build final metrics
        let metrics = collector.build_summary(duration_days);

        info!(
            "✓ Simulation complete: {:.2} CZK profit over {:.1} days",
            metrics.financial.net_profit_czk, duration_days
        );

        Ok(SimulationResult {
            strategy_name: strategy_name.to_string(),
            config: self.config.clone(),
            metrics,
            start_time,
            end_time,
        })
    }

    /// Simulate a single 15-minute block
    fn simulate_block(
        &self,
        battery_state: &mut BatteryState,
        mode: InverterOperationMode,
        strategy_name: &str,
        solar_kwh: f32,
        consumption_kwh: f32,
        import_price: f32,
        export_price: f32,
    ) -> BlockRecord {
        let soc_start = battery_state.soc_percent;

        let mut grid_import = 0.0;
        let mut grid_export = 0.0;
        let mut battery_charge = 0.0;
        let mut battery_discharge = 0.0;

        // Simulate based on operation mode
        match mode {
            InverterOperationMode::ForceCharge => {
                // Charge battery from grid + solar
                let max_charge = self.config.max_battery_charge_rate_kw * 0.25; // 15 min
                let charge_amount = battery_state.charge(max_charge, self.config.battery_efficiency);
                battery_charge = charge_amount;

                // Grid must supply charge + consumption - solar
                grid_import = (charge_amount + consumption_kwh - solar_kwh).max(0.0);

                // Any excess solar is exported
                if solar_kwh > consumption_kwh + charge_amount {
                    grid_export = solar_kwh - consumption_kwh - charge_amount;
                }
            }

            InverterOperationMode::ForceDischarge => {
                // Discharge battery to grid
                let max_discharge = self.config.max_battery_discharge_rate_kw * 0.25;
                let discharge_amount = battery_state.discharge(max_discharge, self.config.battery_efficiency);
                battery_discharge = discharge_amount;

                // Export: battery discharge + solar - consumption
                let total_available = discharge_amount + solar_kwh;
                if total_available > consumption_kwh {
                    grid_export = total_available - consumption_kwh;
                } else {
                    grid_import = consumption_kwh - total_available;
                }
            }

            InverterOperationMode::SelfUse => {
                // Use solar first, then battery, then grid
                if solar_kwh >= consumption_kwh {
                    // Solar covers consumption
                    let excess_solar = solar_kwh - consumption_kwh;

                    // Store excess in battery if possible
                    let stored = battery_state.charge(excess_solar, self.config.battery_efficiency);
                    battery_charge = stored;

                    // Export remaining
                    grid_export = excess_solar - stored;
                } else {
                    // Solar doesn't cover consumption
                    let deficit = consumption_kwh - solar_kwh;

                    // Try to cover from battery
                    let from_battery =
                        battery_state.discharge(deficit, self.config.battery_efficiency);
                    battery_discharge = from_battery;

                    // Import remaining
                    grid_import = deficit - from_battery;
                }
            }

            _ => {
                // Other modes: treat as self-use
                if solar_kwh >= consumption_kwh {
                    let excess_solar = solar_kwh - consumption_kwh;
                    let stored = battery_state.charge(excess_solar, self.config.battery_efficiency);
                    battery_charge = stored;
                    grid_export = excess_solar - stored;
                } else {
                    let deficit = consumption_kwh - solar_kwh;
                    let from_battery =
                        battery_state.discharge(deficit, self.config.battery_efficiency);
                    battery_discharge = from_battery;
                    grid_import = deficit - from_battery;
                }
            }
        }

        // Ensure SOC is within limits
        battery_state.clamp_soc(self.config.min_battery_soc, self.config.max_battery_soc);

        let soc_end = battery_state.soc_percent;

        // Calculate revenue and costs
        let revenue = grid_export * export_price;
        let cost = grid_import * import_price
            + battery_charge * self.config.battery_wear_cost_czk_per_kwh
            + battery_discharge * self.config.battery_wear_cost_czk_per_kwh;

        BlockRecord {
            mode,
            strategy_name: strategy_name.to_string(),
            grid_import_kwh: grid_import,
            grid_export_kwh: grid_export,
            battery_charge_kwh: battery_charge,
            battery_discharge_kwh: battery_discharge,
            solar_generation_kwh: solar_kwh,
            consumption_kwh,
            import_price,
            export_price,
            soc_start,
            soc_end,
            revenue_czk: revenue,
            cost_czk: cost,
            net_profit_czk: revenue - cost,
        }
    }

    /// Create all available strategies
    fn create_all_strategies(&self) -> Vec<Arc<dyn EconomicStrategy>> {
        vec![
            Arc::new(PriceArbitrageStrategy::default()),
            Arc::new(SolarFirstStrategy::default()),
            Arc::new(TimeAwareChargeStrategy::default()),
            Arc::new(MorningPreChargeStrategy::default()),
            Arc::new(DayAheadChargePlanningStrategy::default()),
            Arc::new(SelfUseStrategy::default()),
        ]
    }

    /// Create a strategy by name
    fn create_strategy_by_name(&self, name: &str) -> Result<Arc<dyn EconomicStrategy>> {
        match name {
            "Price-Arbitrage" => Ok(Arc::new(PriceArbitrageStrategy::default())),
            "Solar-First" => Ok(Arc::new(SolarFirstStrategy::default())),
            "Time-Aware-Charge" => Ok(Arc::new(TimeAwareChargeStrategy::default())),
            "Morning-PreCharge" => Ok(Arc::new(MorningPreChargeStrategy::default())),
            "Day-Ahead-Planning" => Ok(Arc::new(DayAheadChargePlanningStrategy::default())),
            "Self-Use" => Ok(Arc::new(SelfUseStrategy::default())),
            _ => anyhow::bail!("Unknown strategy: {}", name),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_benchmark_config_default() {
        let config = BenchmarkConfig::default();
        assert_eq!(config.battery_capacity_kwh, 20.0);
        assert_eq!(config.battery_efficiency, 0.95);
    }

    #[test]
    fn test_control_config_conversion() {
        let bench_config = BenchmarkConfig::default();
        let control_config = bench_config.to_control_config();

        assert_eq!(
            control_config.battery_capacity_kwh,
            bench_config.battery_capacity_kwh
        );
        assert_eq!(
            control_config.battery_efficiency,
            bench_config.battery_efficiency
        );
    }
}
