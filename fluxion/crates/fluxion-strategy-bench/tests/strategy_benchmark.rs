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

//! Comprehensive strategy benchmarking tests
//!
//! These tests run ALL strategies on historical electricity price data
//! and real consumption profiles to analyze performance.
//!
//! Run with: `cargo test --package fluxion-strategy-bench --test strategy_benchmark -- --ignored --test-threads=1`
//!
//! Note: Tests are marked #[ignore] by default because they:
//! - Fetch real historical price data from OTE-CR
//! - Take several minutes to complete
//! - Generate detailed reports
//!
//! For manual CSV data, set environment variable:
//! `PRICE_DATA_CSV=/path/to/prices.csv cargo test ...`

use fluxion_strategy_bench::{
    data_sources::{ConsumptionProfile, PriceDataCache},
    reports::{BenchmarkReport, ReportFormat},
    scenarios::predefined,
    BenchmarkConfig, Simulator,
};
use chrono::NaiveDate;
use std::env;

/// Helper to create price cache with optional manual CSV
fn create_price_cache() -> PriceDataCache {
    let mut cache = PriceDataCache::new().expect("Failed to create price cache");

    // Check for manual CSV path in environment
    if let Ok(csv_path) = env::var("PRICE_DATA_CSV") {
        println!("📂 Using manual CSV file: {}", csv_path);
        cache.set_manual_csv(csv_path.into());
    }

    cache
}

#[test]
#[ignore] // Run explicitly with --ignored flag
fn benchmark_all_strategies_december_2024() {
    tracing_subscriber::fmt::init();

    println!("\n🚀 Running comprehensive strategy benchmark for December 2024");
    println!("═══════════════════════════════════════════════════════════════\n");

    let mut price_cache = create_price_cache();

    // Date range: December 2024
    let start_date = NaiveDate::from_ymd_opt(2024, 12, 1).unwrap();
    let end_date = NaiveDate::from_ymd_opt(2024, 12, 31).unwrap();

    // Load price data
    let price_data = price_cache
        .get_price_data(start_date, end_date)
        .expect("Failed to load price data");

    println!("✓ Loaded {} price points", price_data.prices.len());

    // Generate consumption data (typical household)
    let consumption_data = ConsumptionProfile::TypicalHousehold.generate(start_date, end_date);
    println!("✓ Generated {} consumption points", consumption_data.points.len());

    // Configuration
    let config = BenchmarkConfig {
        battery_capacity_kwh: 20.0,
        initial_soc_percent: 50.0,
        min_battery_soc: 15.0,
        max_battery_soc: 100.0,
        battery_efficiency: 0.95,
        battery_wear_cost_czk_per_kwh: 0.125,
        max_battery_charge_rate_kw: 5.0,
        max_battery_discharge_rate_kw: 5.0,
        solar_config: fluxion_strategy_bench::simulator::SolarConfig {
            peak_capacity_kw: 10.0,
            use_seasonal_profile: true,
        },
        average_household_load_kw: 0.5,
        evening_peak_start_hour: 17,
        evening_target_soc: 90.0,
    };

    let mut simulator = Simulator::new(config);
    simulator.set_price_data(price_data.clone());
    simulator.set_consumption_data(consumption_data.clone());

    println!("\n📊 Running simulations...\n");

    // Run all strategies
    let results = simulator
        .run_all_strategies_individually()
        .expect("Failed to run strategies");

    println!("\n✓ Completed {} strategy simulations", results.len());

    // Generate report
    let mut report = BenchmarkReport::new(
        results,
        "FluxION Strategy Benchmark - December 2024",
    );

    report.add_note("Simulated on typical household consumption profile");
    report.add_note("20kWh battery, 10kW solar array");
    report.add_note("Czech electricity market (OTE-CR) prices");

    // Print summary to console
    report.print_summary();

    // Export detailed reports
    report
        .export_to_file("benchmark_december_2024.md", ReportFormat::Markdown)
        .expect("Failed to export markdown report");

    report
        .export_to_file("benchmark_december_2024.json", ReportFormat::Json)
        .expect("Failed to export JSON report");

    println!("\n✓ Reports exported:");
    println!("  - benchmark_december_2024.md");
    println!("  - benchmark_december_2024.json");

    // Assertions to ensure meaningful results
    if let Some(baseline) = report.get_baseline() {
        assert!(
            baseline.metrics.financial.net_profit_czk.abs() > 0.0,
            "Baseline should have non-zero profit"
        );

        if let Some(optimizer) = report.get_optimizer() {
            let improvement = optimizer.metrics.improvement_vs_baseline(&baseline.metrics);
            println!("\n✨ Economic Optimizer improvement: {:.1}%", improvement);

            // Economic optimizer should perform better than or equal to baseline
            assert!(
                optimizer.metrics.financial.net_profit_czk >= baseline.metrics.financial.net_profit_czk,
                "Economic optimizer should not perform worse than baseline"
            );
        }
    }
}

#[test]
#[ignore]
fn benchmark_price_arbitrage_only() {
    tracing_subscriber::fmt::init();

    println!("\n🎯 Testing Price Arbitrage Strategy");

    let mut price_cache = create_price_cache();

    let start_date = NaiveDate::from_ymd_opt(2024, 12, 1).unwrap();
    let end_date = NaiveDate::from_ymd_opt(2024, 12, 31).unwrap();

    let price_data = price_cache
        .get_price_data(start_date, end_date)
        .expect("Failed to load price data");

    let consumption_data = ConsumptionProfile::TypicalHousehold.generate(start_date, end_date);

    let mut simulator = Simulator::new(BenchmarkConfig::default());
    simulator.set_price_data(price_data.clone());
    simulator.set_consumption_data(consumption_data);

    let result = simulator
        .run_single_strategy("Price-Arbitrage")
        .expect("Failed to run price arbitrage");

    println!("\n📈 Price Arbitrage Results:");
    println!("  Profit: {:.2} CZK", result.metrics.financial.net_profit_czk);
    println!("  Cycles: {:.1}", result.metrics.battery.total_cycles);
    println!(
        "  Self-Consumption: {:.1}%",
        result.metrics.efficiency.self_consumption_rate
    );

    // Price arbitrage should generate some profit in a month
    assert!(result.metrics.financial.net_profit_czk > 0.0);

    // Should have reasonable battery usage
    assert!(result.metrics.battery.total_cycles > 0.0);
    assert!(result.metrics.battery.total_cycles < 200.0); // Shouldn't over-cycle
}

#[test]
#[ignore]
fn benchmark_solar_first_only() {
    tracing_subscriber::fmt::init();

    println!("\n☀️ Testing Solar-First Strategy");

    let mut price_cache = create_price_cache();

    let start_date = NaiveDate::from_ymd_opt(2024, 12, 1).unwrap();
    let end_date = NaiveDate::from_ymd_opt(2024, 12, 31).unwrap();

    let price_data = price_cache
        .get_price_data(start_date, end_date)
        .expect("Failed to load price data");

    let consumption_data = ConsumptionProfile::TypicalHousehold.generate(start_date, end_date);

    let mut simulator = Simulator::new(BenchmarkConfig::default());
    simulator.set_price_data(price_data.clone());
    simulator.set_consumption_data(consumption_data);

    let result = simulator
        .run_single_strategy("Solar-First")
        .expect("Failed to run solar-first");

    println!("\n📈 Solar-First Results:");
    println!("  Profit: {:.2} CZK", result.metrics.financial.net_profit_czk);
    println!(
        "  Self-Consumption: {:.1}%",
        result.metrics.efficiency.self_consumption_rate
    );
    println!(
        "  Autarky: {:.1}%",
        result.metrics.efficiency.autarky_rate
    );

    // Solar-first should have high self-consumption
    assert!(result.metrics.efficiency.self_consumption_rate > 50.0);
}

#[test]
#[ignore]
fn benchmark_predefined_scenarios() {
    tracing_subscriber::fmt::init();

    println!("\n🎭 Running predefined scenario tests");

    let mut price_cache = create_price_cache();

    for scenario in predefined::all() {
        println!("\n━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
        println!("📋 Scenario: {}", scenario.name);
        println!("   {}", scenario.description);

        let simulator = scenario
            .run(&mut price_cache)
            .expect("Failed to create simulator from scenario");

        match simulator.run_economic_optimizer() {
            Ok(result) => {
                println!("   ✓ Profit: {:.2} CZK", result.metrics.financial.net_profit_czk);
                println!("   ✓ Cycles: {:.1}", result.metrics.battery.total_cycles);
                println!(
                    "   ✓ Grade: {}",
                    result.metrics.efficiency.efficiency_grade()
                );

                // Check expected characteristics
                for expectation in &scenario.expected_characteristics {
                    println!("   📝 Expectation: {}", expectation);
                }
            }
            Err(e) => {
                println!("   ✗ Failed: {}", e);
            }
        }
    }

    println!("\n✓ Completed all predefined scenarios");
}

#[test]
#[ignore]
fn benchmark_compare_configurations() {
    tracing_subscriber::fmt::init();

    println!("\n⚙️  Comparing different system configurations");

    let mut price_cache = create_price_cache();

    let start_date = NaiveDate::from_ymd_opt(2024, 12, 1).unwrap();
    let end_date = NaiveDate::from_ymd_opt(2024, 12, 31).unwrap();

    let price_data = price_cache
        .get_price_data(start_date, end_date)
        .expect("Failed to load price data");

    let consumption_data = ConsumptionProfile::TypicalHousehold.generate(start_date, end_date);

    let configurations = vec![
        ("10kWh Battery, 5kW Solar", 10.0, 5.0),
        ("20kWh Battery, 10kW Solar", 20.0, 10.0),
        ("30kWh Battery, 15kW Solar", 30.0, 15.0),
    ];

    let mut results = Vec::new();

    for (name, battery_kwh, solar_kw) in configurations {
        println!("\n🔧 Testing: {}", name);

        let config = BenchmarkConfig {
            battery_capacity_kwh: battery_kwh,
            solar_config: fluxion_strategy_bench::simulator::SolarConfig {
                peak_capacity_kw: solar_kw,
                use_seasonal_profile: true,
            },
            ..BenchmarkConfig::default()
        };

        let mut simulator = Simulator::new(config);
        simulator.set_price_data(price_data.clone());
        simulator.set_consumption_data(consumption_data.clone());

        match simulator.run_economic_optimizer() {
            Ok(result) => {
                println!("   Profit: {:.2} CZK", result.metrics.financial.net_profit_czk);
                println!(
                    "   Profit/kWh battery: {:.2} CZK",
                    result.metrics.financial.net_profit_czk / battery_kwh
                );
                results.push((name, result));
            }
            Err(e) => {
                println!("   Failed: {}", e);
            }
        }
    }

    // Larger systems should generally have higher absolute profit
    println!("\n📊 Configuration comparison complete");
}

#[test]
fn test_simulator_basic_functionality() {
    // This test runs without real data to verify basic simulator functionality
    let config = BenchmarkConfig::default();
    let simulator = Simulator::new(config);

    // Just verify we can create a simulator
    assert_eq!(
        simulator
            .run_baseline()
            .unwrap_err()
            .to_string()
            .contains("Price data not loaded"),
        true
    );
}
