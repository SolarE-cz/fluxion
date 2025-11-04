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

//! OTE-CR electricity price data fetcher with robust retry logic

use super::{HistoricalPriceData, HistoricalPricePoint};
use anyhow::{Context, Result};
use chrono::{DateTime, Duration, NaiveDate, TimeZone, Utc};
use serde::{Deserialize, Serialize};
use std::thread::sleep;
use std::time::Duration as StdDuration;
use tracing::{debug, info, warn};

const OTE_CR_BASE_URL: &str =
    "https://www.ote-cr.cz/en/short-term-markets/electricity/day-ahead-market";
const MAX_RETRIES: u32 = 4;
const INITIAL_BACKOFF_MS: u64 = 2000;

/// Errors that can occur when fetching price data
#[derive(Debug, thiserror::Error)]
pub enum PriceFetchError {
    #[error("HTTP request failed: {0}")]
    HttpError(#[from] reqwest::Error),

    #[error("Failed to parse response: {0}")]
    ParseError(String),

    #[error("No data available for date {0}")]
    NoDataAvailable(String),

    #[error("Access denied (403). OTE-CR may require manual data export. Please use --manual-csv option.")]
    AccessDenied,

    #[error("Rate limited. Please try again later.")]
    RateLimited,
}

/// Fetch historical electricity prices from OTE-CR for a date range
///
/// This function attempts multiple strategies:
/// 1. Direct API/web fetch with proper headers and retry logic
/// 2. If 403 errors persist, provides guidance for manual CSV export
///
/// # Arguments
/// * `start_date` - Start date (inclusive)
/// * `end_date` - End date (inclusive)
///
/// # Returns
/// Historical price data in 15-minute blocks
pub fn fetch_ote_cr_prices(
    start_date: NaiveDate,
    end_date: NaiveDate,
) -> Result<HistoricalPriceData> {
    info!(
        "📊 Fetching OTE-CR price data from {} to {}",
        start_date, end_date
    );

    let mut all_prices = Vec::new();
    let mut current_date = start_date;

    while current_date <= end_date {
        match fetch_single_day_with_retry(current_date) {
            Ok(day_prices) => {
                info!("  ✓ Fetched {} blocks for {}", day_prices.len(), current_date);
                all_prices.extend(day_prices);
            }
            Err(e) => {
                warn!("  ✗ Failed to fetch data for {}: {}", current_date, e);
                // For 403 errors, provide helpful guidance
                if matches!(e.downcast_ref::<PriceFetchError>(), Some(PriceFetchError::AccessDenied)) {
                    warn!("");
                    warn!("╔════════════════════════════════════════════════════════════════╗");
                    warn!("║  OTE-CR website may block automated access.                   ║");
                    warn!("║  Alternative options:                                          ║");
                    warn!("║                                                                ║");
                    warn!("║  1. Manual CSV Export:                                         ║");
                    warn!("║     Visit: {}", OTE_CR_BASE_URL);
                    warn!("║     Export data as CSV and use --manual-csv flag               ║");
                    warn!("║                                                                ║");
                    warn!("║  2. Use cached test data (if available)                       ║");
                    warn!("║                                                                ║");
                    warn!("╚════════════════════════════════════════════════════════════════╝");
                    warn!("");

                    return Err(e);
                }
            }
        }

        current_date = current_date
            .succ_opt()
            .context("Date overflow")?;

        // Be respectful to the server
        sleep(StdDuration::from_millis(500));
    }

    if all_prices.is_empty() {
        anyhow::bail!(PriceFetchError::NoDataAvailable(format!(
            "{} to {}",
            start_date, end_date
        )));
    }

    all_prices.sort_by(|a, b| a.timestamp.cmp(&b.timestamp));

    let start_datetime = Utc
        .from_local_datetime(&start_date.and_hms_opt(0, 0, 0).unwrap())
        .single()
        .context("Invalid start date")?;
    let end_datetime = Utc
        .from_local_datetime(&end_date.and_hms_opt(23, 59, 59).unwrap())
        .single()
        .context("Invalid end date")?;

    Ok(HistoricalPriceData {
        prices: all_prices,
        start_date: start_datetime,
        end_date: end_datetime,
        source: "OTE-CR".to_string(),
    })
}

/// Fetch price data for a single day with exponential backoff retry
fn fetch_single_day_with_retry(date: NaiveDate) -> Result<Vec<HistoricalPricePoint>> {
    let mut retries = 0;
    let mut backoff_ms = INITIAL_BACKOFF_MS;

    loop {
        match fetch_single_day(date) {
            Ok(prices) => return Ok(prices),
            Err(e) => {
                // Check if it's a 403 error
                if let Some(PriceFetchError::AccessDenied) = e.downcast_ref::<PriceFetchError>() {
                    return Err(e);
                }

                retries += 1;
                if retries >= MAX_RETRIES {
                    return Err(e).context("Max retries exceeded");
                }

                debug!(
                    "Retry {}/{} for {} after {}ms: {}",
                    retries, MAX_RETRIES, date, backoff_ms, e
                );

                sleep(StdDuration::from_millis(backoff_ms));
                backoff_ms *= 2; // Exponential backoff
            }
        }
    }
}

/// Fetch price data for a single day from OTE-CR
fn fetch_single_day(date: NaiveDate) -> Result<Vec<HistoricalPricePoint>> {
    let url = format!(
        "{}?time_resolution=PT15M&date={}",
        OTE_CR_BASE_URL,
        date.format("%Y-%m-%d")
    );

    debug!("Fetching: {}", url);

    // Build HTTP client with realistic browser headers to avoid bot detection
    let client = reqwest::blocking::Client::builder()
        .user_agent("Mozilla/5.0 (X11; Linux x86_64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/131.0.0.0 Safari/537.36")
        .timeout(StdDuration::from_secs(30))
        .build()?;

    let response = client
        .get(&url)
        .header("Accept", "text/html,application/xhtml+xml,application/xml;q=0.9,*/*;q=0.8")
        .header("Accept-Language", "en-US,en;q=0.9")
        .header("Accept-Encoding", "gzip, deflate")
        .header("Connection", "keep-alive")
        .send()?;

    let status = response.status();
    debug!("Response status: {}", status);

    if status == reqwest::StatusCode::FORBIDDEN {
        return Err(PriceFetchError::AccessDenied.into());
    }

    if status == reqwest::StatusCode::TOO_MANY_REQUESTS {
        return Err(PriceFetchError::RateLimited.into());
    }

    if !status.is_success() {
        anyhow::bail!("HTTP error: {}", status);
    }

    let body = response.text()?;

    // Try to parse the response
    // OTE-CR may return HTML, JSON, or CSV depending on the endpoint
    parse_ote_cr_response(&body, date)
}

/// Parse OTE-CR response (attempts multiple formats)
fn parse_ote_cr_response(body: &str, date: NaiveDate) -> Result<Vec<HistoricalPricePoint>> {
    // Try JSON first
    if let Ok(prices) = parse_json_response(body, date) {
        return Ok(prices);
    }

    // Try CSV format
    if let Ok(prices) = parse_csv_response(body, date) {
        return Ok(prices);
    }

    // Try HTML table parsing
    if let Ok(prices) = parse_html_response(body, date) {
        return Ok(prices);
    }

    Err(PriceFetchError::ParseError(
        "Unable to parse response in any known format (JSON/CSV/HTML)".to_string(),
    )
    .into())
}

/// Parse JSON format response
fn parse_json_response(body: &str, date: NaiveDate) -> Result<Vec<HistoricalPricePoint>> {
    #[derive(Deserialize)]
    struct JsonResponse {
        data: Vec<JsonDataPoint>,
    }

    #[derive(Deserialize)]
    struct JsonDataPoint {
        #[serde(alias = "DateTime", alias = "datetime", alias = "timestamp")]
        datetime: String,
        #[serde(alias = "Price", alias = "price", alias = "value")]
        price: f32,
    }

    let response: JsonResponse = serde_json::from_str(body)?;

    let prices = response
        .data
        .into_iter()
        .filter_map(|point| {
            let timestamp = DateTime::parse_from_rfc3339(&point.datetime)
                .ok()?
                .with_timezone(&Utc);

            Some(HistoricalPricePoint {
                timestamp,
                price_czk_per_kwh: point.price / 1000.0, // Convert EUR/MWh to CZK/kWh (assuming conversion)
                source: "OTE-CR-API".to_string(),
            })
        })
        .collect();

    Ok(prices)
}

/// Parse CSV format response
fn parse_csv_response(body: &str, date: NaiveDate) -> Result<Vec<HistoricalPricePoint>> {
    let mut reader = csv::ReaderBuilder::new()
        .has_headers(true)
        .flexible(true)
        .from_reader(body.as_bytes());

    let mut prices = Vec::new();

    for result in reader.records() {
        let record = result?;

        // Try to parse timestamp and price from various column positions
        if let (Some(timestamp_str), Some(price_str)) = (record.get(0), record.get(1)) {
            if let (Ok(timestamp), Ok(price)) = (
                DateTime::parse_from_rfc3339(timestamp_str)
                    .or_else(|_| {
                        // Try alternative formats
                        chrono::NaiveDateTime::parse_from_str(timestamp_str, "%Y-%m-%d %H:%M:%S")
                            .map(|dt| DateTime::from_naive_utc_and_offset(dt, Utc).fixed_offset())
                    }),
                price_str.parse::<f32>(),
            ) {
                prices.push(HistoricalPricePoint {
                    timestamp: timestamp.with_timezone(&Utc),
                    price_czk_per_kwh: price / 1000.0,
                    source: "OTE-CR-CSV".to_string(),
                });
            }
        }
    }

    if prices.is_empty() {
        anyhow::bail!("No valid price data found in CSV");
    }

    Ok(prices)
}

/// Parse HTML table response (basic implementation)
fn parse_html_response(body: &str, date: NaiveDate) -> Result<Vec<HistoricalPricePoint>> {
    // This is a placeholder for HTML parsing
    // In a real implementation, you'd use a crate like `scraper` or `select`
    // For now, we'll return an error to indicate this needs manual implementation
    anyhow::bail!("HTML parsing not yet implemented. Please use manual CSV export.")
}

/// Load price data from manually exported CSV file
pub fn load_from_csv_file(file_path: &str) -> Result<HistoricalPriceData> {
    info!("📂 Loading price data from CSV: {}", file_path);

    let mut reader = csv::ReaderBuilder::new()
        .has_headers(true)
        .from_path(file_path)
        .context("Failed to open CSV file")?;

    let mut prices = Vec::new();

    for (idx, result) in reader.records().enumerate() {
        let record = result.context(format!("Failed to read row {}", idx + 1))?;

        // Expected format: Timestamp, Price (CZK/kWh)
        if record.len() < 2 {
            warn!("Skipping row {} - insufficient columns", idx + 1);
            continue;
        }

        let timestamp_str = &record[0];
        let price_str = &record[1];

        // Parse timestamp (try multiple formats)
        let timestamp = DateTime::parse_from_rfc3339(timestamp_str)
            .or_else(|_| {
                chrono::NaiveDateTime::parse_from_str(timestamp_str, "%Y-%m-%d %H:%M:%S")
                    .map(|dt| DateTime::from_naive_utc_and_offset(dt, Utc).fixed_offset())
            })
            .or_else(|_| {
                chrono::NaiveDateTime::parse_from_str(timestamp_str, "%d.%m.%Y %H:%M")
                    .map(|dt| DateTime::from_naive_utc_and_offset(dt, Utc).fixed_offset())
            })
            .context(format!("Failed to parse timestamp in row {}: {}", idx + 1, timestamp_str))?;

        let price = price_str
            .parse::<f32>()
            .context(format!("Failed to parse price in row {}: {}", idx + 1, price_str))?;

        prices.push(HistoricalPricePoint {
            timestamp: timestamp.with_timezone(&Utc),
            price_czk_per_kwh: price,
            source: "Manual-CSV".to_string(),
        });
    }

    if prices.is_empty() {
        anyhow::bail!("No valid price data found in CSV file");
    }

    prices.sort_by(|a, b| a.timestamp.cmp(&b.timestamp));

    let start_date = prices.first().unwrap().timestamp;
    let end_date = prices.last().unwrap().timestamp;

    info!(
        "✓ Loaded {} price points from {} to {}",
        prices.len(),
        start_date,
        end_date
    );

    Ok(HistoricalPriceData {
        prices,
        start_date,
        end_date,
        source: "Manual-CSV".to_string(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_json_response() {
        let json = r#"{
            "data": [
                {"datetime": "2024-12-01T00:00:00Z", "price": 500.0},
                {"datetime": "2024-12-01T00:15:00Z", "price": 520.0}
            ]
        }"#;

        let date = NaiveDate::from_ymd_opt(2024, 12, 1).unwrap();
        let prices = parse_json_response(json, date).unwrap();

        assert_eq!(prices.len(), 2);
        assert_eq!(prices[0].price_czk_per_kwh, 0.5); // 500 / 1000
    }

    #[test]
    fn test_parse_csv_response() {
        let csv = "Timestamp,Price\n2024-12-01T00:00:00Z,500.0\n2024-12-01T00:15:00Z,520.0";

        let date = NaiveDate::from_ymd_opt(2024, 12, 1).unwrap();
        let prices = parse_csv_response(csv, date).unwrap();

        assert_eq!(prices.len(), 2);
    }
}
