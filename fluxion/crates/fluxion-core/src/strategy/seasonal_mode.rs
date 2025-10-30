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

// Seasonal mode detection and thresholds
// Based on analysis/WINTER_STRATEGY_REFINED.md (lines ~483-500)

use chrono::{DateTime, Datelike, Utc};

/// Seasonal operating mode
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SeasonalMode {
    /// May-September
    Summer,
    /// October-April
    Winter,
}

impl SeasonalMode {
    /// Determine the season from a UTC date
    #[must_use]
    pub fn from_date(date: DateTime<Utc>) -> Self {
        match date.month() {
            5..=9 => Self::Summer,
            _ => Self::Winter,
        }
    }

    /// Minimum SOC recommendation by season (percent)
    #[must_use]
    pub fn min_soc_percent(&self) -> f32 {
        match self {
            Self::Summer => 20.0, // Aggressive
            Self::Winter => 50.0, // Conservative
        }
    }

    /// Minimum spread threshold for arbitrage (CZK/kWh)
    #[must_use]
    pub fn min_spread_threshold(&self) -> f32 {
        match self {
            Self::Summer => 2.0, // Lower bar
            Self::Winter => 3.0, // Higher bar for profitability
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_seasonal_detection_october() {
        let date = DateTime::parse_from_rfc3339("2025-10-14T10:00:00Z")
            .unwrap()
            .with_timezone(&Utc);
        assert_eq!(SeasonalMode::from_date(date), SeasonalMode::Winter);
    }

    #[test]
    fn test_seasonal_detection_july() {
        let date = DateTime::parse_from_rfc3339("2025-07-15T10:00:00Z")
            .unwrap()
            .with_timezone(&Utc);
        assert_eq!(SeasonalMode::from_date(date), SeasonalMode::Summer);
    }
}
