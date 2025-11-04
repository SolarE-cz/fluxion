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

//! Comprehensive strategy testing and benchmarking suite for FluxION
//!
//! This crate provides tools for:
//! - Fetching historical electricity price data from OTE-CR
//! - Loading real consumption data from Excel/CSV files
//! - Simulating battery operation with different strategies
//! - Analyzing financial, battery health, and efficiency metrics
//! - Generating comprehensive benchmark reports
//!
//! # Example
//!
//! ```no_run
//! use fluxion_strategy_bench::{Simulator, BenchmarkConfig};
//!
//! let config = BenchmarkConfig::default();
//! let mut simulator = Simulator::new(config);
//!
//! // Load historical data
//! simulator.load_price_data("2024-12-01", "2024-12-31")?;
//! simulator.load_consumption_from_excel("consumption.xlsx")?;
//!
//! // Run all strategies
//! let results = simulator.run_all_strategies()?;
//!
//! // Generate report
//! results.print_summary();
//! results.export_to_json("benchmark_results.json")?;
//! # Ok::<(), anyhow::Error>(())
//! ```

pub mod data_sources;
pub mod metrics;
pub mod reports;
pub mod scenarios;
pub mod simulator;

pub use data_sources::{ConsumptionData, PriceDataCache};
pub use metrics::{BatteryMetrics, EfficiencyMetrics, FinancialMetrics, StrategyMetrics};
pub use reports::{BenchmarkReport, ReportFormat};
pub use scenarios::{Scenario, ScenarioBuilder};
pub use simulator::{BenchmarkConfig, SimulationResult, Simulator};

/// Re-export commonly used types
pub mod prelude {
    pub use crate::data_sources::{ConsumptionData, PriceDataCache};
    pub use crate::metrics::{BatteryMetrics, EfficiencyMetrics, FinancialMetrics};
    pub use crate::reports::{BenchmarkReport, ReportFormat};
    pub use crate::scenarios::{Scenario, ScenarioBuilder};
    pub use crate::simulator::{BenchmarkConfig, Simulator};
}
