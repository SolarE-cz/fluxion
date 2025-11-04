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

//! Battery health and usage metrics

use serde::{Deserialize, Serialize};

/// Battery health and usage metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BatteryMetrics {
    /// Total energy charged to battery (kWh)
    pub total_charge_kwh: f32,

    /// Total energy discharged from battery (kWh)
    pub total_discharge_kwh: f32,

    /// Total full charge/discharge cycles
    /// (calculated as total_charge_kwh / battery_capacity_kwh)
    pub total_cycles: f32,

    /// Average depth of discharge (%)
    pub average_dod: f32,

    /// Minimum SOC reached (%)
    pub min_soc: f32,

    /// Maximum SOC reached (%)
    pub max_soc: f32,

    /// Number of blocks with SOC 0-20%
    pub soc_distribution_0_20: usize,

    /// Number of blocks with SOC 20-40%
    pub soc_distribution_20_40: usize,

    /// Number of blocks with SOC 40-60%
    pub soc_distribution_40_60: usize,

    /// Number of blocks with SOC 60-80%
    pub soc_distribution_60_80: usize,

    /// Number of blocks with SOC 80-100%
    pub soc_distribution_80_100: usize,
}

impl BatteryMetrics {
    /// Create new battery metrics with zero values
    pub fn new() -> Self {
        Self {
            total_charge_kwh: 0.0,
            total_discharge_kwh: 0.0,
            total_cycles: 0.0,
            average_dod: 0.0,
            min_soc: 100.0,
            max_soc: 0.0,
            soc_distribution_0_20: 0,
            soc_distribution_20_40: 0,
            soc_distribution_40_60: 0,
            soc_distribution_60_80: 0,
            soc_distribution_80_100: 0,
        }
    }

    /// Calculate battery efficiency
    /// Efficiency = (discharge / charge) * 100
    pub fn round_trip_efficiency_percent(&self) -> f32 {
        if self.total_charge_kwh == 0.0 {
            return 0.0;
        }
        (self.total_discharge_kwh / self.total_charge_kwh) * 100.0
    }

    /// Estimate remaining lifespan based on cycle count
    /// Typical Li-ion battery: 6000-10000 cycles to 80% capacity
    pub fn estimated_lifespan_years(&self, expected_total_cycles: f32, duration_days: f32) -> f32 {
        if self.total_cycles == 0.0 || duration_days == 0.0 {
            return 0.0;
        }

        let cycles_per_day = self.total_cycles / duration_days;
        let cycles_per_year = cycles_per_day * 365.0;

        if cycles_per_year == 0.0 {
            return 0.0;
        }

        expected_total_cycles / cycles_per_year
    }

    /// Check if battery usage is within healthy range
    /// Returns warnings if usage patterns might harm battery
    pub fn health_warnings(&self) -> Vec<String> {
        let mut warnings = Vec::new();

        if self.min_soc < 10.0 {
            warnings.push(format!(
                "Battery discharged to very low SOC ({}%). This may reduce lifespan.",
                self.min_soc
            ));
        }

        if self.average_dod > 80.0 {
            warnings.push(format!(
                "High average depth of discharge ({}%). Consider reducing cycle depth.",
                self.average_dod
            ));
        }

        // Check SOC distribution - battery should not spend too much time at extremes
        let total_blocks = self.soc_distribution_0_20
            + self.soc_distribution_20_40
            + self.soc_distribution_40_60
            + self.soc_distribution_60_80
            + self.soc_distribution_80_100;

        if total_blocks > 0 {
            let extreme_percent =
                ((self.soc_distribution_0_20 + self.soc_distribution_80_100) as f32
                    / total_blocks as f32)
                    * 100.0;

            if extreme_percent > 50.0 {
                warnings.push(format!(
                    "Battery spends {:.1}% of time at extreme SOC levels (0-20% or 80-100%)",
                    extreme_percent
                ));
            }
        }

        warnings
    }
}

impl Default for BatteryMetrics {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_round_trip_efficiency() {
        let mut metrics = BatteryMetrics::new();
        metrics.total_charge_kwh = 100.0;
        metrics.total_discharge_kwh = 95.0;

        assert_eq!(metrics.round_trip_efficiency_percent(), 95.0);
    }

    #[test]
    fn test_lifespan_estimation() {
        let mut metrics = BatteryMetrics::new();
        metrics.total_cycles = 100.0;

        // 100 cycles in 30 days = 3.33 cycles/day = 1216.67 cycles/year
        // 6000 cycles / 1216.67 = ~4.93 years
        let lifespan = metrics.estimated_lifespan_years(6000.0, 30.0);
        assert!((lifespan - 4.93).abs() < 0.1);
    }

    #[test]
    fn test_health_warnings_low_soc() {
        let mut metrics = BatteryMetrics::new();
        metrics.min_soc = 5.0;

        let warnings = metrics.health_warnings();
        assert!(!warnings.is_empty());
        assert!(warnings[0].contains("very low SOC"));
    }

    #[test]
    fn test_health_warnings_high_dod() {
        let mut metrics = BatteryMetrics::new();
        metrics.average_dod = 85.0;

        let warnings = metrics.health_warnings();
        assert!(!warnings.is_empty());
        assert!(warnings[0].contains("depth of discharge"));
    }
}
