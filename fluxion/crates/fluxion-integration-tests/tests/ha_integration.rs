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

use fluxion_core::{InverterDataSource, PriceDataSource, VendorEntityMapper};
use fluxion_ha::{CzSpotPriceAdapter, HomeAssistantClient, HomeAssistantInverterAdapter};
use fluxion_solax::SolaxEntityMapper;
use std::sync::Arc;

/// Load HA token from .token.txt file (in workspace root)
fn load_token() -> Result<String, std::io::Error> {
    // Try workspace root first
    let workspace_root = std::env::var("CARGO_MANIFEST_DIR")
        .map(|p| {
            std::path::PathBuf::from(p)
                .parent()
                .unwrap()
                .parent()
                .unwrap()
                .to_path_buf()
        })
        .unwrap_or_else(|_| std::path::PathBuf::from("."));

    let token_path = workspace_root.join(".token.txt");
    std::fs::read_to_string(token_path)
        .or_else(|_| std::fs::read_to_string(".token.txt")) // Fallback to current dir
        .map(|s| s.trim().to_string())
}

#[tokio::test]
#[ignore] // Run with: cargo test --test integration_ha -- --ignored
async fn test_ha_connection() {
    let token = load_token().expect("Failed to read .token.txt");
    let base_url = "http://homeassistant.local:8123";

    let client = HomeAssistantClient::new(base_url, token).expect("Failed to create HA client");

    // Test basic connectivity
    let health = client.ping().await;
    assert!(health.is_ok(), "Failed to ping HA: {:?}", health.err());
    assert!(health.unwrap(), "HA health check returned false");

    println!(
        "✅ Successfully connected to Home Assistant at {}",
        base_url
    );
}

#[tokio::test]
#[ignore]
async fn test_read_single_entity() {
    let token = load_token().expect("Failed to read .token.txt");
    let base_url = "http://homeassistant.local:8123";

    let client = HomeAssistantClient::new(base_url, token).expect("Failed to create HA client");

    // Try to read sun entity (always available)
    let result = client.get_state("sun.sun").await;
    if let Err(e) = &result {
        eprintln!("Failed to read sun.sun: {:?}", e);
    }
    assert!(result.is_ok(), "Failed to read sun.sun entity");

    let state = result.unwrap();
    println!("✅ Successfully read sun.sun: {}", state.state);
}

#[tokio::test]
#[ignore]
async fn test_get_all_entities() {
    let token = load_token().expect("Failed to read .token.txt");
    let base_url = "http://homeassistant.local:8123";

    let client = HomeAssistantClient::new(base_url, token).expect("Failed to create HA client");

    // Get all entities to see what's available
    let states = client.get_all_states().await;
    if let Err(e) = &states {
        eprintln!("Failed to get all states: {:?}", e);
        eprintln!("This might be a permissions issue with the token.");
        eprintln!("Make sure your token has access to read states.");
    }
    assert!(
        states.is_ok(),
        "Failed to get all states: {:?}",
        states.err()
    );

    let states = states.unwrap();
    println!("📊 Total entities in HA: {}", states.len());

    // List Solax-related entities
    let solax_entities: Vec<_> = states
        .iter()
        .filter(|s| s.entity_id.contains("solax"))
        .collect();

    println!("\n🔌 Found {} Solax entities:", solax_entities.len());
    for entity in solax_entities.iter().take(20) {
        println!("  - {} = {}", entity.entity_id, entity.state);
    }

    // List price-related entities
    let price_entities: Vec<_> = states
        .iter()
        .filter(|s| s.entity_id.contains("price") || s.entity_id.contains("energy"))
        .collect();

    println!("\n💰 Found {} price/energy entities:", price_entities.len());
    for entity in price_entities.iter().take(20) {
        println!("  - {} = {}", entity.entity_id, entity.state);
    }
}

#[tokio::test]
#[ignore]
async fn test_solax_battery_reading() {
    let token = load_token().expect("Failed to read .token.txt");
    let base_url = "http://homeassistant.local:8123";

    let client =
        Arc::new(HomeAssistantClient::new(base_url, token).expect("Failed to create HA client"));

    // Try to find Solax battery SOC entity
    let states = client.get_all_states().await.expect("Failed to get states");
    let battery_entity = states
        .iter()
        .find(|s| s.entity_id.contains("battery") && s.entity_id.contains("capacity"))
        .or_else(|| {
            states
                .iter()
                .find(|s| s.entity_id.contains("battery") && s.entity_id.contains("soc"))
        })
        .expect("No battery SOC entity found");

    println!("🔋 Battery SOC Entity: {}", battery_entity.entity_id);
    println!("   Current SOC: {}%", battery_entity.state);

    // Try to parse as float
    let soc: f32 = battery_entity
        .state
        .parse()
        .expect("Failed to parse SOC as float");

    assert!((0.0..=100.0).contains(&soc), "SOC out of range: {}", soc);
    println!("✅ Successfully read battery SOC: {}%", soc);
}

#[tokio::test]
#[ignore]
async fn test_solax_inverter_adapter() {
    let token = load_token().expect("Failed to read .token.txt");
    let base_url = "http://homeassistant.local:8123";

    let client =
        Arc::new(HomeAssistantClient::new(base_url, token).expect("Failed to create HA client"));

    // Get all states to find the inverter entity prefix
    let states = client.get_all_states().await.expect("Failed to get states");
    let battery_entity = states
        .iter()
        .find(|s| s.entity_id.contains("battery") && s.entity_id.contains("capacity"))
        .expect("No battery entity found");

    // Extract prefix from entity_id (e.g., "sensor.solax_battery_capacity" -> "solax")
    let parts: Vec<&str> = battery_entity.entity_id.split('_').collect();
    let inverter_prefix = parts
        .first()
        .and_then(|s| s.split('.').nth(1))
        .expect("Could not extract inverter prefix");

    println!("🔌 Using inverter prefix: {}", inverter_prefix);

    // Create adapter with Solax mapper
    let mapper: Arc<dyn VendorEntityMapper> = Arc::new(SolaxEntityMapper::new());
    let adapter = HomeAssistantInverterAdapter::new(client, mapper);

    // Test reading inverter state
    let state = adapter.read_state(inverter_prefix).await;

    match state {
        Ok(state) => {
            println!("✅ Successfully read inverter state:");
            println!("   Battery SOC: {}%", state.battery_soc);
            println!("   Work Mode: {:?}", state.work_mode);
            println!("   Grid Power: {}W", state.grid_power_w);
            println!("   Battery Power: {}W", state.battery_power_w);
            println!("   PV Power: {}W", state.pv_power_w);
            println!("   Online: {}", state.online);
        }
        Err(e) => {
            println!("❌ Failed to read inverter state: {:?}", e);
            panic!("Integration test failed");
        }
    }
}

#[tokio::test]
#[ignore]
async fn test_cz_spot_price_adapter() {
    let token = load_token().expect("Failed to read .token.txt");
    let base_url = "http://homeassistant.local:8123";

    let client =
        Arc::new(HomeAssistantClient::new(base_url, token).expect("Failed to create HA client"));

    // Try to find Czech spot price entity
    let states = client.get_all_states().await.expect("Failed to get states");
    let price_entity = states
        .iter()
        .find(|s| s.entity_id.contains("spot") && s.entity_id.contains("price"))
        .or_else(|| {
            states
                .iter()
                .find(|s| s.entity_id.contains("cz") && s.entity_id.contains("energy"))
        })
        .expect("No spot price entity found. Available entities listed in test_get_all_entities");

    println!("💰 Spot Price Entity: {}", price_entity.entity_id);

    // Create adapter
    let adapter = CzSpotPriceAdapter::new(client, price_entity.entity_id.clone());

    // Test reading prices
    let prices = adapter.read_prices().await;

    match prices {
        Ok(prices) => {
            println!("✅ Successfully read spot prices:");
            println!("   Total blocks: {}", prices.time_block_prices.len());
            println!(
                "   Block duration: {} minutes",
                prices.block_duration_minutes
            );
            println!("   Fetched at: {}", prices.fetched_at);
            println!("   HA last updated: {}", prices.ha_last_updated);

            if !prices.time_block_prices.is_empty() {
                let first = &prices.time_block_prices[0];
                let last = prices.time_block_prices.last().unwrap();
                println!(
                    "   First block: {} CZK/kWh at {}",
                    first.price_czk_per_kwh, first.block_start
                );
                println!(
                    "   Last block: {} CZK/kWh at {}",
                    last.price_czk_per_kwh, last.block_start
                );
            }

            assert!(
                !prices.time_block_prices.is_empty(),
                "No price blocks received"
            );
            assert!(
                prices.block_duration_minutes == 15 || prices.block_duration_minutes == 60,
                "Unexpected block duration: {}",
                prices.block_duration_minutes
            );
        }
        Err(e) => {
            println!("❌ Failed to read spot prices: {:?}", e);
            panic!("Integration test failed");
        }
    }
}

#[tokio::test]
#[ignore]
async fn test_end_to_end_data_flow() {
    println!("\n🚀 Running end-to-end data flow test...\n");

    let token = load_token().expect("Failed to read .token.txt");
    let base_url = "http://homeassistant.local:8123";

    let client =
        Arc::new(HomeAssistantClient::new(base_url, token).expect("Failed to create HA client"));

    // Discover entities
    let states = client.get_all_states().await.expect("Failed to get states");

    let battery_entity = states
        .iter()
        .find(|s| s.entity_id.contains("battery") && s.entity_id.contains("capacity"))
        .expect("No battery entity found");

    let inverter_prefix = battery_entity
        .entity_id
        .split('_')
        .next()
        .and_then(|s| s.split('.').nth(1))
        .expect("Could not extract inverter prefix");

    println!("📋 Step 1: Read inverter state");
    let mapper: Arc<dyn VendorEntityMapper> = Arc::new(SolaxEntityMapper::new());
    let inverter_adapter = HomeAssistantInverterAdapter::new(client.clone(), mapper);
    let inverter_state = inverter_adapter
        .read_state(inverter_prefix)
        .await
        .expect("Failed to read inverter state");

    println!("   ✅ Current SOC: {}%", inverter_state.battery_soc);
    println!("   ✅ Current Mode: {:?}", inverter_state.work_mode);

    // Find price entity
    let price_entity = states
        .iter()
        .find(|s| s.entity_id.contains("spot") && s.entity_id.contains("price"))
        .or_else(|| {
            states
                .iter()
                .find(|s| s.entity_id.contains("cz") && s.entity_id.contains("energy"))
        });

    if let Some(price_entity) = price_entity {
        println!("\n📋 Step 2: Read spot prices");
        let price_adapter = CzSpotPriceAdapter::new(client.clone(), price_entity.entity_id.clone());
        let spot_prices = price_adapter
            .read_prices()
            .await
            .expect("Failed to read spot prices");

        println!(
            "   ✅ Received {} price blocks",
            spot_prices.time_block_prices.len()
        );

        println!("\n📋 Step 3: Analyze prices");
        let analysis = fluxion_core::analyze_prices(
            &spot_prices.time_block_prices,
            4,    // 4 hours charge
            2,    // 2 hours discharge
            true, // use spot for buy
            true, // use spot for sell
        );

        println!(
            "   ✅ Identified {} charge blocks",
            analysis.charge_blocks.len()
        );
        println!(
            "   ✅ Identified {} discharge blocks",
            analysis.discharge_blocks.len()
        );
        println!(
            "   ✅ Price range: {:.2} - {:.2} CZK/kWh",
            analysis.price_range.min_czk_per_kwh, analysis.price_range.max_czk_per_kwh
        );

        println!("\n📋 Step 4: Generate schedule");
        let schedule_config = fluxion_core::ScheduleConfig {
            min_battery_soc: 10.0,
            max_battery_soc: 100.0,
            target_inverters: Vec::new(), // Empty = all inverters
            display_currency: fluxion_core::Currency::EUR,
        };

        let schedule = fluxion_core::generate_schedule(
            &spot_prices.time_block_prices,
            &analysis,
            &schedule_config,
            None, // No i18n in tests
        );

        println!(
            "   ✅ Generated schedule with {} blocks",
            schedule.scheduled_blocks.len()
        );

        // Show first few schedule entries
        for (i, block) in schedule.scheduled_blocks.iter().take(5).enumerate() {
            println!("      Block {}: {:?} - {}", i, block.mode, block.reason);
        }

        println!("\n✅ End-to-end data flow test completed successfully!");
    } else {
        println!("\n⚠️  No price entity found, skipping price-related steps");
        println!("✅ Inverter reading test passed");
    }
}
