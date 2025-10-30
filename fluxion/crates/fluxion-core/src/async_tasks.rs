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

/// Components for managing async workers via channels in Bevy ECS
///
/// Architecture:
/// - Long-running background tasks spawned once at startup
/// - Communication via channels (crossbeam)
/// - ECS systems use try_recv() for non-blocking message passing
use bevy_ecs::prelude::*;
use chrono::{DateTime, Utc};
use crossbeam_channel::{Receiver, Sender};

use crate::components::{InverterCommand, SpotPriceData};

// ============= Price Fetcher =============

/// Component marking this entity as the price fetcher worker
#[derive(Component)]
pub struct PriceFetcher {
    pub source_name: String,
    pub fetch_interval_secs: u64,
}

/// Component that holds a channel receiver for price updates
#[derive(Component)]
pub struct PriceChannel {
    pub receiver: Receiver<SpotPriceData>,
}

// ============= Inverter Command Writer =============

/// Component marking this entity as the inverter command writer worker
#[derive(Component)]
pub struct InverterCommandWriter {
    pub source_name: String,
}

/// Component that holds channels for sending commands and receiving results
#[derive(Component)]
pub struct InverterCommandChannel {
    /// Send commands to the background worker
    pub sender: Sender<(String, InverterCommand)>,
    /// Receive command results from the background worker
    pub result_receiver: Receiver<(String, anyhow::Result<()>)>,
}

// ============= Health Checker =============

/// Component marking this entity as a health checker worker
#[derive(Component)]
pub struct HealthChecker {
    pub source_name: String,
    pub check_interval_secs: u64,
}

/// Component that holds a channel receiver for health check results
/// The tuple contains (source_name, is_healthy)
#[derive(Component)]
pub struct HealthCheckChannel {
    pub receiver: Receiver<(String, bool)>,
}

// ============= Inverter State Reader =============

/// Component marking this entity as the inverter state reader worker
#[derive(Component)]
pub struct InverterStateReader {
    pub poll_interval_secs: u64,
}

/// Component that holds channels for inverter state polling
/// The tuple contains (inverter_id, Result<GenericInverterState>)
#[derive(Component)]
pub struct InverterStateChannel {
    pub sender: Sender<(String, anyhow::Result<crate::GenericInverterState>)>,
    pub receiver: Receiver<(String, anyhow::Result<crate::GenericInverterState>)>,
}

// ============= Battery History Fetcher =============

/// Component marking this entity as the battery history fetcher worker
#[derive(Component)]
pub struct BatteryHistoryFetcher {
    pub fetch_interval_secs: u64,
}

/// Component that holds a channel receiver for battery history fetch requests
/// The tuple contains (entity_id, start_time)
#[derive(Component)]
pub struct BatteryHistoryChannel {
    pub receiver: Receiver<(String, DateTime<Utc>)>,
}
