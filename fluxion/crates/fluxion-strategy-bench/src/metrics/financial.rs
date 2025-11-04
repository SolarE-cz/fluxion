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

//! Financial performance metrics

use serde::{Deserialize, Serialize};

/// Financial performance metrics for a simulation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FinancialMetrics {
    /// Total revenue from all operations (CZK)
    pub total_revenue_czk: f32,

    /// Total costs from all operations (CZK)
    pub total_cost_czk: f32,

    /// Net profit (revenue - costs) (CZK)
    pub net_profit_czk: f32,

    /// Revenue from grid export (CZK)
    pub grid_export_revenue_czk: f32,

    /// Cost of grid import (CZK)
    pub grid_import_cost_czk: f32,

    /// Cost of battery degradation (CZK)
    pub battery_wear_cost_czk: f32,

    /// Revenue from avoided grid import (CZK)
    /// This is the money saved by using battery/solar instead of buying from grid
    pub avoided_import_revenue_czk: f32,
}

impl FinancialMetrics {
    /// Create new financial metrics with zero values
    pub fn new() -> Self {
        Self {
            total_revenue_czk: 0.0,
            total_cost_czk: 0.0,
            net_profit_czk: 0.0,
            grid_export_revenue_czk: 0.0,
            grid_import_cost_czk: 0.0,
            battery_wear_cost_czk: 0.0,
            avoided_import_revenue_czk: 0.0,
        }
    }

    /// Calculate ROI as percentage
    /// ROI = (Net Profit / Total Investment) * 100
    ///
    /// For battery optimization, we consider the battery cost as investment
    pub fn roi_percent(&self, battery_investment_czk: f32) -> f32 {
        if battery_investment_czk == 0.0 {
            return 0.0;
        }
        (self.net_profit_czk / battery_investment_czk) * 100.0
    }

    /// Calculate payback period in years
    /// Assumes daily profit continues at current rate
    pub fn payback_years(&self, battery_investment_czk: f32, duration_days: f32) -> f32 {
        if self.net_profit_czk <= 0.0 || duration_days == 0.0 {
            return f32::INFINITY;
        }

        let daily_profit = self.net_profit_czk / duration_days;
        let annual_profit = daily_profit * 365.0;

        if annual_profit <= 0.0 {
            return f32::INFINITY;
        }

        battery_investment_czk / annual_profit
    }

    /// Get profit margin as percentage
    /// Margin = (Net Profit / Total Revenue) * 100
    pub fn profit_margin_percent(&self) -> f32 {
        if self.total_revenue_czk == 0.0 {
            return 0.0;
        }
        (self.net_profit_czk / self.total_revenue_czk) * 100.0
    }
}

impl Default for FinancialMetrics {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_roi_calculation() {
        let mut metrics = FinancialMetrics::new();
        metrics.net_profit_czk = 5000.0;

        let roi = metrics.roi_percent(50000.0);
        assert_eq!(roi, 10.0); // 10% ROI
    }

    #[test]
    fn test_payback_calculation() {
        let mut metrics = FinancialMetrics::new();
        metrics.net_profit_czk = 1000.0; // 1000 CZK over 30 days

        let payback = metrics.payback_years(50000.0, 30.0);
        // Daily: 1000/30 = 33.33 CZK
        // Annual: 33.33 * 365 = 12166.67 CZK
        // Payback: 50000 / 12166.67 = ~4.11 years
        assert!((payback - 4.11).abs() < 0.1);
    }

    #[test]
    fn test_profit_margin() {
        let mut metrics = FinancialMetrics::new();
        metrics.total_revenue_czk = 10000.0;
        metrics.net_profit_czk = 2000.0;

        assert_eq!(metrics.profit_margin_percent(), 20.0);
    }
}
