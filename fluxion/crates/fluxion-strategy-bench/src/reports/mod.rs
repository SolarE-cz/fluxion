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

//! Report generation for benchmark results

use crate::simulator::SimulationResult;
use anyhow::Result;
use comfy_table::{presets::UTF8_FULL, Attribute, Cell, Color, Table};
use serde::{Deserialize, Serialize};
use std::fs::File;
use std::io::Write;
use std::path::Path;

/// Report format options
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ReportFormat {
    /// Markdown format
    Markdown,
    /// JSON format
    Json,
    /// Plain text with tables
    Text,
}

/// Comprehensive benchmark report
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BenchmarkReport {
    /// All simulation results
    pub results: Vec<SimulationResult>,

    /// Report generation timestamp
    pub generated_at: chrono::DateTime<chrono::Utc>,

    /// Title of the report
    pub title: String,

    /// Additional notes or comments
    pub notes: Vec<String>,
}

impl BenchmarkReport {
    /// Create a new benchmark report
    pub fn new(results: Vec<SimulationResult>, title: impl Into<String>) -> Self {
        Self {
            results,
            generated_at: chrono::Utc::now(),
            title: title.into(),
            notes: Vec::new(),
        }
    }

    /// Add a note to the report
    pub fn add_note(&mut self, note: impl Into<String>) {
        self.notes.push(note.into());
    }

    /// Get baseline result (Self-Use strategy)
    pub fn get_baseline(&self) -> Option<&SimulationResult> {
        self.results.iter().find(|r| r.strategy_name.contains("Baseline") || r.strategy_name.contains("Self-Use"))
    }

    /// Get economic optimizer result
    pub fn get_optimizer(&self) -> Option<&SimulationResult> {
        self.results.iter().find(|r| r.strategy_name.contains("Economic-Optimizer"))
    }

    /// Sort results by profit (descending)
    pub fn sorted_by_profit(&self) -> Vec<&SimulationResult> {
        let mut sorted: Vec<&SimulationResult> = self.results.iter().collect();
        sorted.sort_by(|a, b| {
            b.metrics
                .financial
                .net_profit_czk
                .partial_cmp(&a.metrics.financial.net_profit_czk)
                .unwrap()
        });
        sorted
    }

    /// Generate report in specified format
    pub fn generate(&self, format: ReportFormat) -> String {
        match format {
            ReportFormat::Markdown => self.generate_markdown(),
            ReportFormat::Json => self.generate_json(),
            ReportFormat::Text => self.generate_text(),
        }
    }

    /// Export report to file
    pub fn export_to_file<P: AsRef<Path>>(&self, path: P, format: ReportFormat) -> Result<()> {
        let content = self.generate(format);
        let mut file = File::create(path)?;
        file.write_all(content.as_bytes())?;
        Ok(())
    }

    /// Print summary to console
    pub fn print_summary(&self) {
        println!("\n{}", self.generate_text());
    }

    /// Generate markdown report
    fn generate_markdown(&self) -> String {
        let mut output = String::new();

        // Title
        output.push_str(&format!("# {}\n\n", self.title));
        output.push_str(&format!("**Generated:** {}\n\n", self.generated_at.format("%Y-%m-%d %H:%M:%S UTC")));

        // Summary
        if let Some(baseline) = self.get_baseline() {
            if let Some(optimizer) = self.get_optimizer() {
                let improvement = optimizer.metrics.improvement_vs_baseline(&baseline.metrics);
                output.push_str("## Executive Summary\n\n");
                output.push_str(&format!(
                    "- **Baseline Profit:** {:.2} CZK\n",
                    baseline.metrics.financial.net_profit_czk
                ));
                output.push_str(&format!(
                    "- **Optimized Profit:** {:.2} CZK\n",
                    optimizer.metrics.financial.net_profit_czk
                ));
                output.push_str(&format!(
                    "- **Improvement:** {:.1}%\n",
                    improvement
                ));
                output.push_str(&format!(
                    "- **Duration:** {:.1} days\n\n",
                    baseline.metrics.duration_days
                ));
            }
        }

        // Strategy Rankings Table
        output.push_str("## Strategy Performance Rankings\n\n");
        output.push_str("| Rank | Strategy | Net Profit | Cycles | Self-Consumption | Efficiency Grade |\n");
        output.push_str("|------|----------|------------|--------|------------------|------------------|\n");

        for (idx, result) in self.sorted_by_profit().iter().enumerate() {
            output.push_str(&format!(
                "| {} | {} | {:.2} CZK | {:.1} | {:.1}% | {} |\n",
                idx + 1,
                result.strategy_name,
                result.metrics.financial.net_profit_czk,
                result.metrics.battery.total_cycles,
                result.metrics.efficiency.self_consumption_rate,
                result.metrics.efficiency.efficiency_grade()
            ));
        }

        output.push_str("\n");

        // Detailed metrics for each strategy
        output.push_str("## Detailed Strategy Metrics\n\n");

        for result in &self.results {
            output.push_str(&format!("### {}\n\n", result.strategy_name));

            // Financial
            output.push_str("#### Financial Metrics\n\n");
            output.push_str(&format!(
                "- **Net Profit:** {:.2} CZK ({:.2} CZK/day)\n",
                result.metrics.financial.net_profit_czk,
                result.metrics.daily_average_profit()
            ));
            output.push_str(&format!(
                "- **Total Revenue:** {:.2} CZK\n",
                result.metrics.financial.total_revenue_czk
            ));
            output.push_str(&format!(
                "- **Total Cost:** {:.2} CZK\n",
                result.metrics.financial.total_cost_czk
            ));
            output.push_str(&format!(
                "- **Battery Wear Cost:** {:.2} CZK\n\n",
                result.metrics.financial.battery_wear_cost_czk
            ));

            // Battery
            output.push_str("#### Battery Health\n\n");
            output.push_str(&format!(
                "- **Total Cycles:** {:.1}\n",
                result.metrics.battery.total_cycles
            ));
            output.push_str(&format!(
                "- **Average DoD:** {:.1}%\n",
                result.metrics.battery.average_dod
            ));
            output.push_str(&format!(
                "- **SOC Range:** {:.1}% - {:.1}%\n\n",
                result.metrics.battery.min_soc,
                result.metrics.battery.max_soc
            ));

            // Efficiency
            output.push_str("#### Efficiency\n\n");
            output.push_str(&format!(
                "- **Self-Consumption Rate:** {:.1}%\n",
                result.metrics.efficiency.self_consumption_rate
            ));
            output.push_str(&format!(
                "- **Autarky Rate:** {:.1}%\n",
                result.metrics.efficiency.autarky_rate
            ));
            output.push_str(&format!(
                "- **Grid Independence:** {:.1}%\n",
                result.metrics.efficiency.grid_independence_score()
            ));
            output.push_str(&format!(
                "- **Grade:** {}\n\n",
                result.metrics.efficiency.efficiency_grade()
            ));
        }

        // Notes
        if !self.notes.is_empty() {
            output.push_str("## Notes\n\n");
            for note in &self.notes {
                output.push_str(&format!("- {}\n", note));
            }
            output.push_str("\n");
        }

        output
    }

    /// Generate JSON report
    fn generate_json(&self) -> String {
        serde_json::to_string_pretty(self).unwrap_or_else(|_| "{}".to_string())
    }

    /// Generate text report with tables
    fn generate_text(&self) -> String {
        let mut output = String::new();

        output.push_str(&format!("\n{}\n", "=".repeat(80)));
        output.push_str(&format!("{:^80}\n", self.title));
        output.push_str(&format!("{:^80}\n", format!("Generated: {}", self.generated_at.format("%Y-%m-%d %H:%M:%S UTC"))));
        output.push_str(&format!("{}\n\n", "=".repeat(80)));

        // Executive summary
        if let Some(baseline) = self.get_baseline() {
            if let Some(optimizer) = self.get_optimizer() {
                let improvement = optimizer.metrics.improvement_vs_baseline(&baseline.metrics);
                output.push_str("EXECUTIVE SUMMARY\n");
                output.push_str(&format!("─{}\n\n", "─".repeat(79)));
                output.push_str(&format!("  Baseline Profit:  {:>12.2} CZK\n", baseline.metrics.financial.net_profit_czk));
                output.push_str(&format!("  Optimized Profit: {:>12.2} CZK\n", optimizer.metrics.financial.net_profit_czk));
                output.push_str(&format!("  Improvement:      {:>12.1}%\n", improvement));
                output.push_str(&format!("  Duration:         {:>12.1} days\n\n", baseline.metrics.duration_days));
            }
        }

        // Strategy rankings table
        output.push_str("STRATEGY PERFORMANCE RANKINGS\n");
        output.push_str(&format!("─{}\n\n", "─".repeat(79)));

        let mut table = Table::new();
        table.load_preset(UTF8_FULL);
        table.set_header(vec![
            Cell::new("Rank").add_attribute(Attribute::Bold),
            Cell::new("Strategy").add_attribute(Attribute::Bold),
            Cell::new("Profit (CZK)").add_attribute(Attribute::Bold),
            Cell::new("Cycles").add_attribute(Attribute::Bold),
            Cell::new("Self-Use %").add_attribute(Attribute::Bold),
            Cell::new("Grade").add_attribute(Attribute::Bold),
        ]);

        for (idx, result) in self.sorted_by_profit().iter().enumerate() {
            let profit_cell = if idx == 0 {
                Cell::new(format!("{:.2}", result.metrics.financial.net_profit_czk))
                    .fg(Color::Green)
                    .add_attribute(Attribute::Bold)
            } else {
                Cell::new(format!("{:.2}", result.metrics.financial.net_profit_czk))
            };

            table.add_row(vec![
                Cell::new(idx + 1),
                Cell::new(&result.strategy_name),
                profit_cell,
                Cell::new(format!("{:.1}", result.metrics.battery.total_cycles)),
                Cell::new(format!("{:.1}", result.metrics.efficiency.self_consumption_rate)),
                Cell::new(result.metrics.efficiency.efficiency_grade()),
            ]);
        }

        output.push_str(&format!("{}\n\n", table));

        // Key insights
        output.push_str("KEY INSIGHTS\n");
        output.push_str(&format!("─{}\n\n", "─".repeat(79)));

        if let Some(best) = self.sorted_by_profit().first() {
            output.push_str(&format!("  ✓ Best performer: {} ({:.2} CZK)\n", best.strategy_name, best.metrics.financial.net_profit_czk));

            if best.metrics.battery.total_cycles > 100.0 {
                output.push_str(&format!("  ⚠ High battery usage: {:.1} cycles (monitor degradation)\n", best.metrics.battery.total_cycles));
            } else {
                output.push_str(&format!("  ✓ Healthy battery usage: {:.1} cycles\n", best.metrics.battery.total_cycles));
            }

            if best.metrics.efficiency.self_consumption_rate > 80.0 {
                output.push_str(&format!("  ✓ Excellent self-consumption: {:.1}%\n", best.metrics.efficiency.self_consumption_rate));
            }

            if best.metrics.efficiency.autarky_rate > 70.0 {
                output.push_str(&format!("  ✓ High grid independence: {:.1}% autarky\n", best.metrics.efficiency.autarky_rate));
            }
        }

        output.push_str("\n");

        output
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::metrics::MetricsSummary;
    use crate::simulator::BenchmarkConfig;

    #[test]
    fn test_report_creation() {
        let result = SimulationResult {
            strategy_name: "Test".to_string(),
            config: BenchmarkConfig::default(),
            metrics: MetricsSummary::new(),
            start_time: chrono::Utc::now(),
            end_time: chrono::Utc::now(),
        };

        let report = BenchmarkReport::new(vec![result], "Test Report");
        assert_eq!(report.title, "Test Report");
        assert_eq!(report.results.len(), 1);
    }

    #[test]
    fn test_report_sorting() {
        let mut result1 = SimulationResult {
            strategy_name: "Strategy1".to_string(),
            config: BenchmarkConfig::default(),
            metrics: MetricsSummary::new(),
            start_time: chrono::Utc::now(),
            end_time: chrono::Utc::now(),
        };
        result1.metrics.financial.net_profit_czk = 100.0;

        let mut result2 = SimulationResult {
            strategy_name: "Strategy2".to_string(),
            config: BenchmarkConfig::default(),
            metrics: MetricsSummary::new(),
            start_time: chrono::Utc::now(),
            end_time: chrono::Utc::now(),
        };
        result2.metrics.financial.net_profit_czk = 200.0;

        let report = BenchmarkReport::new(vec![result1, result2], "Test");
        let sorted = report.sorted_by_profit();

        assert_eq!(sorted[0].strategy_name, "Strategy2");
        assert_eq!(sorted[1].strategy_name, "Strategy1");
    }
}
