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

//! Custom benchmark tests for user-provided Excel consumption data
//!
//! This test file is designed to work with real consumption data from Excel files.
//!
//! Usage:
//! ```bash
//! # With both price and consumption data
//! PRICE_DATA_CSV=/path/to/prices.csv \
//! CONSUMPTION_EXCEL=/path/to/consumption.xlsx \
//! cargo test -p fluxion-strategy-bench --test real_data_benchmark -- --ignored --nocapture
//!
//! # With consumption data and synthetic prices
//! CONSUMPTION_EXCEL=/path/to/consumption.xlsx \
//! cargo test -p fluxion-strategy-bench --test real_data_benchmark -- --ignored --nocapture
//! ```
//!
//! Excel File Format:
//! - Column A: Timestamp (any common format)
//! - Column B: Consumption in kW
//! - First row: Headers (optional, will be skipped)

use fluxion_strategy_bench::{
    data_sources::{load_from_excel, load_from_csv, PriceDataCache},
    reports::{BenchmarkReport, ReportFormat},
    BenchmarkConfig, Simulator,
};
use chrono::NaiveDate;
use std::env;

/// Helper to load consumption data from environment or use synthetic
fn load_consumption_data() -> Result<fluxion_strategy_bench::data_sources::ConsumptionData, Box<dyn std::error::Error>> {
    // Check for Excel file in environment
    if let Ok(excel_path) = env::var("CONSUMPTION_EXCEL") {
        println!("📊 Loading consumption data from Excel: {}", excel_path);
        let data = load_from_excel(&excel_path)?;
        println!("   ✓ Loaded {} consumption points", data.points.len());
        println!("   ✓ Date range: {} to {}",
            data.points.first().unwrap().timestamp,
            data.points.last().unwrap().timestamp
        );
        return Ok(data);
    }

    // Check for CSV file in environment
    if let Ok(csv_path) = env::var("CONSUMPTION_CSV") {
        println!("📊 Loading consumption data from CSV: {}", csv_path);
        let data = load_from_csv(&csv_path)?;
        println!("   ✓ Loaded {} consumption points", data.points.len());
        return Ok(data);
    }

    // Fall back to synthetic data
    println!("⚠️  No CONSUMPTION_EXCEL or CONSUMPTION_CSV provided, using synthetic data");
    println!("   Set CONSUMPTION_EXCEL=/path/to/file.xlsx to use your real data");

    use fluxion_strategy_bench::data_sources::ConsumptionProfile;
    let data = ConsumptionProfile::TypicalHousehold.generate(
        NaiveDate::from_ymd_opt(2024, 12, 1).unwrap(),
        NaiveDate::from_ymd_opt(2024, 12, 31).unwrap(),
    );

    Ok(data)
}

#[test]
#[ignore] // Run with --ignored flag
fn benchmark_with_real_consumption_data() {
    tracing_subscriber::fmt::init();

    println!("\n🚀 Running COMPREHENSIVE Strategy Benchmark with Real Data");
    println!("═══════════════════════════════════════════════════════════════");
    println!("Testing ALL strategies with EQUAL priority:\n");
    println!("  1. Price Arbitrage       - Exploit price spreads");
    println!("  2. Solar-First           - Maximize self-consumption");
    println!("  3. Time-Aware Charge     - Prepare for expensive periods");
    println!("  4. Morning Pre-Charge    - Morning peak preparation");
    println!("  5. Day-Ahead Planning    - 24-35 hour optimization");
    println!("  6. Winter Peak Discharge - Conservative winter operation");
    println!("  7. Self-Use (baseline)   - Standard operation");
    println!("  8. Economic Optimizer    - Dynamic strategy selection");
    println!("\n═══════════════════════════════════════════════════════════════\n");

    // Load consumption data (from Excel or synthetic)
    let consumption_data = load_consumption_data()
        .expect("Failed to load consumption data");

    // Determine date range from consumption data
    let start_date = consumption_data.points.first().unwrap().timestamp.date_naive();
    let end_date = consumption_data.points.last().unwrap().timestamp.date_naive();

    println!("\n📅 Analysis Period: {} to {}", start_date, end_date);
    println!("   Duration: {} days\n", (end_date - start_date).num_days() + 1);

    // Load price data
    let mut price_cache = PriceDataCache::new().expect("Failed to create price cache");

    if let Ok(csv_path) = env::var("PRICE_DATA_CSV") {
        println!("📂 Using manual CSV price data: {}", csv_path);
        price_cache.set_manual_csv(csv_path.into());
    }

    let price_data = price_cache
        .get_price_data(start_date, end_date)
        .expect("Failed to load price data");

    println!("✓ Loaded {} price points", price_data.prices.len());

    // Display price statistics
    let price_stats = price_data.statistics();
    println!("\n💰 Price Statistics:");
    println!("   Mean:   {:.3} CZK/kWh", price_stats.mean);
    println!("   Median: {:.3} CZK/kWh", price_stats.median);
    println!("   Min:    {:.3} CZK/kWh", price_stats.min);
    println!("   Max:    {:.3} CZK/kWh", price_stats.max);
    println!("   StdDev: {:.3} CZK/kWh", price_stats.std_dev);

    // Display consumption statistics
    let avg_consumption: f32 = consumption_data.points.iter()
        .map(|p| p.consumption_kw)
        .sum::<f32>() / consumption_data.points.len() as f32;
    let max_consumption = consumption_data.points.iter()
        .map(|p| p.consumption_kw)
        .fold(0.0f32, f32::max);

    println!("\n⚡ Consumption Statistics:");
    println!("   Average: {:.3} kW", avg_consumption);
    println!("   Maximum: {:.3} kW", max_consumption);
    println!("   Total:   {:.1} kWh", consumption_data.points.iter().map(|p| p.consumption_kw * 0.25).sum::<f32>());

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
        average_household_load_kw: avg_consumption, // Use actual average
        evening_peak_start_hour: 17,
        evening_target_soc: 90.0,
    };

    println!("\n🔋 System Configuration:");
    println!("   Battery:      {} kWh", config.battery_capacity_kwh);
    println!("   Solar:        {} kW peak", config.solar_config.peak_capacity_kw);
    println!("   Efficiency:   {:.1}%", config.battery_efficiency * 100.0);
    println!("   Charge rate:  {} kW", config.max_battery_charge_rate_kw);
    println!("   SOC range:    {:.0}% - {:.0}%", config.min_battery_soc, config.max_battery_soc);

    let mut simulator = Simulator::new(config.clone());
    simulator.set_price_data(price_data.clone());
    simulator.set_consumption_data(consumption_data.clone());

    println!("\n📊 Running simulations for ALL strategies...\n");
    println!("   This will test each strategy individually AND the combined optimizer");
    println!("   Expected duration: 2-5 minutes depending on data size\n");

    // Run all strategies with equal priority
    let results = simulator
        .run_all_strategies_individually()
        .expect("Failed to run strategies");

    println!("\n✓ Completed {} strategy simulations\n", results.len());

    // Generate comprehensive report
    let mut report = BenchmarkReport::new(
        results,
        format!(
            "FluxION Comprehensive Strategy Benchmark - {} to {}",
            start_date, end_date
        ),
    );

    report.add_note(format!("Real consumption data: {} points", consumption_data.points.len()));
    report.add_note(format!("Price data: {} points", price_data.prices.len()));
    report.add_note(format!("System: {}kWh battery, {}kW solar",
        config.battery_capacity_kwh,
        config.solar_config.peak_capacity_kw
    ));
    report.add_note(format!("Analysis period: {} days", (end_date - start_date).num_days() + 1));
    report.add_note("All strategies tested with equal priority".to_string());

    // Print summary to console
    report.print_summary();

    // Export detailed reports
    let markdown_filename = format!("benchmark_real_data_{}_{}.md",
        start_date.format("%Y%m%d"),
        end_date.format("%Y%m%d")
    );
    let json_filename = format!("benchmark_real_data_{}_{}.json",
        start_date.format("%Y%m%d"),
        end_date.format("%Y%m%d")
    );

    report
        .export_to_file(&markdown_filename, ReportFormat::Markdown)
        .expect("Failed to export markdown report");

    report
        .export_to_file(&json_filename, ReportFormat::Json)
        .expect("Failed to export JSON report");

    println!("\n📄 Detailed reports exported:");
    println!("   - {}", markdown_filename);
    println!("   - {}", json_filename);

    // Detailed per-strategy analysis
    println!("\n📊 DETAILED STRATEGY ANALYSIS");
    println!("═══════════════════════════════════════════════════════════════\n");

    for result in report.sorted_by_profit() {
        println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
        println!("Strategy: {}", result.strategy_name);
        println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");

        // Financial
        println!("\n💰 Financial Performance:");
        println!("   Net Profit:        {:>10.2} CZK", result.metrics.financial.net_profit_czk);
        println!("   Daily Average:     {:>10.2} CZK/day", result.metrics.daily_average_profit());
        println!("   Total Revenue:     {:>10.2} CZK", result.metrics.financial.total_revenue_czk);
        println!("   Total Cost:        {:>10.2} CZK", result.metrics.financial.total_cost_czk);
        println!("   Grid Import Cost:  {:>10.2} CZK", result.metrics.financial.grid_import_cost_czk);
        println!("   Grid Export Rev:   {:>10.2} CZK", result.metrics.financial.grid_export_revenue_czk);
        println!("   Battery Wear Cost: {:>10.2} CZK", result.metrics.financial.battery_wear_cost_czk);
        println!("   Profit Margin:     {:>10.1}%", result.metrics.financial.profit_margin_percent());

        // Battery Health
        println!("\n🔋 Battery Health:");
        println!("   Total Cycles:      {:>10.1}", result.metrics.battery.total_cycles);
        println!("   Average DoD:       {:>10.1}%", result.metrics.battery.average_dod);
        println!("   SOC Range:         {:>10.1}% - {:.1}%",
            result.metrics.battery.min_soc,
            result.metrics.battery.max_soc
        );
        println!("   Total Charged:     {:>10.1} kWh", result.metrics.battery.total_charge_kwh);
        println!("   Total Discharged:  {:>10.1} kWh", result.metrics.battery.total_discharge_kwh);
        println!("   Efficiency:        {:>10.1}%", result.metrics.battery.round_trip_efficiency_percent());

        // Efficiency
        println!("\n⚡ Efficiency Metrics:");
        println!("   Self-Consumption:  {:>10.1}%", result.metrics.efficiency.self_consumption_rate);
        println!("   Autarky Rate:      {:>10.1}%", result.metrics.efficiency.autarky_rate);
        println!("   Grid Independence: {:>10.1}%", result.metrics.efficiency.grid_independence_score());
        println!("   Solar Generated:   {:>10.1} kWh", result.metrics.efficiency.total_solar_generation_kwh);
        println!("   Grid Import:       {:>10.1} kWh", result.metrics.efficiency.total_grid_import_kwh);
        println!("   Grid Export:       {:>10.1} kWh", result.metrics.efficiency.total_grid_export_kwh);
        println!("   Efficiency Grade:  {:>10}", result.metrics.efficiency.efficiency_grade());

        // Strategy-specific stats (if available)
        if !result.metrics.strategy_stats.is_empty() {
            println!("\n📈 Strategy Contribution:");
            for stat in &result.metrics.strategy_stats {
                if stat.total_blocks > 0 {
                    println!("   {:<20} Win Rate: {:>6.1}%  Profit: {:>8.2} CZK  Profit Rate: {:>5.1}%",
                        stat.strategy_name,
                        stat.win_rate_percent,
                        stat.total_profit_czk,
                        stat.profit_rate_percent
                    );
                }
            }
        }

        // Health warnings
        let warnings = result.metrics.battery.health_warnings();
        if !warnings.is_empty() {
            println!("\n⚠️  Health Warnings:");
            for warning in warnings {
                println!("   ⚠ {}", warning);
            }
        }

        println!();
    }

    // Comparative analysis
    if let Some(baseline) = report.get_baseline() {
        println!("\n📊 COMPARATIVE ANALYSIS");
        println!("═══════════════════════════════════════════════════════════════\n");
        println!("Baseline (Self-Use only): {:.2} CZK\n", baseline.metrics.financial.net_profit_czk);

        for result in report.sorted_by_profit() {
            if result.strategy_name != baseline.strategy_name {
                let improvement = result.metrics.improvement_vs_baseline(&baseline.metrics);
                let symbol = if improvement > 0.0 { "▲" } else { "▼" };
                let color = if improvement > 0.0 { "✓" } else { "✗" };

                println!("{} {:<25} {:>8.2} CZK  {} {:>6.1}% vs baseline",
                    color,
                    result.strategy_name,
                    result.metrics.financial.net_profit_czk,
                    symbol,
                    improvement
                );
            }
        }
    }

    // Key insights
    println!("\n💡 KEY INSIGHTS");
    println!("═══════════════════════════════════════════════════════════════\n");

    if let Some(best) = report.sorted_by_profit().first() {
        println!("✓ Best Performer: {} ({:.2} CZK total profit)",
            best.strategy_name,
            best.metrics.financial.net_profit_czk
        );

        if let Some(baseline) = report.get_baseline() {
            let improvement = best.metrics.improvement_vs_baseline(&baseline.metrics);
            println!("✓ Improvement over baseline: {:.1}%", improvement);
        }

        println!("✓ Battery Usage: {:.1} cycles ({} intensity)",
            best.metrics.battery.total_cycles,
            if best.metrics.battery.total_cycles < 30.0 { "Conservative" }
            else if best.metrics.battery.total_cycles < 50.0 { "Moderate" }
            else { "Aggressive" }
        );

        println!("✓ Self-Consumption: {:.1}% ({})",
            best.metrics.efficiency.self_consumption_rate,
            if best.metrics.efficiency.self_consumption_rate > 80.0 { "Excellent" }
            else if best.metrics.efficiency.self_consumption_rate > 60.0 { "Good" }
            else { "Room for improvement" }
        );

        println!("✓ Grid Independence: {:.1}% autarky",
            best.metrics.efficiency.autarky_rate
        );

        println!("✓ Overall Grade: {}", best.metrics.efficiency.efficiency_grade());
    }

    println!("\n✓ All {} strategies tested with equal priority", report.results.len());
    println!("✓ Full analysis complete!\n");

    // Assertions
    if let Some(baseline) = report.get_baseline() {
        assert!(
            baseline.metrics.financial.net_profit_czk.abs() > 0.0,
            "Baseline should have non-zero profit"
        );

        if let Some(optimizer) = report.get_optimizer() {
            assert!(
                optimizer.metrics.financial.net_profit_czk >= baseline.metrics.financial.net_profit_czk,
                "Economic optimizer should not perform worse than baseline"
            );
        }
    }
}

#[test]
fn test_excel_loading_instructions() {
    // This test just prints helpful instructions
    println!("\n━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!("📋 HOW TO USE YOUR EXCEL CONSUMPTION DATA");
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n");

    println!("1. Prepare your Excel file with 2 columns:");
    println!("   ┌─────────────────────┬──────────────┐");
    println!("   │ Timestamp           │ Consumption  │");
    println!("   ├─────────────────────┼──────────────┤");
    println!("   │ 2024-12-01 00:00:00 │ 0.45         │");
    println!("   │ 2024-12-01 00:15:00 │ 0.43         │");
    println!("   │ 2024-12-01 00:30:00 │ 0.52         │");
    println!("   │ ...                 │ ...          │");
    println!("   └─────────────────────┴──────────────┘\n");

    println!("2. Run the benchmark with your data:");
    println!("   CONSUMPTION_EXCEL=/path/to/your_file.xlsx \\");
    println!("   PRICE_DATA_CSV=/path/to/prices.csv \\");
    println!("   cargo test -p fluxion-strategy-bench \\");
    println!("     --test real_data_benchmark \\");
    println!("     benchmark_with_real_consumption_data \\");
    println!("     -- --ignored --nocapture\n");

    println!("3. Supported timestamp formats:");
    println!("   ✓ 2024-12-01T00:00:00Z     (ISO 8601)");
    println!("   ✓ 2024-12-01 00:00:00      (Standard)");
    println!("   ✓ 01.12.2024 00:00         (European)");
    println!("   ✓ 12/01/2024 00:00         (US)");
    println!("   ✓ Excel date cells         (Native)\n");

    println!("4. Consumption units: kW (kilowatts)\n");

    println!("5. CSV format also supported:");
    println!("   CONSUMPTION_CSV=/path/to/consumption.csv\n");

    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n");
}
