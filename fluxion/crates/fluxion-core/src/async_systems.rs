// Copyright (c) 2025 SOLARE S.R.O.
//
// This file is part of FluxION.
//
// Licensed under the Creative Commons Attribution-NonCommercial-NoDerivatives 4.0 International
// (CC BY-NC-ND 4.0). You may use and share this file for non-commercial purposes only and you may not
// create derivatives. See <https://creativecommons.org/licenses/by-nc-nd/4.0/>.
//
// This software is provided "AS IS", without warranty of any kind.
//
// For commercial licensing, please contact: info@solare.cz

use bevy_ecs::prelude::*;
use crossbeam_channel::unbounded;
use futures_timer::Delay;
use std::time::Duration;
use tracing::{debug, error, info, trace, warn};

use crate::{
    async_runtime::AsyncRuntime,
    async_tasks::*,
    components::*,
    pricing::analyze_prices,
    resources::SystemConfig,
    scheduling::{ScheduleConfig, generate_schedule_with_optimizer},
};

use super::{InverterDataSourceResource, PriceDataSourceResource};

/// Startup system that spawns all long-running async worker tasks
/// These tasks run in the background and communicate via channels
pub fn setup_async_workers(
    mut commands: Commands,
    runtime: Res<AsyncRuntime>,
    price_source: Res<PriceDataSourceResource>,
    inverter_source: Res<InverterDataSourceResource>,
) {
    use bevy_tasks::AsyncComputeTaskPool;

    // Ensure AsyncComputeTaskPool is initialized
    let pool = AsyncComputeTaskPool::get();
    info!(
        "🚀 Setting up async worker entities (pool threads: {})...",
        pool.thread_num()
    );

    // Note: We don't actually use Bevy's task pool for spawning because our async code
    // uses reqwest/tokio which requires tokio runtime. Instead we rely on tokio runtime
    // being active in the main thread.

    // ============= Price Fetcher Worker =============

    let (price_tx, price_rx) = unbounded();
    let price_source_clone = price_source.0.clone();

    runtime.spawn(async move {
        info!("💰 Price fetcher worker started");

        // Fetch immediately on startup for fast initialization
        debug!("Fetching initial price data...");
        match price_source_clone.read_prices().await {
            Ok(prices) => {
                debug!(
                    "✅ Fetched {} price blocks on startup",
                    prices.time_block_prices.len()
                );
                if let Err(e) = price_tx.send(prices) {
                    error!("Failed to send prices to channel: {}", e);
                    return; // Channel closed, worker should exit
                }
            }
            Err(e) => {
                error!("❌ Failed to fetch initial prices: {}", e);
            }
        }

        // Then continue with regular polling
        loop {
            Delay::new(Duration::from_secs(5)).await;

            debug!("Fetching price data...");
            match price_source_clone.read_prices().await {
                Ok(prices) => {
                    debug!("✅ Fetched {} price blocks", prices.time_block_prices.len());
                    if let Err(e) = price_tx.send(prices) {
                        error!("Failed to send prices to channel: {}", e);
                        break; // Channel closed, worker should exit
                    }
                }
                Err(e) => {
                    error!("❌ Failed to fetch prices: {}", e);
                }
            }
        }
        warn!("💰 Price fetcher worker stopped");
    });

    commands.spawn((
        PriceFetcher {
            source_name: "spot_price".to_string(),
            fetch_interval_secs: 60,
        },
        PriceChannel { receiver: price_rx },
    ));

    info!("✅ Price fetcher entity created");

    // ============= Battery History Fetcher Worker =============
    // Note: Battery history is fetched via a simpler approach - directly in a system
    // that has access to the BatteryHistory resource and can update it.
    // The fetch happens periodically in trigger_battery_history_fetch_system

    // ============= Inverter Command Writer Worker =============

    let (cmd_tx, cmd_rx): (crossbeam_channel::Sender<(String, InverterCommand)>, _) = unbounded();
    let (result_tx, result_rx): (
        crossbeam_channel::Sender<(String, Result<(), anyhow::Error>)>,
        _,
    ) = unbounded();
    let inverter_source_clone = inverter_source.0.clone();

    runtime.spawn(async move {
        info!("🔌 Inverter command writer started");
        while let Ok((inverter_id, command)) = cmd_rx.recv() {
            debug!("Executing command for {}: {:?}", inverter_id, command);

            let result = inverter_source_clone
                .write_command(&inverter_id, &command)
                .await;

            if let Err(e) = result_tx.send((inverter_id.clone(), result)) {
                error!("Failed to send command result to channel: {}", e);
                break;
            }
        }
        warn!("🔌 Inverter command writer stopped");
    });

    commands.spawn((
        InverterCommandWriter {
            source_name: "ha_inverter".to_string(),
        },
        InverterCommandChannel {
            sender: cmd_tx,
            result_receiver: result_rx,
        },
    ));

    info!("✅ Inverter command writer entity created");

    // ============= Health Checker Workers =============
    info!("🛠️ Setting up health checkers...");

    // Health check for inverter source
    let (health_tx_inv, health_rx_inv) = unbounded();
    let inverter_source_clone = inverter_source.0.clone();

    info!("🛠️ Spawning inverter health checker...");
    runtime.spawn(async move {
        info!("🏥 Inverter health checker started");
        loop {
            Delay::new(Duration::from_secs(300)).await; // 5 minutes

            debug!("Checking inverter source health...");
            match inverter_source_clone.health_check().await {
                Ok(healthy) => {
                    if healthy {
                        debug!("✅ Inverter source healthy");
                    } else {
                        warn!("⚠️ Inverter source unhealthy");
                    }
                    let _ = health_tx_inv.send(("inverter_source".to_string(), healthy));
                }
                Err(e) => {
                    error!("❌ Inverter health check failed: {}", e);
                    let _ = health_tx_inv.send(("inverter_source".to_string(), false));
                }
            }
        }
    });

    // Health check for price source
    let (health_tx_price, health_rx_price) = unbounded();
    let price_source_clone = price_source.0.clone();

    runtime.spawn(async move {
        info!("🏥 Price health checker started");
        loop {
            Delay::new(Duration::from_secs(300)).await; // 5 minutes

            debug!("Checking price source health...");
            match price_source_clone.health_check().await {
                Ok(healthy) => {
                    if healthy {
                        debug!("✅ Price source healthy");
                    } else {
                        warn!("⚠️ Price source unhealthy");
                    }
                    let _ = health_tx_price.send(("price_source".to_string(), healthy));
                }
                Err(e) => {
                    error!("❌ Price health check failed: {}", e);
                    let _ = health_tx_price.send(("price_source".to_string(), false));
                }
            }
        }
    });

    // Create health check entities
    commands.spawn((
        HealthChecker {
            source_name: "inverter_source".to_string(),
            check_interval_secs: 300,
        },
        HealthCheckChannel {
            receiver: health_rx_inv,
        },
    ));

    commands.spawn((
        HealthChecker {
            source_name: "price_source".to_string(),
            check_interval_secs: 300,
        },
        HealthCheckChannel {
            receiver: health_rx_price,
        },
    ));

    info!("✅ Health checker entities created");

    // ============= Inverter State Reader Channel =============
    info!("📊 Setting up inverter state reader channel...");

    let (state_tx, state_rx) = unbounded();

    commands.spawn((
        InverterStateReader {
            poll_interval_secs: 5,
        },
        InverterStateChannel {
            sender: state_tx,
            receiver: state_rx,
        },
    ));

    info!("✅ Inverter state reader entity created");
    info!("🎉 All async workers initialized successfully");
}

/// System that polls the price channel and updates price data
pub fn poll_price_channel(
    price_channel: Query<&PriceChannel>,
    mut commands: Commands,
    mut price_data_query: Query<(Entity, &mut SpotPriceData)>,
    mut price_analysis_query: Query<(Entity, &mut PriceAnalysis)>,
    mut schedule_query: Query<&mut OperationSchedule>,
    battery_query: Query<&BatteryStatus>,
    config: Res<SystemConfig>,
) {
    // Get the price channel
    let Ok(channel) = price_channel.single() else {
        return; // No price fetcher entity yet
    };

    // NON-BLOCKING: try to receive from channel
    // Process all available messages in queue
    while let Ok(new_prices) = channel.receiver.try_recv() {
        let new_block_count = new_prices.time_block_prices.len();
        let new_hours = new_block_count as f32 / 4.0;

        // Detect if we got significantly more data (day-ahead prices arrived)
        let old_block_count = price_data_query
            .single()
            .ok()
            .map(|(_, data)| data.time_block_prices.len())
            .unwrap_or(0);

        let is_day_ahead_arrival = new_block_count > old_block_count + 10;

        if is_day_ahead_arrival {
            info!(
                "📊 Day-ahead prices arrived! Old: {} blocks ({:.1}h), New: {} blocks ({:.1}h). Will recalculate schedule.",
                old_block_count,
                old_block_count as f32 / 4.0,
                new_block_count,
                new_hours
            );
        } else {
            debug!(
                "📊 Received price data update: {} blocks ({:.1} hours) - no schedule regeneration needed",
                new_block_count, new_hours
            );
        }

        // Update price data entity or create if doesn't exist
        if let Ok((_, mut price_data)) = price_data_query.single_mut() {
            *price_data = new_prices.clone();
        } else {
            commands.spawn(new_prices.clone());
        }

        info!(
            "🔄 Regenerating schedule due to: {}",
            if is_day_ahead_arrival {
                "day-ahead prices arrival"
            } else {
                "no existing schedule"
            }
        );

        // Regenerate schedule based on new prices
        let analysis = analyze_prices(
            &new_prices.time_block_prices,
            config.control_config.force_charge_hours,
            config.control_config.force_discharge_hours,
            config.pricing_config.use_spot_prices_to_buy,
            config.pricing_config.use_spot_prices_to_sell,
        );

        let schedule_config = ScheduleConfig {
            min_battery_soc: config.control_config.min_battery_soc,
            max_battery_soc: config.control_config.max_battery_soc,
            target_inverters: config.inverters.iter().map(|i| i.id.clone()).collect(),
            display_currency: config.system_config.display_currency,
            default_battery_mode: config.control_config.default_battery_mode,
        };

        // Get current battery SOC (use average if multiple batteries, or default if none)
        let current_soc = battery_query
            .iter()
            .map(|b| b.soc_percent as f32)
            .sum::<f32>()
            / battery_query.iter().count().max(1) as f32;

        let current_soc = if current_soc > 0.0 { current_soc } else { 50.0 }; // Default to 50% if no data

        // Use economic optimizer for schedule generation
        let new_schedule = generate_schedule_with_optimizer(
            &new_prices.time_block_prices,
            &config.control_config,
            &schedule_config,
            current_soc,
            None, // Future: Add solar forecast integration (Solcast/Forecast.Solar API)
            None, // Future: Add consumption forecast (HA energy dashboard or ML model)
            0.8,  // Export price is typically 80% of import price
            Some(&config.strategies_config),
        );

        // Update or create PriceAnalysis entity
        if let Ok((_, mut price_analysis)) = price_analysis_query.single_mut() {
            *price_analysis = analysis;
        } else {
            commands.spawn(analysis);
        }

        // Update schedule or create if doesn't exist
        if let Ok(mut schedule) = schedule_query.single_mut() {
            *schedule = new_schedule;
            info!("✅ Schedule regenerated based on new price data");
        } else {
            commands.spawn(new_schedule);
            info!("✅ Initial schedule created");
        }
    }
}

/// System that polls health check channels and logs results
/// Also maintains HealthStatus components for each data source
pub fn poll_health_channels(
    health_channels: Query<&HealthCheckChannel>,
    mut health_status_query: Query<&mut HealthStatus>,
    mut commands: Commands,
) {
    for health_channel in health_channels.iter() {
        // NON-BLOCKING: check for health status updates
        while let Ok((source_name, is_healthy)) = health_channel.receiver.try_recv() {
            if is_healthy {
                debug!("✅ {} is healthy", source_name);
            } else {
                warn!("⚠️ {} is unhealthy", source_name);
            }

            // Update or create HealthStatus component
            let mut found = false;
            for mut status in health_status_query.iter_mut() {
                if status.source_name == source_name {
                    status.is_healthy = is_healthy;
                    status.last_check = chrono::Utc::now();
                    if !is_healthy {
                        status
                            .recent_errors
                            .push(format!("Health check failed at {}", chrono::Utc::now()));
                        // Keep only last 10 errors
                        if status.recent_errors.len() > 10 {
                            status.recent_errors.drain(0..1);
                        }
                    } else {
                        // Clear errors on successful check
                        status.recent_errors.clear();
                    }
                    found = true;
                    break;
                }
            }

            if !found {
                // Create new HealthStatus entity
                commands.spawn(HealthStatus {
                    source_name: source_name.clone(),
                    is_healthy,
                    last_check: chrono::Utc::now(),
                    recent_errors: Vec::new(),
                });
                info!("📊 Created health status entity for {}", source_name);
            }
        }
    }
}

/// System that polls command result channel and updates inverter state
pub fn poll_command_results(
    command_channel: Query<&InverterCommandChannel>,
    _current_mode_query: Query<(&Inverter, &mut CurrentMode)>,
) {
    let Ok(cmd_channel) = command_channel.single() else {
        return;
    };

    // NON-BLOCKING: check for command results
    while let Ok((inverter_id, result)) = cmd_channel.result_receiver.try_recv() {
        match result {
            Ok(()) => {
                info!("✅ Command succeeded for {}", inverter_id);

                // Find the inverter and update its current mode
                // Note: The actual mode update should happen in the system that sent the command
                // This just confirms the command was executed
            }
            Err(e) => {
                error!("❌ Command failed for {}: {}", inverter_id, e);
            }
        }
    }
}

/// System that periodically spawns tasks to poll inverter state
/// Uses non-blocking async tasks - results will be picked up by poll_inverter_state_results
pub fn spawn_inverter_state_polls(
    runtime: Res<AsyncRuntime>,
    inverter_source: Res<InverterDataSourceResource>,
    state_channel: Query<&InverterStateChannel>,
    inverters: Query<&Inverter>,
    mut last_poll: Local<Option<std::time::Instant>>,
) {
    use std::time::{Duration, Instant};

    // Rate limit: poll every 5 seconds
    let now = Instant::now();
    if let Some(last) = *last_poll
        && now.duration_since(last) < Duration::from_secs(5)
    {
        return;
    }
    *last_poll = Some(now);

    let Ok(channel) = state_channel.single() else {
        return;
    };

    // Spawn async task for each inverter
    for inverter in inverters.iter() {
        let inverter_id = inverter.id.clone();
        let source = inverter_source.0.clone();
        let tx = channel.sender.clone();

        debug!("📡 Spawning state poll for inverter: {}", inverter_id);

        runtime.spawn(async move {
            match source.read_state(&inverter_id).await {
                Ok(state) => {
                    debug!(
                        "✅ Retrieved state for {}: SOC={:.1}%",
                        inverter_id, state.battery_soc
                    );
                    let _ = tx.send((inverter_id, Ok(state)));
                }
                Err(e) => {
                    warn!("❌ Failed to read state for {}: {}", inverter_id, e);
                    let _ = tx.send((inverter_id, Err(e)));
                }
            }
        });
    }
}

/// System that processes inverter state results from the channel
pub fn poll_inverter_state_results(
    state_channel: Query<&InverterStateChannel>,
    mut commands: Commands,
    mut inverters: Query<(Entity, &Inverter, Option<&mut RawInverterState>)>,
) {
    use chrono::Utc;

    let Ok(channel) = state_channel.single() else {
        return;
    };

    // Process all available state updates
    while let Ok((inverter_id, result)) = channel.receiver.try_recv() {
        // Find the matching inverter entity
        for (entity, inverter, existing_state) in inverters.iter_mut() {
            if inverter.id == inverter_id {
                if let Ok(state) = result {
                    let raw_state = RawInverterState {
                        state,
                        last_updated: Utc::now(),
                    };

                    // Update or insert the RawInverterState component
                    if let Some(mut existing) = existing_state {
                        *existing = raw_state;
                    } else {
                        commands.entity(entity).insert(raw_state);
                    }
                }
                break;
            }
        }
    }
}

type InverterComponentsQuery<'a> = (
    Entity,
    &'a RawInverterState,
    // Core components
    Option<&'a mut BatteryStatus>,
    Option<&'a mut GridPower>,
    Option<&'a mut PowerGeneration>,
    Option<&'a mut InverterStatus>,
    // Extended components
    Option<&'a mut ExtendedPv>,
    Option<&'a mut EpsStatus>,
    Option<&'a mut BatteryExtended>,
    Option<&'a mut GridTotals>,
    Option<&'a mut ThreePhase>,
    Option<&'a mut Temperatures>,
);

/// System that decomposes RawInverterState into individual ECS components
/// This ensures BatteryStatus, GridPower, and PowerGeneration components are always up-to-date
/// Also populates extended components if data is available
/// Also collects battery SOC history for visualization
pub fn decompose_inverter_state(
    mut inverters: Query<InverterComponentsQuery>,
    mut commands: Commands,
    mut battery_history: ResMut<BatteryHistory>,
    mut pv_history: ResMut<PvHistory>,
    mut last_history_update: Local<Option<std::time::Instant>>,
) {
    for (
        entity,
        raw_state,
        battery,
        grid,
        pv,
        status,
        ext_pv,
        eps,
        bat_ext,
        grid_tot,
        three_phase,
        temps,
    ) in inverters.iter_mut()
    {
        let state = &raw_state.state;

        // Update or insert BatteryStatus
        let battery_status = BatteryStatus {
            soc_percent: state.battery_soc as u16,
            voltage_v: state.battery_voltage_v.unwrap_or(0.0),
            current_a: state.battery_current_a.unwrap_or(0.0),
            power_w: state.battery_power_w as i32,
            temperature_c: state.battery_temperature_c.unwrap_or(0.0),
            cycles: 0, // Not available in GenericInverterState
        };

        if let Some(mut existing) = battery {
            *existing = battery_status;
        } else {
            commands.entity(entity).insert(battery_status);
        }

        // Collect battery SOC history (every 15 minutes)
        let now = std::time::Instant::now();
        let should_collect = last_history_update
            .map(|last| now.duration_since(last).as_secs() >= 15 * 60)
            .unwrap_or(true);

        if should_collect {
            let history_point = BatteryHistoryPoint {
                timestamp: chrono::Utc::now(),
                soc: state.battery_soc,
                power_w: state.battery_power_w,
                voltage_v: Some(state.battery_voltage_v.unwrap_or(0.0)),
            };

            battery_history.add_point(history_point);

            // Also collect PV generation history at the same interval
            let pv_history_point = PvHistoryPoint {
                timestamp: chrono::Utc::now(),
                power_w: state.pv_power_w,
                pv1_power_w: state.pv1_power_w,
                pv2_power_w: state.pv2_power_w,
            };

            pv_history.add_point(pv_history_point);
            *last_history_update = Some(now);

            info!(
                "📊 Collected history: Battery {:.1}% ({:.0}W), PV {:.0}W (battery: {} pts, PV: {} pts)",
                state.battery_soc,
                state.battery_power_w,
                state.pv_power_w,
                battery_history.len(),
                pv_history.len()
            );
        } else {
            let elapsed_secs = last_history_update
                .map(|last| now.duration_since(last).as_secs())
                .unwrap_or(0);
            trace!(
                "⏱️ Battery history: {} points, next collection in {} seconds",
                battery_history.len(),
                (15_u64 * 60).saturating_sub(elapsed_secs)
            );
        }

        // Update or insert GridPower
        let grid_power = GridPower {
            export_power_w: state.grid_power_w as i32,
            grid_frequency_hz: state.inverter_frequency_hz.unwrap_or(0.0),
            grid_voltage_v: state.inverter_voltage_v.unwrap_or(0.0),
        };

        if let Some(mut existing) = grid {
            *existing = grid_power;
        } else {
            commands.entity(entity).insert(grid_power);
        }

        // Update or insert PowerGeneration
        let power_gen = PowerGeneration {
            current_power_w: state.pv_power_w as u16,
            daily_energy_kwh: state.today_solar_energy_kwh.unwrap_or(0.0),
            total_energy_kwh: state.total_solar_energy_kwh.unwrap_or(0.0),
            pv1_power_w: state.pv1_power_w.unwrap_or(0.0) as u16,
            pv2_power_w: state.pv2_power_w.unwrap_or(0.0) as u16,
        };

        if let Some(mut existing) = pv {
            *existing = power_gen;
        } else {
            commands.entity(entity).insert(power_gen);
        }

        // Update or insert InverterStatus
        let inv_status = InverterStatus {
            // Future: Map work_mode from GenericInverterState to RunMode enum
            // Currently GenericInverterState doesn't expose work_mode,
            // but vendor-specific state does. Consider adding to generic state.
            run_mode: RunMode::Normal,
            error_code: state.fault_code.unwrap_or(0),
            temperature_c: state.inverter_temperature_c.unwrap_or(0.0),
            last_update: Some(raw_state.last_updated),
            connection_healthy: state.online,
        };

        if let Some(mut existing) = status {
            *existing = inv_status;
        } else {
            commands.entity(entity).insert(inv_status);
        }

        // ============= Extended Components (Optional) =============

        // ExtendedPv - PV3/PV4 strings (if available)
        if state.pv3_power_w.is_some() || state.pv4_power_w.is_some() {
            let extended_pv = ExtendedPv {
                pv3_voltage_v: 0.0, // Not in GenericInverterState
                pv3_current_a: 0.0,
                pv3_power_w: state.pv3_power_w.unwrap_or(0.0) as u16,
                pv4_voltage_v: 0.0,
                pv4_current_a: 0.0,
                pv4_power_w: state.pv4_power_w.unwrap_or(0.0) as u16,
            };

            if let Some(mut existing) = ext_pv {
                *existing = extended_pv;
            } else {
                commands.entity(entity).insert(extended_pv);
            }
        }

        // EpsStatus - Emergency Power Supply (if available)
        if state.eps_voltage_v.is_some()
            || state.eps_current_a.is_some()
            || state.eps_power_w.is_some()
        {
            let eps_status = EpsStatus {
                voltage_v: state.eps_voltage_v.unwrap_or(0.0),
                current_a: state.eps_current_a.unwrap_or(0.0),
                power_w: state.eps_power_w.unwrap_or(0.0) as i16,
                frequency_hz: 0.0, // Not in GenericInverterState
            };

            if let Some(mut existing) = eps {
                *existing = eps_status;
            } else {
                commands.entity(entity).insert(eps_status);
            }
        }

        // BatteryExtended - BMS detailed data
        let battery_extended = BatteryExtended {
            output_energy_total_kwh: state.battery_output_energy_total_kwh.unwrap_or(0.0),
            output_energy_today_kwh: state.battery_output_energy_today_kwh.unwrap_or(0.0),
            input_energy_total_kwh: state.battery_input_energy_total_kwh.unwrap_or(0.0),
            input_energy_today_kwh: state.battery_input_energy_today_kwh.unwrap_or(0.0),
            pack_number: 0, // Not in GenericInverterState
            state_of_health_percent: state.battery_soh_percent.unwrap_or(100.0) as u16,
            bms_charge_max_current_a: state.bms_charge_max_current_a.unwrap_or(0.0),
            bms_discharge_max_current_a: state.bms_discharge_max_current_a.unwrap_or(0.0),
            bms_capacity_ah: 0, // Not in GenericInverterState
            board_temperature_c: state.board_temperature_c.unwrap_or(0.0),
            boost_temperature_c: state.boost_temperature_c.unwrap_or(0.0),
        };

        if let Some(mut existing) = bat_ext {
            *existing = battery_extended;
        } else {
            commands.entity(entity).insert(battery_extended);
        }

        // GridTotals - Lifetime energy totals
        let grid_totals = GridTotals {
            export_total_kwh: state.grid_export_total_kwh.unwrap_or(0.0),
            import_total_kwh: state.grid_import_total_kwh.unwrap_or(0.0),
            today_yield_kwh: state.today_yield_kwh.unwrap_or(0.0),
            total_yield_kwh: state.total_yield_kwh.unwrap_or(0.0),
        };

        if let Some(mut existing) = grid_tot {
            *existing = grid_totals;
        } else {
            commands.entity(entity).insert(grid_totals);
        }

        // ThreePhase - Per-phase data (if available)
        if state.l1_voltage_v.is_some()
            || state.l2_voltage_v.is_some()
            || state.l3_voltage_v.is_some()
        {
            let three_phase_data = ThreePhase {
                l1_voltage_v: state.l1_voltage_v.unwrap_or(0.0),
                l1_current_a: state.l1_current_a.unwrap_or(0.0),
                l1_power_w: state.l1_power_w.unwrap_or(0.0) as i16,
                l1_frequency_hz: 0.0, // Not in GenericInverterState
                l2_voltage_v: state.l2_voltage_v.unwrap_or(0.0),
                l2_current_a: state.l2_current_a.unwrap_or(0.0),
                l2_power_w: state.l2_power_w.unwrap_or(0.0) as i16,
                l2_frequency_hz: 0.0,
                l3_voltage_v: state.l3_voltage_v.unwrap_or(0.0),
                l3_current_a: state.l3_current_a.unwrap_or(0.0),
                l3_power_w: state.l3_power_w.unwrap_or(0.0) as i16,
                l3_frequency_hz: 0.0,
            };

            if let Some(mut existing) = three_phase {
                *existing = three_phase_data;
            } else {
                commands.entity(entity).insert(three_phase_data);
            }
        }

        // Temperatures - Consolidated temperature monitoring
        let temperatures = Temperatures {
            inverter_c: state.inverter_temperature_c.unwrap_or(0.0),
            battery_c: state.battery_temperature_c.unwrap_or(0.0),
            board_c: state.board_temperature_c.unwrap_or(0.0),
            boost_c: state.boost_temperature_c.unwrap_or(0.0),
        };

        if let Some(mut existing) = temps {
            *existing = temperatures;
        } else {
            commands.entity(entity).insert(temperatures);
        }
    }
}
