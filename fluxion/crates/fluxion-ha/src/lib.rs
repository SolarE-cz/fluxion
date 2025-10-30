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

pub mod adapters;
pub mod client;
pub mod errors;
pub mod plugin;
pub mod types;

pub use adapters::{CzSpotPriceAdapter, HomeAssistantInverterAdapter};
pub use client::HomeAssistantClient;
pub use errors::{HaError, HaResult};
pub use plugin::HaPlugin;
pub use types::{HaEntityState, HaHistoryState, HistoryDataPoint};
