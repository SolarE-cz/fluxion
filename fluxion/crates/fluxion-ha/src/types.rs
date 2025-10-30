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

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HaEntityState {
    pub entity_id: String,
    pub state: String,
    pub attributes: serde_json::Value,
    pub last_changed: String,
    pub last_updated: String,
}

/// Historical state point from HA history API
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HaHistoryState {
    pub entity_id: String,
    pub state: String,
    pub attributes: Option<serde_json::Value>,
    pub last_changed: String,
    pub last_updated: String,
}

/// Parsed history data point with numeric value and timestamp
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HistoryDataPoint {
    pub timestamp: DateTime<Utc>,
    pub value: f32,
}
