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

//! Consumption data loading and generation

use anyhow::{Context, Result};
use calamine::{open_workbook, DataType, Reader, Xlsx};
use chrono::{DateTime, Datelike, Duration, NaiveDate, NaiveDateTime, NaiveTime, TimeZone, Timelike, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::Path;
use tracing::{debug, info, warn};

/// Single consumption data point
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConsumptionPoint {
    /// Timestamp of the measurement
    pub timestamp: DateTime<Utc>,
    /// Consumption in kW at this timestamp
    pub consumption_kw: f32,
}

/// Collection of consumption data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConsumptionData {
    /// All consumption points sorted by timestamp
    pub points: Vec<ConsumptionPoint>,
    /// Source of the data
    pub source: String,
}

impl ConsumptionData {
    /// Get consumption at a specific timestamp (or nearest available)
    pub fn get_consumption_at(&self, timestamp: DateTime<Utc>) -> f32 {
        if self.points.is_empty() {
            return 0.0;
        }

        // Find the closest timestamp
        let closest = self
            .points
            .iter()
            .min_by_key(|p| {
                let diff = (p.timestamp.timestamp() - timestamp.timestamp()).abs();
                diff
            })
            .unwrap();

        closest.consumption_kw
    }

    /// Get average consumption for a time range
    pub fn average_in_range(&self, start: DateTime<Utc>, end: DateTime<Utc>) -> f32 {
        let points_in_range: Vec<f32> = self
            .points
            .iter()
            .filter(|p| p.timestamp >= start && p.timestamp <= end)
            .map(|p| p.consumption_kw)
            .collect();

        if points_in_range.is_empty() {
            return 0.0;
        }

        points_in_range.iter().sum::<f32>() / points_in_range.len() as f32
    }

    /// Interpolate consumption for 15-minute blocks
    pub fn interpolate_15min_blocks(&self, start: DateTime<Utc>, end: DateTime<Utc>) -> Vec<ConsumptionPoint> {
        let mut result = Vec::new();
        let mut current = start;

        while current < end {
            let consumption = self.get_consumption_at(current);
            result.push(ConsumptionPoint {
                timestamp: current,
                consumption_kw: consumption,
            });

            current = current + Duration::minutes(15);
        }

        result
    }
}

/// Load consumption data from Excel file
///
/// Expected formats:
/// - Two columns: Timestamp, Consumption (kW)
/// - First row is header
/// - Timestamp formats: ISO8601, "YYYY-MM-DD HH:MM:SS", "DD.MM.YYYY HH:MM"
pub fn load_from_excel<P: AsRef<Path>>(path: P) -> Result<ConsumptionData> {
    let path = path.as_ref();
    info!("📊 Loading consumption data from Excel: {:?}", path);

    let mut workbook: Xlsx<_> = open_workbook(path)
        .context("Failed to open Excel file")?;

    // Get the first worksheet
    let sheet_names = workbook.sheet_names().to_vec();
    if sheet_names.is_empty() {
        anyhow::bail!("Excel file has no worksheets");
    }

    let sheet_name = &sheet_names[0];
    info!("  Reading worksheet: {}", sheet_name);

    let range = workbook
        .worksheet_range(sheet_name)
        .context("Failed to read worksheet")?
        .context("Worksheet is empty")?;

    let mut points = Vec::new();
    let mut header_row = true;

    for (row_idx, row) in range.rows().enumerate() {
        // Skip header row
        if header_row {
            header_row = false;
            debug!("  Header: {:?}", row);
            continue;
        }

        if row.len() < 2 {
            warn!("  Skipping row {} - insufficient columns", row_idx + 1);
            continue;
        }

        // Parse timestamp (first column)
        let timestamp = parse_excel_timestamp(&row[0])
            .context(format!("Failed to parse timestamp in row {}", row_idx + 1))?;

        // Parse consumption (second column)
        let consumption_kw = parse_excel_number(&row[1])
            .context(format!("Failed to parse consumption in row {}", row_idx + 1))?;

        points.push(ConsumptionPoint {
            timestamp,
            consumption_kw,
        });
    }

    if points.is_empty() {
        anyhow::bail!("No valid consumption data found in Excel file");
    }

    points.sort_by(|a, b| a.timestamp.cmp(&b.timestamp));

    info!(
        "✓ Loaded {} consumption points from {} to {}",
        points.len(),
        points.first().unwrap().timestamp,
        points.last().unwrap().timestamp
    );

    Ok(ConsumptionData {
        points,
        source: format!("Excel:{}", path.display()),
    })
}

/// Load consumption data from CSV file
pub fn load_from_csv<P: AsRef<Path>>(path: P) -> Result<ConsumptionData> {
    let path = path.as_ref();
    info!("📊 Loading consumption data from CSV: {:?}", path);

    let mut reader = csv::ReaderBuilder::new()
        .has_headers(true)
        .from_path(path)
        .context("Failed to open CSV file")?;

    let mut points = Vec::new();

    for (idx, result) in reader.records().enumerate() {
        let record = result.context(format!("Failed to read row {}", idx + 1))?;

        if record.len() < 2 {
            warn!("Skipping row {} - insufficient columns", idx + 1);
            continue;
        }

        let timestamp_str = &record[0];
        let consumption_str = &record[1];

        // Parse timestamp
        let timestamp = DateTime::parse_from_rfc3339(timestamp_str)
            .or_else(|_| {
                NaiveDateTime::parse_from_str(timestamp_str, "%Y-%m-%d %H:%M:%S")
                    .map(|dt| DateTime::from_naive_utc_and_offset(dt, Utc).fixed_offset())
            })
            .or_else(|_| {
                NaiveDateTime::parse_from_str(timestamp_str, "%d.%m.%Y %H:%M")
                    .map(|dt| DateTime::from_naive_utc_and_offset(dt, Utc).fixed_offset())
            })
            .context(format!("Failed to parse timestamp in row {}: {}", idx + 1, timestamp_str))?
            .with_timezone(&Utc);

        let consumption_kw = consumption_str
            .parse::<f32>()
            .context(format!("Failed to parse consumption in row {}: {}", idx + 1, consumption_str))?;

        points.push(ConsumptionPoint {
            timestamp,
            consumption_kw,
        });
    }

    if points.is_empty() {
        anyhow::bail!("No valid consumption data found in CSV file");
    }

    points.sort_by(|a, b| a.timestamp.cmp(&b.timestamp));

    info!(
        "✓ Loaded {} consumption points from {} to {}",
        points.len(),
        points.first().unwrap().timestamp,
        points.last().unwrap().timestamp
    );

    Ok(ConsumptionData {
        points,
        source: format!("CSV:{}", path.display()),
    })
}

/// Parse Excel cell as timestamp
fn parse_excel_timestamp(cell: &DataType) -> Result<DateTime<Utc>> {
    match cell {
        DataType::String(s) => {
            // Try ISO8601
            if let Ok(dt) = DateTime::parse_from_rfc3339(s) {
                return Ok(dt.with_timezone(&Utc));
            }

            // Try "YYYY-MM-DD HH:MM:SS"
            if let Ok(dt) = NaiveDateTime::parse_from_str(s, "%Y-%m-%d %H:%M:%S") {
                return Ok(Utc.from_utc_datetime(&dt));
            }

            // Try "DD.MM.YYYY HH:MM"
            if let Ok(dt) = NaiveDateTime::parse_from_str(s, "%d.%m.%Y %H:%M") {
                return Ok(Utc.from_utc_datetime(&dt));
            }

            // Try "DD/MM/YYYY HH:MM"
            if let Ok(dt) = NaiveDateTime::parse_from_str(s, "%d/%m/%Y %H:%M") {
                return Ok(Utc.from_utc_datetime(&dt));
            }

            anyhow::bail!("Unsupported timestamp format: {}", s);
        }
        DataType::DateTime(excel_date) => {
            // Excel date is days since 1899-12-30
            let base_date = NaiveDate::from_ymd_opt(1899, 12, 30).unwrap();
            let days = excel_date.floor() as i64;
            let fraction = excel_date - excel_date.floor();
            let seconds = (fraction * 86400.0) as i64;

            let date = base_date + Duration::days(days);
            let time = NaiveTime::from_num_seconds_from_midnight_opt((seconds % 86400) as u32, 0)
                .context("Invalid time from Excel date")?;

            let naive_datetime = NaiveDateTime::new(date, time);
            Ok(Utc.from_utc_datetime(&naive_datetime))
        }
        _ => anyhow::bail!("Expected string or datetime, got {:?}", cell),
    }
}

/// Parse Excel cell as number
fn parse_excel_number(cell: &DataType) -> Result<f32> {
    match cell {
        DataType::Float(f) => Ok(*f as f32),
        DataType::Int(i) => Ok(*i as f32),
        DataType::String(s) => s
            .parse::<f32>()
            .context(format!("Failed to parse number from string: {}", s)),
        _ => anyhow::bail!("Expected number, got {:?}", cell),
    }
}

/// Predefined consumption profile
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ConsumptionProfile {
    /// Typical household with 4 people
    TypicalHousehold,
    /// Small household (1-2 people)
    SmallHousehold,
    /// Large household (5+ people)
    LargeHousehold,
    /// Custom average consumption (kW)
    Custom(f32),
}

impl ConsumptionProfile {
    /// Generate synthetic consumption data for a date range
    ///
    /// Creates realistic consumption patterns with:
    /// - Morning peak (6-9 AM)
    /// - Evening peak (5-10 PM)
    /// - Lower consumption at night
    /// - Weekend vs weekday differences
    pub fn generate(&self, start: NaiveDate, end: NaiveDate) -> ConsumptionData {
        let base_consumption = match self {
            ConsumptionProfile::TypicalHousehold => 0.5,  // kW
            ConsumptionProfile::SmallHousehold => 0.3,
            ConsumptionProfile::LargeHousehold => 0.8,
            ConsumptionProfile::Custom(c) => *c,
        };

        let mut points = Vec::new();
        let mut current_date = start;

        while current_date <= end {
            // Generate 96 points for this day (15-minute intervals)
            for hour in 0..24 {
                for minute in [0, 15, 30, 45] {
                    let time = NaiveTime::from_hms_opt(hour, minute, 0).unwrap();
                    let naive_dt = NaiveDateTime::new(current_date, time);
                    let timestamp = Utc.from_utc_datetime(&naive_dt);

                    // Calculate consumption based on time of day
                    let consumption = calculate_consumption_pattern(
                        base_consumption,
                        hour,
                        current_date.weekday().num_days_from_monday() >= 5, // Weekend
                    );

                    points.push(ConsumptionPoint {
                        timestamp,
                        consumption_kw: consumption,
                    });
                }
            }

            current_date = current_date.succ_opt().unwrap();
        }

        ConsumptionData {
            points,
            source: format!("Synthetic:{:?}", self),
        }
    }
}

/// Calculate realistic consumption pattern for a given hour
fn calculate_consumption_pattern(base_kw: f32, hour: u32, is_weekend: bool) -> f32 {
    let pattern_multiplier = match hour {
        // Night (0-5 AM): 40-50% of base
        0..=5 => 0.4 + (hour as f32 * 0.02),

        // Morning peak (6-9 AM): 100-150% of base
        6..=9 => 1.0 + ((hour - 6) as f32 * 0.15),

        // Day (10 AM-4 PM): 60-80% of base
        10..=16 => if is_weekend { 0.9 } else { 0.7 },

        // Evening peak (5-10 PM): 120-180% of base
        17..=22 => 1.2 + ((hour - 17) as f32 * 0.1),

        // Late night (11 PM-midnight): 60% of base
        _ => 0.6,
    };

    // Add some random variation (±10%)
    let variation = 1.0 + (rand_variation() * 0.1 - 0.05);

    base_kw * pattern_multiplier * variation
}

/// Simple pseudo-random variation (deterministic for testing)
fn rand_variation() -> f32 {
    // Use a simple deterministic pattern for reproducibility
    0.5
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_consumption_profile_generation() {
        let start = NaiveDate::from_ymd_opt(2024, 12, 1).unwrap();
        let end = NaiveDate::from_ymd_opt(2024, 12, 2).unwrap();

        let profile = ConsumptionProfile::TypicalHousehold;
        let data = profile.generate(start, end);

        // Should have 96 points per day * 2 days = 192
        assert_eq!(data.points.len(), 192);
    }

    #[test]
    fn test_consumption_pattern() {
        // Night should be lower than evening peak
        let night = calculate_consumption_pattern(1.0, 2, false);
        let evening = calculate_consumption_pattern(1.0, 19, false);
        assert!(night < evening);

        // Morning peak should be higher than day
        let morning = calculate_consumption_pattern(1.0, 7, false);
        let day = calculate_consumption_pattern(1.0, 14, false);
        assert!(morning > day);
    }

    #[test]
    fn test_parse_excel_number() {
        assert_eq!(parse_excel_number(&DataType::Float(1.5)).unwrap(), 1.5);
        assert_eq!(parse_excel_number(&DataType::Int(42)).unwrap(), 42.0);
        assert_eq!(
            parse_excel_number(&DataType::String("3.14".to_string())).unwrap(),
            3.14
        );
    }
}
