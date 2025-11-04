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

//! Performance metrics for strategy benchmarking

mod battery;
mod efficiency;
mod financial;
mod strategy_stats;

pub use battery::BatteryMetrics;
pub use efficiency::EfficiencyMetrics;
pub use financial::FinancialMetrics;
pub use strategy_stats::{StrategyMetrics, StrategyStatistics};

use fluxion_core::components::InverterOperationMode;
use serde::{Deserialize, Serialize};

/// Complete metrics summary for a simulation run
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetricsSummary {
    /// Financial performance metrics
    pub financial: FinancialMetrics,

    /// Battery health and usage metrics
    pub battery: BatteryMetrics,

    /// Efficiency and utilization metrics
    pub efficiency: EfficiencyMetrics,

    /// Per-strategy statistics
    pub strategy_stats: Vec<StrategyStatistics>,

    /// Total simulation duration in days
    pub duration_days: f32,

    /// Total number of 15-minute blocks simulated
    pub total_blocks: usize,
}

impl MetricsSummary {
    /// Create a new empty metrics summary
    pub fn new() -> Self {
        Self {
            financial: FinancialMetrics::new(),
            battery: BatteryMetrics::new(),
            efficiency: EfficiencyMetrics::new(),
            strategy_stats: Vec::new(),
            duration_days: 0.0,
            total_blocks: 0,
        }
    }

    /// Calculate improvement percentage compared to baseline
    pub fn improvement_vs_baseline(&self, baseline: &MetricsSummary) -> f32 {
        if baseline.financial.net_profit_czk == 0.0 {
            return 0.0;
        }

        ((self.financial.net_profit_czk - baseline.financial.net_profit_czk)
            / baseline.financial.net_profit_czk.abs())
            * 100.0
    }

    /// Get daily average profit
    pub fn daily_average_profit(&self) -> f32 {
        if self.duration_days == 0.0 {
            return 0.0;
        }
        self.financial.net_profit_czk / self.duration_days
    }

    /// Get cost per kWh cycled
    pub fn cost_per_kwh_cycled(&self) -> f32 {
        let total_cycled = self.battery.total_charge_kwh + self.battery.total_discharge_kwh;
        if total_cycled == 0.0 {
            return 0.0;
        }
        self.financial.battery_wear_cost_czk / total_cycled
    }
}

impl Default for MetricsSummary {
    fn default() -> Self {
        Self::new()
    }
}

/// Record of a single operation block for metrics tracking
#[derive(Debug, Clone)]
pub struct BlockRecord {
    /// Operation mode used
    pub mode: InverterOperationMode,

    /// Strategy that made the decision
    pub strategy_name: String,

    /// Energy imported from grid (kWh)
    pub grid_import_kwh: f32,

    /// Energy exported to grid (kWh)
    pub grid_export_kwh: f32,

    /// Energy charged to battery (kWh)
    pub battery_charge_kwh: f32,

    /// Energy discharged from battery (kWh)
    pub battery_discharge_kwh: f32,

    /// Solar energy generated (kWh)
    pub solar_generation_kwh: f32,

    /// Household consumption (kWh)
    pub consumption_kwh: f32,

    /// Import price (CZK/kWh)
    pub import_price: f32,

    /// Export price (CZK/kWh)
    pub export_price: f32,

    /// Battery SOC at start of block (%)
    pub soc_start: f32,

    /// Battery SOC at end of block (%)
    pub soc_end: f32,

    /// Revenue from this block (CZK)
    pub revenue_czk: f32,

    /// Cost for this block (CZK)
    pub cost_czk: f32,

    /// Net profit for this block (CZK)
    pub net_profit_czk: f32,
}

/// Collector for building metrics from block records
pub struct MetricsCollector {
    records: Vec<BlockRecord>,
    battery_capacity_kwh: f32,
    battery_wear_cost_czk_per_kwh: f32,
}

impl MetricsCollector {
    /// Create a new metrics collector
    pub fn new(battery_capacity_kwh: f32, battery_wear_cost_czk_per_kwh: f32) -> Self {
        Self {
            records: Vec::new(),
            battery_capacity_kwh,
            battery_wear_cost_czk_per_kwh,
        }
    }

    /// Add a block record to the collector
    pub fn add_record(&mut self, record: BlockRecord) {
        self.records.push(record);
    }

    /// Build complete metrics summary from collected records
    pub fn build_summary(&self, duration_days: f32) -> MetricsSummary {
        let mut summary = MetricsSummary::new();
        summary.duration_days = duration_days;
        summary.total_blocks = self.records.len();

        // Calculate financial metrics
        summary.financial = self.calculate_financial_metrics();

        // Calculate battery metrics
        summary.battery = self.calculate_battery_metrics();

        // Calculate efficiency metrics
        summary.efficiency = self.calculate_efficiency_metrics();

        // Calculate per-strategy statistics
        summary.strategy_stats = self.calculate_strategy_statistics();

        summary
    }

    fn calculate_financial_metrics(&self) -> FinancialMetrics {
        let mut metrics = FinancialMetrics::new();

        for record in &self.records {
            metrics.total_revenue_czk += record.revenue_czk;
            metrics.total_cost_czk += record.cost_czk;
            metrics.grid_import_cost_czk += record.grid_import_kwh * record.import_price;
            metrics.grid_export_revenue_czk += record.grid_export_kwh * record.export_price;
        }

        // Calculate battery wear cost
        let total_cycled =
            self.records.iter().map(|r| r.battery_charge_kwh).sum::<f32>();
        metrics.battery_wear_cost_czk = total_cycled * self.battery_wear_cost_czk_per_kwh;

        // Net profit = revenue - costs
        metrics.net_profit_czk = metrics.total_revenue_czk - metrics.total_cost_czk;

        metrics
    }

    fn calculate_battery_metrics(&self) -> BatteryMetrics {
        let mut metrics = BatteryMetrics::new();

        for record in &self.records {
            metrics.total_charge_kwh += record.battery_charge_kwh;
            metrics.total_discharge_kwh += record.battery_discharge_kwh;
        }

        // Calculate cycles (full charge/discharge cycle = battery capacity)
        metrics.total_cycles = metrics.total_charge_kwh / self.battery_capacity_kwh;

        // Calculate average depth of discharge
        let soc_changes: Vec<f32> = self
            .records
            .iter()
            .map(|r| (r.soc_end - r.soc_start).abs())
            .collect();

        if !soc_changes.is_empty() {
            metrics.average_dod = soc_changes.iter().sum::<f32>() / soc_changes.len() as f32;
        }

        // SOC distribution (count blocks in different SOC ranges)
        for record in &self.records {
            let soc = (record.soc_start + record.soc_end) / 2.0;
            if soc < 20.0 {
                metrics.soc_distribution_0_20 += 1;
            } else if soc < 40.0 {
                metrics.soc_distribution_20_40 += 1;
            } else if soc < 60.0 {
                metrics.soc_distribution_40_60 += 1;
            } else if soc < 80.0 {
                metrics.soc_distribution_60_80 += 1;
            } else {
                metrics.soc_distribution_80_100 += 1;
            }
        }

        // Find min/max SOC
        metrics.min_soc = self
            .records
            .iter()
            .map(|r| r.soc_start.min(r.soc_end))
            .fold(100.0, f32::min);

        metrics.max_soc = self
            .records
            .iter()
            .map(|r| r.soc_start.max(r.soc_end))
            .fold(0.0, f32::max);

        metrics
    }

    fn calculate_efficiency_metrics(&self) -> EfficiencyMetrics {
        let mut metrics = EfficiencyMetrics::new();

        let total_solar: f32 = self.records.iter().map(|r| r.solar_generation_kwh).sum();
        let total_consumption: f32 = self.records.iter().map(|r| r.consumption_kwh).sum();
        let total_import: f32 = self.records.iter().map(|r| r.grid_import_kwh).sum();
        let total_export: f32 = self.records.iter().map(|r| r.grid_export_kwh).sum();

        metrics.total_solar_generation_kwh = total_solar;
        metrics.total_consumption_kwh = total_consumption;
        metrics.total_grid_import_kwh = total_import;
        metrics.total_grid_export_kwh = total_export;

        // Self-consumption = solar directly used (solar - export)
        metrics.self_consumption_kwh = total_solar - total_export;

        // Self-consumption rate = self-consumed / total solar
        if total_solar > 0.0 {
            metrics.self_consumption_rate = (metrics.self_consumption_kwh / total_solar) * 100.0;
        }

        // Autarky rate = consumption covered by solar / total consumption
        let solar_covered = total_consumption - total_import;
        if total_consumption > 0.0 {
            metrics.autarky_rate = (solar_covered / total_consumption).max(0.0) * 100.0;
        }

        // Battery utilization = actual cycled / maximum possible
        let max_possible_cycles =
            (self.records.len() as f32 * 0.25) * self.battery_capacity_kwh; // 0.25h per block
        let actual_cycled = self.records.iter().map(|r| r.battery_charge_kwh).sum::<f32>();
        if max_possible_cycles > 0.0 {
            metrics.battery_utilization_rate = (actual_cycled / max_possible_cycles) * 100.0;
        }

        metrics
    }

    fn calculate_strategy_statistics(&self) -> Vec<StrategyStatistics> {
        use std::collections::HashMap;

        let mut strategy_data: HashMap<String, StrategyMetrics> = HashMap::new();

        for record in &self.records {
            let entry = strategy_data
                .entry(record.strategy_name.clone())
                .or_insert_with(StrategyMetrics::new);

            entry.total_blocks += 1;
            entry.total_profit_czk += record.net_profit_czk;
            entry.total_charge_kwh += record.battery_charge_kwh;
            entry.total_discharge_kwh += record.battery_discharge_kwh;

            if record.net_profit_czk > 0.0 {
                entry.profitable_blocks += 1;
            }
        }

        // Convert to statistics
        strategy_data
            .into_iter()
            .map(|(name, metrics)| {
                let win_rate = if self.records.is_empty() {
                    0.0
                } else {
                    (metrics.total_blocks as f32 / self.records.len() as f32) * 100.0
                };

                let profit_rate = if metrics.total_blocks == 0 {
                    0.0
                } else {
                    (metrics.profitable_blocks as f32 / metrics.total_blocks as f32) * 100.0
                };

                StrategyStatistics {
                    strategy_name: name,
                    total_blocks: metrics.total_blocks,
                    win_rate_percent: win_rate,
                    total_profit_czk: metrics.total_profit_czk,
                    average_profit_per_block: if metrics.total_blocks == 0 {
                        0.0
                    } else {
                        metrics.total_profit_czk / metrics.total_blocks as f32
                    },
                    profitable_blocks: metrics.profitable_blocks,
                    profit_rate_percent: profit_rate,
                    total_charge_kwh: metrics.total_charge_kwh,
                    total_discharge_kwh: metrics.total_discharge_kwh,
                }
            })
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_metrics_summary_creation() {
        let summary = MetricsSummary::new();
        assert_eq!(summary.total_blocks, 0);
        assert_eq!(summary.duration_days, 0.0);
    }

    #[test]
    fn test_daily_average_profit() {
        let mut summary = MetricsSummary::new();
        summary.financial.net_profit_czk = 300.0;
        summary.duration_days = 10.0;

        assert_eq!(summary.daily_average_profit(), 30.0);
    }

    #[test]
    fn test_improvement_calculation() {
        let mut baseline = MetricsSummary::new();
        baseline.financial.net_profit_czk = 100.0;

        let mut improved = MetricsSummary::new();
        improved.financial.net_profit_czk = 150.0;

        assert_eq!(improved.improvement_vs_baseline(&baseline), 50.0);
    }
}
