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

//! Temporary caching for price data during test runs

use super::{fetch_ote_cr_prices, load_from_csv_file, HistoricalPriceData};
use anyhow::{Context, Result};
use chrono::NaiveDate;
use std::collections::HashMap;
use std::path::PathBuf;
use tempfile::TempDir;
use tracing::{debug, info};

/// Price data cache that fetches once and reuses during test runs
///
/// The cache is stored in a temporary directory that is automatically
/// cleaned up when the cache is dropped.
pub struct PriceDataCache {
    /// Temporary directory for cached data
    temp_dir: TempDir,
    /// In-memory cache of loaded data
    cache: HashMap<String, HistoricalPriceData>,
    /// Optional manual CSV file path to use instead of fetching
    manual_csv_path: Option<PathBuf>,
}

impl PriceDataCache {
    /// Create a new price data cache
    pub fn new() -> Result<Self> {
        let temp_dir = TempDir::new().context("Failed to create temporary directory")?;
        info!("📦 Created price data cache at: {:?}", temp_dir.path());

        Ok(Self {
            temp_dir,
            cache: HashMap::new(),
            manual_csv_path: None,
        })
    }

    /// Set a manual CSV file to use instead of fetching from OTE-CR
    pub fn set_manual_csv(&mut self, path: PathBuf) {
        info!("📂 Using manual CSV file: {:?}", path);
        self.manual_csv_path = Some(path);
    }

    /// Get or fetch price data for a date range
    ///
    /// Data is fetched on first access and cached for subsequent calls.
    pub fn get_price_data(
        &mut self,
        start_date: NaiveDate,
        end_date: NaiveDate,
    ) -> Result<&HistoricalPriceData> {
        let cache_key = format!("{}_{}", start_date, end_date);

        if self.cache.contains_key(&cache_key) {
            debug!("💾 Using cached price data for {}", cache_key);
            return Ok(self.cache.get(&cache_key).unwrap());
        }

        info!("🌐 Fetching price data for {} to {}", start_date, end_date);

        let data = if let Some(csv_path) = &self.manual_csv_path {
            // Load from manual CSV file
            let mut data = load_from_csv_file(csv_path.to_str().unwrap())?;

            // Filter to requested date range
            data.prices.retain(|p| {
                let date = p.timestamp.date_naive();
                date >= start_date && date <= end_date
            });

            if data.prices.is_empty() {
                anyhow::bail!(
                    "No price data found in date range {} to {} in manual CSV",
                    start_date,
                    end_date
                );
            }

            data
        } else {
            // Fetch from OTE-CR
            fetch_ote_cr_prices(start_date, end_date)?
        };

        // Save to disk cache
        let cache_file = self
            .temp_dir
            .path()
            .join(format!("{}.json", cache_key));
        let json = serde_json::to_string_pretty(&data)?;
        std::fs::write(&cache_file, json)
            .context("Failed to write cache file")?;

        debug!("💾 Cached price data to: {:?}", cache_file);

        self.cache.insert(cache_key.clone(), data);
        Ok(self.cache.get(&cache_key).unwrap())
    }

    /// Get cache directory path (useful for debugging)
    pub fn cache_path(&self) -> &std::path::Path {
        self.temp_dir.path()
    }

    /// Clear the in-memory cache (but keep disk cache)
    pub fn clear_memory_cache(&mut self) {
        self.cache.clear();
    }

    /// Get number of cached date ranges
    pub fn cache_size(&self) -> usize {
        self.cache.len()
    }
}

impl Default for PriceDataCache {
    fn default() -> Self {
        Self::new().expect("Failed to create price data cache")
    }
}

impl Drop for PriceDataCache {
    fn drop(&mut self) {
        debug!("🗑️  Cleaning up price data cache");
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cache_creation() {
        let cache = PriceDataCache::new().unwrap();
        assert_eq!(cache.cache_size(), 0);
        assert!(cache.cache_path().exists());
    }

    #[test]
    fn test_cache_key_generation() {
        let start = NaiveDate::from_ymd_opt(2024, 12, 1).unwrap();
        let end = NaiveDate::from_ymd_opt(2024, 12, 31).unwrap();
        let key = format!("{}_{}", start, end);
        assert_eq!(key, "2024-12-01_2024-12-31");
    }
}
