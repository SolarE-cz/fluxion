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

pub mod async_runtime;
pub mod async_systems;
pub mod async_tasks;
pub mod components;
pub mod continuous_systems;
pub mod debug;
pub mod execution;
pub mod pricing;
pub mod resources;
pub mod scheduling;
pub mod strategy;
pub mod traits;
pub mod web_bridge;

pub use async_runtime::*;
pub use async_tasks::*;
use bevy_app::prelude::*;
use bevy_ecs::prelude::*;
pub use components::*;
pub use continuous_systems::{
    ContinuousSystemsPlugin, InverterDataSourceResource, PriceDataSourceResource,
    schedule_execution_system,
};
pub use debug::*;
pub use execution::*;
pub use pricing::*;
pub use resources::*;
pub use scheduling::*;
pub use strategy::*;
pub use traits::{
    EntityChange, GenericInverterState, InverterDataSource, ModeChangeRequest, PriceDataSource,
    VendorEntityMapper,
};
pub use web_bridge::{
    InverterData, PriceBlockData, PriceData, PvGenerationHistoryPoint, ScheduleData,
    SystemHealthData, WebQueryChannel, WebQueryResponse, WebQuerySender, web_query_system,
};

/// Core plugin that registers fundamental ECS resources and systems
pub struct FluxionCorePlugin;

impl Plugin for FluxionCorePlugin {
    fn build(&self, app: &mut App) {
        app
            // Initialize debug mode (default: enabled for safety)
            .init_resource::<DebugModeConfig>()
            // Note: ExecutionConfig is now inserted by main.rs with configured values
            .add_systems(Startup, debug_mode_startup_system)
            // Add continuous systems plugin
            .add_plugins(ContinuousSystemsPlugin);
    }
}

/// Startup system to log debug mode status
fn debug_mode_startup_system(debug_config: Res<DebugModeConfig>) {
    if debug_config.is_enabled() {
        tracing::info!("🔍 DEBUG MODE: Enabled (safe mode - no real changes will be made)");
        tracing::info!("🔍 Set debug_mode: false in config to enable production mode");
    } else {
        DebugModeConfig::warn_production_mode();
    }
}
