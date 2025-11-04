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

//! Regression tests to ensure strategy performance doesn't degrade
//!
//! These tests establish baseline performance metrics and verify that
//! future changes don't negatively impact strategy efficiency.

use fluxion_strategy_bench::{
    data_sources::{ConsumptionProfile, HistoricalPriceData, HistoricalPricePoint},
    BenchmarkConfig, Simulator,
};
use chrono::{NaiveDate, TimeZone, Utc};

/// Create synthetic price data for consistent testing
fn create_test_price_data() -> HistoricalPriceData {
    let start_date = NaiveDate::from_ymd_opt(2024, 12, 1).unwrap();
    let end_date = NaiveDate::from_ymd_opt(2024, 12, 7).unwrap(); // 1 week

    let mut prices = Vec::new();
    let mut current_date = start_date;

    while current_date <= end_date {
        for hour in 0..24 {
            for minute in [0, 15, 30, 45] {
                let timestamp = Utc
                    .from_local_datetime(&current_date.and_hms_opt(hour, minute, 0).unwrap())
                    .single()
                    .unwrap();

                // Synthetic price pattern:
                // - Night (0-6): 0.30 CZK/kWh (cheap)
                // - Morning (7-9): 0.60 CZK/kWh (expensive)
                // - Day (10-16): 0.40 CZK/kWh (moderate)
                // - Evening (17-22): 0.70 CZK/kWh (expensive)
                // - Late (23): 0.35 CZK/kWh (cheap)
                let price = match hour {
                    0..=6 => 0.30,
                    7..=9 => 0.60,
                    10..=16 => 0.40,
                    17..=22 => 0.70,
                    _ => 0.35,
                };

                prices.push(HistoricalPricePoint {
                    timestamp,
                    price_czk_per_kwh: price,
                    source: "Synthetic".to_string(),
                });
            }
        }

        current_date = current_date.succ_opt().unwrap();
    }

    let start_datetime = Utc
        .from_local_datetime(&start_date.and_hms_opt(0, 0, 0).unwrap())
        .single()
        .unwrap();
    let end_datetime = Utc
        .from_local_datetime(&end_date.and_hms_opt(23, 59, 59).unwrap())
        .single()
        .unwrap();

    HistoricalPriceData {
        prices,
        start_date: start_datetime,
        end_date: end_datetime,
        source: "Synthetic".to_string(),
    }
}

#[test]
fn regression_economic_optimizer_baseline() {
    // This test establishes a baseline for economic optimizer performance
    // on synthetic data. Future changes should not cause this to degrade.

    let price_data = create_test_price_data();
    let consumption_data = ConsumptionProfile::TypicalHousehold.generate(
        NaiveDate::from_ymd_opt(2024, 12, 1).unwrap(),
        NaiveDate::from_ymd_opt(2024, 12, 7).unwrap(),
    );

    let mut simulator = Simulator::new(BenchmarkConfig::default());
    simulator.set_price_data(price_data);
    simulator.set_consumption_data(consumption_data);

    let result = simulator
        .run_economic_optimizer()
        .expect("Failed to run economic optimizer");

    println!("\n📊 Regression Test Results:");
    println!("  Profit: {:.2} CZK", result.metrics.financial.net_profit_czk);
    println!("  Daily avg: {:.2} CZK/day", result.metrics.daily_average_profit());
    println!("  Cycles: {:.1}", result.metrics.battery.total_cycles);
    println!(
        "  Self-consumption: {:.1}%",
        result.metrics.efficiency.self_consumption_rate
    );

    // Baseline assertions (established from initial implementation)
    // These should remain stable or improve

    // Should generate positive profit over a week with clear price patterns
    assert!(
        result.metrics.financial.net_profit_czk > 0.0,
        "Economic optimizer should generate profit"
    );

    // Should have reasonable battery usage (not over-cycling)
    assert!(
        result.metrics.battery.total_cycles < 50.0,
        "Should not over-cycle battery in 7 days"
    );

    // Should achieve decent self-consumption
    assert!(
        result.metrics.efficiency.self_consumption_rate > 40.0,
        "Should achieve >40% self-consumption"
    );

    // Battery SOC should stay within healthy range
    assert!(
        result.metrics.battery.min_soc >= 15.0,
        "Should respect minimum SOC"
    );
    assert!(
        result.metrics.battery.max_soc <= 100.0,
        "Should respect maximum SOC"
    );

    // Financial metrics should be internally consistent
    assert!(
        result.metrics.financial.net_profit_czk
            == result.metrics.financial.total_revenue_czk - result.metrics.financial.total_cost_czk,
        "Profit should equal revenue minus costs"
    );
}

#[test]
fn regression_price_arbitrage_effectiveness() {
    // Price arbitrage should exploit the clear price differences in synthetic data

    let price_data = create_test_price_data();
    let consumption_data = ConsumptionProfile::TypicalHousehold.generate(
        NaiveDate::from_ymd_opt(2024, 12, 1).unwrap(),
        NaiveDate::from_ymd_opt(2024, 12, 7).unwrap(),
    );

    let mut simulator = Simulator::new(BenchmarkConfig::default());
    simulator.set_price_data(price_data);
    simulator.set_consumption_data(consumption_data);

    let result = simulator
        .run_single_strategy("Price-Arbitrage")
        .expect("Failed to run price arbitrage");

    // Price arbitrage should be profitable with clear price patterns
    assert!(
        result.metrics.financial.net_profit_czk > 0.0,
        "Price arbitrage should generate profit"
    );

    // Should have some charge/discharge activity
    assert!(
        result.metrics.battery.total_charge_kwh > 0.0,
        "Should charge battery"
    );
    assert!(
        result.metrics.battery.total_discharge_kwh > 0.0,
        "Should discharge battery"
    );

    println!("\n💰 Price Arbitrage Regression:");
    println!("  Profit: {:.2} CZK", result.metrics.financial.net_profit_czk);
    println!("  Charge: {:.2} kWh", result.metrics.battery.total_charge_kwh);
    println!(
        "  Discharge: {:.2} kWh",
        result.metrics.battery.total_discharge_kwh
    );
}

#[test]
fn regression_self_use_baseline() {
    // Self-use strategy should provide baseline performance

    let price_data = create_test_price_data();
    let consumption_data = ConsumptionProfile::TypicalHousehold.generate(
        NaiveDate::from_ymd_opt(2024, 12, 1).unwrap(),
        NaiveDate::from_ymd_opt(2024, 12, 7).unwrap(),
    );

    let mut simulator = Simulator::new(BenchmarkConfig::default());
    simulator.set_price_data(price_data);
    simulator.set_consumption_data(consumption_data);

    let result = simulator
        .run_baseline()
        .expect("Failed to run baseline");

    // Self-use should work reliably
    assert!(
        result.metrics.total_blocks > 0,
        "Should simulate blocks"
    );

    // Should have reasonable efficiency
    assert!(
        result.metrics.efficiency.self_consumption_rate > 30.0,
        "Self-use should achieve >30% self-consumption"
    );

    println!("\n🏠 Self-Use Baseline Regression:");
    println!("  Profit: {:.2} CZK", result.metrics.financial.net_profit_czk);
    println!(
        "  Self-consumption: {:.1}%",
        result.metrics.efficiency.self_consumption_rate
    );
    println!(
        "  Autarky: {:.1}%",
        result.metrics.efficiency.autarky_rate
    );
}

#[test]
fn regression_optimizer_beats_baseline() {
    // Economic optimizer should outperform simple self-use

    let price_data = create_test_price_data();
    let consumption_data = ConsumptionProfile::TypicalHousehold.generate(
        NaiveDate::from_ymd_opt(2024, 12, 1).unwrap(),
        NaiveDate::from_ymd_opt(2024, 12, 7).unwrap(),
    );

    let mut simulator = Simulator::new(BenchmarkConfig::default());
    simulator.set_price_data(price_data.clone());
    simulator.set_consumption_data(consumption_data.clone());

    let baseline = simulator.run_baseline().expect("Failed to run baseline");

    let mut simulator2 = Simulator::new(BenchmarkConfig::default());
    simulator2.set_price_data(price_data);
    simulator2.set_consumption_data(consumption_data);

    let optimizer = simulator2
        .run_economic_optimizer()
        .expect("Failed to run optimizer");

    println!("\n⚔️  Optimizer vs Baseline:");
    println!(
        "  Baseline:  {:.2} CZK",
        baseline.metrics.financial.net_profit_czk
    );
    println!(
        "  Optimizer: {:.2} CZK",
        optimizer.metrics.financial.net_profit_czk
    );
    println!(
        "  Improvement: {:.1}%",
        optimizer.metrics.improvement_vs_baseline(&baseline.metrics)
    );

    // Economic optimizer should perform at least as well as baseline
    assert!(
        optimizer.metrics.financial.net_profit_czk >= baseline.metrics.financial.net_profit_czk,
        "Economic optimizer should not be worse than baseline"
    );
}

#[test]
fn regression_battery_health_limits() {
    // Verify all strategies respect battery health limits

    let price_data = create_test_price_data();
    let consumption_data = ConsumptionProfile::TypicalHousehold.generate(
        NaiveDate::from_ymd_opt(2024, 12, 1).unwrap(),
        NaiveDate::from_ymd_opt(2024, 12, 7).unwrap(),
    );

    let strategies = vec![
        "Price-Arbitrage",
        "Solar-First",
        "Time-Aware-Charge",
        "Morning-PreCharge",
        "Day-Ahead-Planning",
        "Self-Use",
    ];

    for strategy_name in strategies {
        let mut simulator = Simulator::new(BenchmarkConfig::default());
        simulator.set_price_data(price_data.clone());
        simulator.set_consumption_data(consumption_data.clone());

        let result = simulator
            .run_single_strategy(strategy_name)
            .expect(&format!("Failed to run {}", strategy_name));

        // All strategies should respect battery limits
        assert!(
            result.metrics.battery.min_soc >= 15.0,
            "{} should respect min SOC",
            strategy_name
        );

        assert!(
            result.metrics.battery.max_soc <= 100.0,
            "{} should respect max SOC",
            strategy_name
        );

        // Should not over-cycle battery
        assert!(
            result.metrics.battery.total_cycles < 100.0,
            "{} should not over-cycle battery ({})",
            strategy_name,
            result.metrics.battery.total_cycles
        );

        println!(
            "✓ {} respects battery limits (SOC: {:.1}%-{:.1}%, Cycles: {:.1})",
            strategy_name,
            result.metrics.battery.min_soc,
            result.metrics.battery.max_soc,
            result.metrics.battery.total_cycles
        );
    }
}
