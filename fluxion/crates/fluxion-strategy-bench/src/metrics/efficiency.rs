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

//! Efficiency and utilization metrics

use serde::{Deserialize, Serialize};

/// Efficiency and utilization metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EfficiencyMetrics {
    /// Total solar energy generated (kWh)
    pub total_solar_generation_kwh: f32,

    /// Total household consumption (kWh)
    pub total_consumption_kwh: f32,

    /// Total energy imported from grid (kWh)
    pub total_grid_import_kwh: f32,

    /// Total energy exported to grid (kWh)
    pub total_grid_export_kwh: f32,

    /// Solar energy directly used (not exported) (kWh)
    pub self_consumption_kwh: f32,

    /// Self-consumption rate (%)
    /// = (self_consumption / total_solar) * 100
    pub self_consumption_rate: f32,

    /// Autarky rate (%)
    /// = (consumption covered by solar / total consumption) * 100
    pub autarky_rate: f32,

    /// Battery utilization rate (%)
    /// = (actual battery usage / maximum possible) * 100
    pub battery_utilization_rate: f32,
}

impl EfficiencyMetrics {
    /// Create new efficiency metrics with zero values
    pub fn new() -> Self {
        Self {
            total_solar_generation_kwh: 0.0,
            total_consumption_kwh: 0.0,
            total_grid_import_kwh: 0.0,
            total_grid_export_kwh: 0.0,
            self_consumption_kwh: 0.0,
            self_consumption_rate: 0.0,
            autarky_rate: 0.0,
            battery_utilization_rate: 0.0,
        }
    }

    /// Calculate energy balance
    /// Positive = net export, Negative = net import
    pub fn energy_balance_kwh(&self) -> f32 {
        self.total_grid_export_kwh - self.total_grid_import_kwh
    }

    /// Calculate grid independence score (0-100)
    /// Higher is better - combines autarky and self-consumption
    pub fn grid_independence_score(&self) -> f32 {
        (self.autarky_rate + self.self_consumption_rate) / 2.0
    }

    /// Calculate solar utilization efficiency
    /// How well we use the solar energy generated
    pub fn solar_utilization_percent(&self) -> f32 {
        if self.total_solar_generation_kwh == 0.0 {
            return 0.0;
        }

        // Solar used = consumption from solar (not from grid)
        let solar_used = self.total_consumption_kwh - self.total_grid_import_kwh;
        (solar_used / self.total_solar_generation_kwh).min(1.0) * 100.0
    }

    /// Get efficiency grade (A-F)
    pub fn efficiency_grade(&self) -> char {
        let score = self.grid_independence_score();

        if score >= 90.0 {
            'A'
        } else if score >= 80.0 {
            'B'
        } else if score >= 70.0 {
            'C'
        } else if score >= 60.0 {
            'D'
        } else if score >= 50.0 {
            'E'
        } else {
            'F'
        }
    }
}

impl Default for EfficiencyMetrics {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_energy_balance() {
        let mut metrics = EfficiencyMetrics::new();
        metrics.total_grid_export_kwh = 100.0;
        metrics.total_grid_import_kwh = 40.0;

        assert_eq!(metrics.energy_balance_kwh(), 60.0); // Net export
    }

    #[test]
    fn test_grid_independence_score() {
        let mut metrics = EfficiencyMetrics::new();
        metrics.autarky_rate = 80.0;
        metrics.self_consumption_rate = 90.0;

        assert_eq!(metrics.grid_independence_score(), 85.0);
    }

    #[test]
    fn test_efficiency_grade_a() {
        let mut metrics = EfficiencyMetrics::new();
        metrics.autarky_rate = 95.0;
        metrics.self_consumption_rate = 95.0;

        assert_eq!(metrics.efficiency_grade(), 'A');
    }

    #[test]
    fn test_efficiency_grade_f() {
        let mut metrics = EfficiencyMetrics::new();
        metrics.autarky_rate = 30.0;
        metrics.self_consumption_rate = 40.0;

        assert_eq!(metrics.efficiency_grade(), 'F');
    }

    #[test]
    fn test_solar_utilization() {
        let mut metrics = EfficiencyMetrics::new();
        metrics.total_solar_generation_kwh = 100.0;
        metrics.total_consumption_kwh = 80.0;
        metrics.total_grid_import_kwh = 10.0;

        // Solar used = 80 - 10 = 70 kWh
        // Utilization = 70 / 100 = 70%
        assert_eq!(metrics.solar_utilization_percent(), 70.0);
    }
}
