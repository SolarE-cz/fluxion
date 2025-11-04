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

//! Strategy simulation engine for running strategies on historical data

mod runner;

pub use runner::{BenchmarkConfig, SimulationResult, Simulator};

use chrono::{DateTime, Utc};
use fluxion_core::components::TimeBlockPrice;
use serde::{Deserialize, Serialize};

/// State of the battery at a point in time
#[derive(Debug, Clone)]
pub struct BatteryState {
    /// State of charge (%)
    pub soc_percent: f32,
    /// Battery capacity (kWh)
    pub capacity_kwh: f32,
    /// Current energy stored (kWh)
    pub energy_stored_kwh: f32,
}

impl BatteryState {
    /// Create new battery state
    pub fn new(capacity_kwh: f32, initial_soc_percent: f32) -> Self {
        let energy_stored_kwh = capacity_kwh * (initial_soc_percent / 100.0);
        Self {
            soc_percent: initial_soc_percent,
            capacity_kwh,
            energy_stored_kwh,
        }
    }

    /// Charge the battery
    /// Returns actual energy charged (may be less if battery is full)
    pub fn charge(&mut self, energy_kwh: f32, efficiency: f32) -> f32 {
        let available_capacity = self.capacity_kwh - self.energy_stored_kwh;
        let actual_charge = energy_kwh.min(available_capacity);
        let energy_added = actual_charge * efficiency;

        self.energy_stored_kwh += energy_added;
        self.soc_percent = (self.energy_stored_kwh / self.capacity_kwh) * 100.0;

        actual_charge
    }

    /// Discharge the battery
    /// Returns actual energy discharged (may be less if battery is empty)
    pub fn discharge(&mut self, energy_kwh: f32, efficiency: f32) -> f32 {
        let available_energy = self.energy_stored_kwh;
        let actual_discharge = energy_kwh.min(available_energy / efficiency);
        let energy_removed = actual_discharge / efficiency;

        self.energy_stored_kwh -= energy_removed;
        self.soc_percent = (self.energy_stored_kwh / self.capacity_kwh) * 100.0;

        actual_discharge
    }

    /// Check if battery can accept charge
    pub fn can_charge(&self, min_capacity_percent: f32) -> bool {
        self.soc_percent < 100.0 - min_capacity_percent
    }

    /// Check if battery can discharge
    pub fn can_discharge(&self, min_soc_percent: f32) -> bool {
        self.soc_percent > min_soc_percent
    }

    /// Clamp SOC to valid range
    pub fn clamp_soc(&mut self, min_percent: f32, max_percent: f32) {
        self.soc_percent = self.soc_percent.clamp(min_percent, max_percent);
        self.energy_stored_kwh = self.capacity_kwh * (self.soc_percent / 100.0);
    }
}

/// Simulation environment for a single time block
#[derive(Debug, Clone)]
pub struct SimulationBlock {
    /// Time block start
    pub timestamp: DateTime<Utc>,
    /// Price data for this block
    pub price: TimeBlockPrice,
    /// Solar generation forecast (kWh)
    pub solar_kwh: f32,
    /// Consumption forecast (kWh)
    pub consumption_kwh: f32,
    /// Battery state at start of block
    pub battery_state: BatteryState,
}

/// Configuration for solar generation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SolarConfig {
    /// Peak capacity (kW)
    pub peak_capacity_kw: f32,
    /// Use seasonal profile (true) or constant (false)
    pub use_seasonal_profile: bool,
}

impl SolarConfig {
    /// Generate solar forecast for a time block
    /// Returns kWh for 15-minute block
    pub fn generate_forecast(&self, timestamp: DateTime<Utc>) -> f32 {
        if !self.use_seasonal_profile {
            return self.peak_capacity_kw * 0.25 * 0.5; // 50% of peak, 15min
        }

        let hour = timestamp.hour();
        let month = timestamp.month();

        // Simple solar generation model
        // Peak hours: 10 AM - 4 PM
        // Seasonal variation: Winter (Nov-Feb): 30%, Spring/Fall (Mar-May, Sep-Oct): 60%, Summer (Jun-Aug): 100%

        let seasonal_factor = match month {
            11 | 12 | 1 | 2 => 0.3,       // Winter
            3 | 4 | 5 | 9 | 10 => 0.6,    // Spring/Fall
            6 | 7 | 8 => 1.0,             // Summer
            _ => 0.5,
        };

        let hour_factor = match hour {
            6 => 0.1,
            7 => 0.3,
            8 => 0.5,
            9 => 0.7,
            10..=14 => 1.0,
            15 => 0.8,
            16 => 0.6,
            17 => 0.4,
            18 => 0.2,
            _ => 0.0,
        };

        self.peak_capacity_kw * 0.25 * seasonal_factor * hour_factor // 0.25 = 15 minutes
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::TimeZone;

    #[test]
    fn test_battery_charge() {
        let mut battery = BatteryState::new(20.0, 50.0);
        let charged = battery.charge(5.0, 0.95);

        assert_eq!(charged, 5.0);
        assert!((battery.energy_stored_kwh - 14.75).abs() < 0.1); // 10 + 5*0.95
    }

    #[test]
    fn test_battery_discharge() {
        let mut battery = BatteryState::new(20.0, 50.0);
        let discharged = battery.discharge(3.0, 0.95);

        assert_eq!(discharged, 3.0);
        assert!((battery.energy_stored_kwh - 6.84).abs() < 0.1); // 10 - 3/0.95
    }

    #[test]
    fn test_battery_limits() {
        let mut battery = BatteryState::new(20.0, 95.0);
        assert!(!battery.can_charge(10.0));
        assert!(battery.can_discharge(10.0));

        let mut battery2 = BatteryState::new(20.0, 15.0);
        assert!(battery2.can_charge(10.0));
        assert!(!battery2.can_discharge(20.0));
    }

    #[test]
    fn test_solar_generation_summer() {
        let config = SolarConfig {
            peak_capacity_kw: 10.0,
            use_seasonal_profile: true,
        };

        // Summer noon
        let timestamp = Utc.with_ymd_and_hms(2024, 7, 15, 12, 0, 0).unwrap();
        let solar = config.generate_forecast(timestamp);

        // Peak: 10 kW * 0.25h * 1.0 (summer) * 1.0 (noon) = 2.5 kWh
        assert!((solar - 2.5).abs() < 0.1);
    }

    #[test]
    fn test_solar_generation_winter() {
        let config = SolarConfig {
            peak_capacity_kw: 10.0,
            use_seasonal_profile: true,
        };

        // Winter noon
        let timestamp = Utc.with_ymd_and_hms(2024, 12, 15, 12, 0, 0).unwrap();
        let solar = config.generate_forecast(timestamp);

        // Peak: 10 kW * 0.25h * 0.3 (winter) * 1.0 (noon) = 0.75 kWh
        assert!((solar - 0.75).abs() < 0.1);
    }
}
