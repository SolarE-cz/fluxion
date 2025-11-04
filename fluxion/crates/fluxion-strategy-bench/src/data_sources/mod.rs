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

//! Data source modules for fetching and loading historical data

mod cache;
mod consumption;
mod price_fetcher;

pub use cache::PriceDataCache;
pub use consumption::{ConsumptionData, ConsumptionProfile};
pub use price_fetcher::{fetch_ote_cr_prices, PriceFetchError};

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// Historical price data for a single 15-minute block
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HistoricalPricePoint {
    /// Timestamp of the block start
    pub timestamp: DateTime<Utc>,
    /// Price in CZK/kWh
    pub price_czk_per_kwh: f32,
    /// Data source identifier
    pub source: String,
}

/// Collection of historical price data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HistoricalPriceData {
    /// All price points sorted by timestamp
    pub prices: Vec<HistoricalPricePoint>,
    /// Start date of the data
    pub start_date: DateTime<Utc>,
    /// End date of the data
    pub end_date: DateTime<Utc>,
    /// Data source
    pub source: String,
}

impl HistoricalPriceData {
    /// Get price for a specific timestamp (or nearest available)
    pub fn get_price_at(&self, timestamp: DateTime<Utc>) -> Option<f32> {
        self.prices
            .iter()
            .find(|p| p.timestamp == timestamp)
            .map(|p| p.price_czk_per_kwh)
    }

    /// Get all prices within a date range
    pub fn prices_in_range(
        &self,
        start: DateTime<Utc>,
        end: DateTime<Utc>,
    ) -> Vec<&HistoricalPricePoint> {
        self.prices
            .iter()
            .filter(|p| p.timestamp >= start && p.timestamp <= end)
            .collect()
    }

    /// Calculate statistics for the price data
    pub fn statistics(&self) -> PriceStatistics {
        let prices: Vec<f32> = self.prices.iter().map(|p| p.price_czk_per_kwh).collect();

        if prices.is_empty() {
            return PriceStatistics::default();
        }

        let sum: f32 = prices.iter().sum();
        let count = prices.len() as f32;
        let mean = sum / count;

        let mut sorted_prices = prices.clone();
        sorted_prices.sort_by(|a, b| a.partial_cmp(b).unwrap());

        let min = sorted_prices[0];
        let max = sorted_prices[sorted_prices.len() - 1];
        let median = sorted_prices[sorted_prices.len() / 2];

        // Calculate standard deviation
        let variance: f32 = prices.iter().map(|p| (p - mean).powi(2)).sum::<f32>() / count;
        let std_dev = variance.sqrt();

        PriceStatistics {
            mean,
            median,
            min,
            max,
            std_dev,
            count: prices.len(),
        }
    }
}

/// Statistical summary of price data
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct PriceStatistics {
    pub mean: f32,
    pub median: f32,
    pub min: f32,
    pub max: f32,
    pub std_dev: f32,
    pub count: usize,
}
