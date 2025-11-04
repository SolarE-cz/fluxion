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

//! Test scenario definitions and builders

use crate::data_sources::{ConsumptionData, ConsumptionProfile, HistoricalPriceData, PriceDataCache};
use crate::simulator::{BenchmarkConfig, Simulator};
use anyhow::Result;
use chrono::NaiveDate;
use serde::{Deserialize, Serialize};

/// A complete test scenario definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Scenario {
    /// Scenario name
    pub name: String,

    /// Description of what this scenario tests
    pub description: String,

    /// Start date for price data
    pub start_date: NaiveDate,

    /// End date for price data
    pub end_date: NaiveDate,

    /// Benchmark configuration
    pub config: BenchmarkConfig,

    /// Consumption profile type
    pub consumption_profile: String,

    /// Expected characteristics (for validation)
    pub expected_characteristics: Vec<String>,
}

impl Scenario {
    /// Create a new scenario
    pub fn new(name: impl Into<String>, description: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            description: description.into(),
            start_date: NaiveDate::from_ymd_opt(2024, 12, 1).unwrap(),
            end_date: NaiveDate::from_ymd_opt(2024, 12, 31).unwrap(),
            config: BenchmarkConfig::default(),
            consumption_profile: "TypicalHousehold".to_string(),
            expected_characteristics: Vec::new(),
        }
    }

    /// Build and run this scenario
    pub fn run(&self, price_cache: &mut PriceDataCache) -> Result<Simulator> {
        // Load price data
        let price_data = price_cache.get_price_data(self.start_date, self.end_date)?;

        // Generate consumption data based on profile
        let consumption_data = match self.consumption_profile.as_str() {
            "TypicalHousehold" => ConsumptionProfile::TypicalHousehold.generate(self.start_date, self.end_date),
            "SmallHousehold" => ConsumptionProfile::SmallHousehold.generate(self.start_date, self.end_date),
            "LargeHousehold" => ConsumptionProfile::LargeHousehold.generate(self.start_date, self.end_date),
            _ => ConsumptionProfile::TypicalHousehold.generate(self.start_date, self.end_date),
        };

        // Create simulator
        let mut simulator = Simulator::new(self.config.clone());
        simulator.set_price_data(price_data.clone());
        simulator.set_consumption_data(consumption_data);

        Ok(simulator)
    }
}

/// Builder for creating test scenarios
pub struct ScenarioBuilder {
    scenario: Scenario,
}

impl ScenarioBuilder {
    /// Create a new scenario builder
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            scenario: Scenario::new(name, ""),
        }
    }

    /// Set description
    pub fn description(mut self, desc: impl Into<String>) -> Self {
        self.scenario.description = desc.into();
        self
    }

    /// Set date range
    pub fn date_range(mut self, start: NaiveDate, end: NaiveDate) -> Self {
        self.scenario.start_date = start;
        self.scenario.end_date = end;
        self
    }

    /// Set battery capacity
    pub fn battery_capacity(mut self, kwh: f32) -> Self {
        self.scenario.config.battery_capacity_kwh = kwh;
        self
    }

    /// Set solar peak capacity
    pub fn solar_capacity(mut self, kw: f32) -> Self {
        self.scenario.config.solar_config.peak_capacity_kw = kw;
        self
    }

    /// Set consumption profile
    pub fn consumption_profile(mut self, profile: impl Into<String>) -> Self {
        self.scenario.consumption_profile = profile.into();
        self
    }

    /// Add expected characteristic
    pub fn expect(mut self, characteristic: impl Into<String>) -> Self {
        self.scenario.expected_characteristics.push(characteristic.into());
        self
    }

    /// Set full configuration
    pub fn config(mut self, config: BenchmarkConfig) -> Self {
        self.scenario.config = config;
        self
    }

    /// Build the scenario
    pub fn build(self) -> Scenario {
        self.scenario
    }
}

/// Predefined scenarios for common test cases
pub mod predefined {
    use super::*;

    /// Winter scenario with high prices and low solar
    pub fn winter_high_price() -> Scenario {
        ScenarioBuilder::new("Winter High Price")
            .description("December with high electricity prices and low solar generation")
            .date_range(
                NaiveDate::from_ymd_opt(2024, 12, 1).unwrap(),
                NaiveDate::from_ymd_opt(2024, 12, 31).unwrap(),
            )
            .battery_capacity(20.0)
            .solar_capacity(10.0)
            .consumption_profile("TypicalHousehold")
            .expect("Winter peak discharge should activate")
            .expect("Time-aware charge should prepare for evening peaks")
            .build()
    }

    /// Summer scenario with high solar and moderate prices
    pub fn summer_high_solar() -> Scenario {
        ScenarioBuilder::new("Summer High Solar")
            .description("July with abundant solar generation and moderate prices")
            .date_range(
                NaiveDate::from_ymd_opt(2024, 7, 1).unwrap(),
                NaiveDate::from_ymd_opt(2024, 7, 31).unwrap(),
            )
            .battery_capacity(20.0)
            .solar_capacity(10.0)
            .consumption_profile("TypicalHousehold")
            .expect("Solar-first strategy should dominate")
            .expect("High self-consumption rate")
            .build()
    }

    /// Price spike scenario
    pub fn price_spike() -> Scenario {
        ScenarioBuilder::new("Price Spike")
            .description("Period with extreme price volatility")
            .date_range(
                NaiveDate::from_ymd_opt(2024, 1, 15).unwrap(),
                NaiveDate::from_ymd_opt(2024, 1, 25).unwrap(),
            )
            .battery_capacity(20.0)
            .solar_capacity(10.0)
            .consumption_profile("TypicalHousehold")
            .expect("Price arbitrage should be highly profitable")
            .expect("Frequent charge/discharge cycles")
            .build()
    }

    /// Small system scenario
    pub fn small_system() -> Scenario {
        ScenarioBuilder::new("Small System")
            .description("Small household with limited battery and solar capacity")
            .date_range(
                NaiveDate::from_ymd_opt(2024, 6, 1).unwrap(),
                NaiveDate::from_ymd_opt(2024, 6, 30).unwrap(),
            )
            .battery_capacity(10.0)
            .solar_capacity(5.0)
            .consumption_profile("SmallHousehold")
            .expect("Limited battery utilization")
            .expect("Strategies should adapt to smaller capacity")
            .build()
    }

    /// Large system scenario
    pub fn large_system() -> Scenario {
        ScenarioBuilder::new("Large System")
            .description("Large household with high capacity battery and solar")
            .date_range(
                NaiveDate::from_ymd_opt(2024, 6, 1).unwrap(),
                NaiveDate::from_ymd_opt(2024, 6, 30).unwrap(),
            )
            .battery_capacity(30.0)
            .solar_capacity(15.0)
            .consumption_profile("LargeHousehold")
            .expect("High battery utilization")
            .expect("Maximum self-sufficiency potential")
            .build()
    }

    /// Get all predefined scenarios
    pub fn all() -> Vec<Scenario> {
        vec![
            winter_high_price(),
            summer_high_solar(),
            price_spike(),
            small_system(),
            large_system(),
        ]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_scenario_builder() {
        let scenario = ScenarioBuilder::new("Test Scenario")
            .description("A test scenario")
            .battery_capacity(25.0)
            .solar_capacity(12.0)
            .consumption_profile("LargeHousehold")
            .expect("Something should happen")
            .build();

        assert_eq!(scenario.name, "Test Scenario");
        assert_eq!(scenario.config.battery_capacity_kwh, 25.0);
        assert_eq!(scenario.config.solar_config.peak_capacity_kw, 12.0);
        assert_eq!(scenario.consumption_profile, "LargeHousehold");
        assert_eq!(scenario.expected_characteristics.len(), 1);
    }

    #[test]
    fn test_predefined_scenarios() {
        let scenarios = predefined::all();
        assert_eq!(scenarios.len(), 5);

        let winter = &scenarios[0];
        assert!(winter.name.contains("Winter"));
    }
}
