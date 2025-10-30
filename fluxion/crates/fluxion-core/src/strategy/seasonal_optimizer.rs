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

// Adaptive Seasonal Optimizer
// Routes evaluation to different strategy sets based on season
// References:
// - analysis/WINTER_STRATEGY_REFINED.md seasonal wrapper

use super::day_ahead_planning::DayAheadChargePlanningStrategy;
use super::morning_precharge::MorningPreChargeStrategy;
use super::seasonal_mode::SeasonalMode;
use super::winter_peak_discharge::WinterPeakDischargeStrategy;
use crate::strategy::{BlockEvaluation, EconomicStrategy, EvaluationContext};
use crate::strategy::{
    PriceArbitrageStrategy, SelfUseStrategy, SolarFirstStrategy, TimeAwareChargeStrategy,
};

/// Strategies configuration for optimizer construction
pub struct SeasonalStrategiesConfig {
    pub winter_peak_discharge_enabled: bool,
    pub winter_peak_min_spread_czk: f32,
    pub winter_peak_min_soc_to_start: f32,
    pub winter_peak_min_soc_target: f32,
    pub winter_peak_min_hours_to_solar: u32,
    pub winter_peak_solar_window_start: u32,
    pub winter_peak_solar_window_end: u32,
    pub solar_aware_charging_enabled: bool,
    pub solar_aware_solar_window_start: u32,
    pub solar_aware_solar_window_end: u32,
    pub solar_aware_midday_max_soc: f32,
    pub solar_aware_min_solar_forecast_kwh: f32,
}

impl Default for SeasonalStrategiesConfig {
    fn default() -> Self {
        Self {
            winter_peak_discharge_enabled: true,
            winter_peak_min_spread_czk: 3.0,
            winter_peak_min_soc_to_start: 70.0,
            winter_peak_min_soc_target: 50.0,
            winter_peak_min_hours_to_solar: 4,
            winter_peak_solar_window_start: 10,
            winter_peak_solar_window_end: 14,
            solar_aware_charging_enabled: true,
            solar_aware_solar_window_start: 10,
            solar_aware_solar_window_end: 14,
            solar_aware_midday_max_soc: 90.0,
            solar_aware_min_solar_forecast_kwh: 2.0,
        }
    }
}

pub struct AdaptiveSeasonalOptimizer {
    winter_strategies: Vec<Box<dyn EconomicStrategy>>, // order matters
    summer_strategies: Vec<Box<dyn EconomicStrategy>>, // order matters
    /// Direct reference to Winter-Peak-Discharge for global planning
    winter_discharge_strategy: std::sync::Arc<WinterPeakDischargeStrategy>,
}

impl AdaptiveSeasonalOptimizer {
    #[must_use]
    pub fn new(
        winter_strategies: Vec<Box<dyn EconomicStrategy>>,
        summer_strategies: Vec<Box<dyn EconomicStrategy>>,
        winter_discharge_strategy: std::sync::Arc<WinterPeakDischargeStrategy>,
    ) -> Self {
        Self {
            winter_strategies,
            summer_strategies,
            winter_discharge_strategy,
        }
    }

    /// Construct optimizer with sensible defaults
    #[must_use]
    pub fn with_defaults() -> Self {
        Self::with_config(&SeasonalStrategiesConfig::default())
    }

    /// Construct optimizer with custom configuration
    #[must_use]
    pub fn with_config(config: &SeasonalStrategiesConfig) -> Self {
        // Create shared Winter-Peak-Discharge strategy for both reference and strategy list
        let winter_discharge = std::sync::Arc::new(WinterPeakDischargeStrategy::new(
            config.winter_peak_discharge_enabled,
            config.winter_peak_min_spread_czk,
            config.winter_peak_min_soc_to_start,
            config.winter_peak_min_soc_target,
            config.winter_peak_min_hours_to_solar,
            config.winter_peak_solar_window_start,
            config.winter_peak_solar_window_end,
        ));

        let winter_strategies: Vec<Box<dyn EconomicStrategy>> = vec![
            // DISABLED: SolarAwareChargingStrategy
            // This was overriding TimeAwareChargeStrategy with more lenient price thresholds (30% above min)
            // causing charging before reaching the actual cheapest blocks.
            // TimeAwareChargeStrategy now handles all charging with "upcoming cheapest blocks" logic.
            // Box::new(SolarAwareChargingStrategy::new(
            //     config.solar_aware_charging_enabled,
            //     config.solar_aware_solar_window_start,
            //     config.solar_aware_solar_window_end,
            //     config.solar_aware_midday_max_soc,
            //     config.solar_aware_min_solar_forecast_kwh,
            // )),
            Box::new(winter_discharge.as_ref().clone()),
            Box::new(MorningPreChargeStrategy::default()),
            Box::new(DayAheadChargePlanningStrategy::default()),
            Box::new(TimeAwareChargeStrategy::default()),
            Box::new(SelfUseStrategy::default()),
        ];

        let summer_strategies: Vec<Box<dyn EconomicStrategy>> = vec![
            Box::new(MorningPreChargeStrategy::default()),
            Box::new(TimeAwareChargeStrategy::default()),
            Box::new(PriceArbitrageStrategy::default()),
            Box::new(SolarFirstStrategy::default()),
            Box::new(SelfUseStrategy::default()),
        ];

        Self {
            winter_strategies,
            summer_strategies,
            winter_discharge_strategy: winter_discharge,
        }
    }

    /// Plan discharge blocks globally before scheduling (for winter season)
    /// This must be called before the main optimization loop
    pub fn plan_discharge_blocks(
        &self,
        all_blocks: &[crate::components::TimeBlockPrice],
        initial_soc: f32,
        control_config: &crate::resources::ControlConfig,
    ) {
        self.winter_discharge_strategy.plan_discharge_blocks(
            all_blocks,
            initial_soc,
            control_config,
        );
    }

    /// Evaluate active season's strategies and pick the best by net_profit_czk
    #[must_use]
    pub fn evaluate(&self, context: &EvaluationContext) -> BlockEvaluation {
        self.evaluate_with_debug(context, false)
    }

    /// Evaluate with optional debug information capture
    #[must_use]
    pub fn evaluate_with_debug(
        &self,
        context: &EvaluationContext,
        capture_debug: bool,
    ) -> BlockEvaluation {
        let season = SeasonalMode::from_date(context.price_block.block_start);
        let strategies: &Vec<Box<dyn EconomicStrategy>> = match season {
            SeasonalMode::Winter => &self.winter_strategies,
            SeasonalMode::Summer => &self.summer_strategies,
        };

        // Evaluate all enabled strategies
        let evaluations: Vec<BlockEvaluation> = strategies
            .iter()
            .filter(|s| s.is_enabled())
            .map(|s| s.evaluate(context))
            .collect();

        // Find the best evaluation by profit
        let mut best = evaluations
            .iter()
            .max_by(|a, b| a.net_profit_czk.partial_cmp(&b.net_profit_czk).unwrap())
            .cloned()
            .unwrap_or_else(|| {
                // Fallback to Self-Use
                use crate::components::InverterOperationMode;
                let mut fallback = BlockEvaluation::new(
                    context.price_block.block_start,
                    context.price_block.duration_minutes,
                    InverterOperationMode::SelfUse,
                    "Fallback".to_string(),
                );
                fallback.reason = "No strategy applicable, using Self-Use fallback".to_string();
                fallback
            });

        // Capture debug info if requested
        if capture_debug && !evaluations.is_empty() {
            use crate::strategy::{BlockDebugInfo, StrategyEvaluation};

            let strategy_evals: Vec<StrategyEvaluation> = evaluations
                .iter()
                .map(|eval| StrategyEvaluation {
                    strategy_name: eval.strategy_name.clone(),
                    mode: eval.mode,
                    net_profit_czk: eval.net_profit_czk,
                    reason: eval.reason.clone(),
                })
                .collect();

            // Build winning reason by comparing to next best
            let mut sorted_evals = evaluations.clone();
            sorted_evals.sort_by(|a, b| b.net_profit_czk.partial_cmp(&a.net_profit_czk).unwrap());

            let winning_reason = if sorted_evals.len() > 1 {
                format!(
                    "{} won with {:.2} CZK profit vs second-best {} with {:.2} CZK",
                    sorted_evals[0].strategy_name,
                    sorted_evals[0].net_profit_czk,
                    sorted_evals[1].strategy_name,
                    sorted_evals[1].net_profit_czk
                )
            } else {
                format!("{} was the only applicable strategy", best.strategy_name)
            };

            // Collect key conditions from evaluation context
            let conditions = vec![
                format!("SOC: {:.1}%", context.current_battery_soc),
                format!(
                    "Price: {:.4} CZK/kWh",
                    context.price_block.price_czk_per_kwh
                ),
                format!("Solar forecast: {:.2} kWh", context.solar_forecast_kwh),
                format!(
                    "Consumption forecast: {:.2} kWh",
                    context.consumption_forecast_kwh
                ),
                format!(
                    "Export price: {:.4} CZK/kWh",
                    context.grid_export_price_czk_per_kwh
                ),
                format!("Season: {season:?}"),
            ];

            best.debug_info = Some(BlockDebugInfo {
                evaluated_strategies: strategy_evals,
                winning_reason,
                conditions,
            });
        }

        best
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::components::TimeBlockPrice;
    use crate::resources::ControlConfig;
    use chrono::Utc;

    fn cfg() -> ControlConfig {
        ControlConfig {
            force_charge_hours: 8,
            force_discharge_hours: 2,
            min_battery_soc: 50.0,
            max_battery_soc: 100.0,
            maximum_export_power_w: 9500,
            battery_capacity_kwh: 23.0,
            battery_wear_cost_czk_per_kwh: 0.125,
            battery_efficiency: 0.95,
            min_mode_change_interval_secs: 300,
            average_household_load_kw: 0.5,
            hardware_min_battery_soc: 10.0,
            max_battery_charge_rate_kw: 10.0,
            evening_target_soc: 90.0,
            evening_peak_start_hour: 17,
            ..Default::default()
        }
    }

    #[test]
    fn test_optimizer_winter_route() {
        let optimizer = AdaptiveSeasonalOptimizer::with_defaults();
        let now = chrono::DateTime::parse_from_rfc3339("2025-10-14T16:00:00Z")
            .unwrap()
            .with_timezone(&Utc);
        let price_block = TimeBlockPrice {
            block_start: now,
            duration_minutes: 15,
            price_czk_per_kwh: 9.53,
        };
        let all = vec![TimeBlockPrice {
            block_start: now,
            duration_minutes: 15,
            price_czk_per_kwh: 2.40,
        }];
        let ctx = EvaluationContext {
            price_block: &price_block,
            control_config: &cfg(),
            current_battery_soc: 100.0,
            solar_forecast_kwh: 0.0,
            consumption_forecast_kwh: 0.5,
            grid_export_price_czk_per_kwh: 9.53,
            all_price_blocks: Some(&all),
        };
        let eval = optimizer.evaluate(&ctx);
        // Should consider winter strategies; we don't assert exact mode, only that evaluation produced
        assert!(!eval.strategy_name.is_empty());
    }
}
