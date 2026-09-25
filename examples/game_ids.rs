//! Example demonstrating `bevy_interned_id` usage in a game-like scenario.
//!
//! This example shows how to use interned string IDs for game entities,
//! spells, and items with full Bevy ECS integration.
//!
//! Run with: `cargo run --example game_ids`
//!
//! Note: This example uses minimal Bevy dependencies and runs without a window,
//! making it suitable for CI and headless environments.

// Create a facade module to make the generated code work with individual crates
mod bevy {
    pub mod ecs {
        pub mod intern {
            pub use bevy_ecs::intern::*;
        }
    }
    pub mod reflect {
        pub use bevy_reflect::*;
    }
    pub mod prelude {
        pub use bevy_ecs::prelude::*;
        pub use bevy_reflect::prelude::*;
    }
}

use bevy::prelude::*;
use bevy_ecs::world::World;
use bevy_interned_id::{InternedId, interned_id};
use std::collections::HashMap;

// Define various ID types for a game.

// The `interned_id!` one-liner is the recommended way to declare an ID type:
// it writes the newtype, the `Interned<str>` field, and the required derives
// for you. Attributes before the visibility (like `#[derive(Component)]`) are
// forwarded to the generated struct.
interned_id!(#[derive(Component)] pub ItemId); // items in the player's inventory
interned_id!(#[derive(Component)] pub EnemyId); // enemy types

// For contrast, `SpellId` is written with the lower-level `#[derive(InternedId)]`
// directly — the exact form the macro above expands to.
/// Unique identifier for spells in the game.
#[derive(InternedId, Clone, Copy, PartialEq, Eq, Debug)]
pub struct SpellId(bevy::ecs::intern::Interned<str>);

/// Component marking an entity's health.
#[derive(Component)]
pub struct Health(pub u32);

/// Component marking an entity's damage output.
#[derive(Component)]
pub struct Damage(pub u32);

/// Data associated with a spell.
pub struct SpellData {
    pub damage: u32,
    pub mana_cost: u32,
    pub element: &'static str,
}

fn main() {
    println!("=== bevy_interned_id Game Example ===\n");

    // Demonstrate basic ID creation and comparison
    demo_basic_ids();

    // Demonstrate using IDs in a HashMap
    demo_spell_registry();

    // Demonstrate pattern matching with deref
    demo_pattern_matching();

    // Demonstrate serialization
    demo_serialization();

    // Demonstrate ECS integration
    demo_ecs_integration();

    println!("\n=== All demonstrations complete! ===");
}

fn demo_basic_ids() {
    println!("--- Basic ID Creation ---");

    // Create IDs from strings
    let fireball = SpellId::new("fireball");
    let ice_bolt = SpellId::new("ice_bolt");
    let fireball2 = SpellId::new("fireball");

    // IDs can be displayed
    println!("Created spell: {fireball}");
    println!("Created spell: {ice_bolt}");

    // Same string values produce equal IDs (pointer equality)
    assert_eq!(fireball, fireball2);
    println!("fireball == fireball2: {}", fireball == fireball2);

    // Different strings produce different IDs
    assert_ne!(fireball, ice_bolt);
    println!("fireball == ice_bolt: {}", fireball == ice_bolt);

    // Access the underlying string
    println!("Spell name: {}", fireball.as_str());

    // IDs can be used as string slices via Deref
    assert!(fireball.starts_with("fire"));
    println!(
        "fireball starts with 'fire': {}",
        fireball.starts_with("fire")
    );

    // Default creates an empty ID
    let empty_id = SpellId::default();
    println!("Default ID is empty: '{}'", empty_id.as_str());

    println!();
}

fn demo_spell_registry() {
    println!("--- Spell Registry (HashMap) ---");

    let mut spells: HashMap<SpellId, SpellData> = HashMap::new();

    // Register some spells
    spells.insert(
        SpellId::new("fireball"),
        SpellData {
            damage: 50,
            mana_cost: 20,
            element: "fire",
        },
    );

    spells.insert(
        SpellId::new("ice_bolt"),
        SpellData {
            damage: 35,
            mana_cost: 15,
            element: "ice",
        },
    );

    spells.insert(
        SpellId::new("lightning_strike"),
        SpellData {
            damage: 75,
            mana_cost: 40,
            element: "lightning",
        },
    );

    // Look up spells by ID
    let fireball_id = SpellId::new("fireball");
    if let Some(spell) = spells.get(&fireball_id) {
        println!(
            "Fireball: {} damage, {} mana, {} element",
            spell.damage, spell.mana_cost, spell.element
        );
    }

    // Iterate over all spells
    println!("\nAll registered spells:");
    for (id, data) in &spells {
        println!("  - {}: {} damage", id, data.damage);
    }

    println!();
}

fn demo_pattern_matching() {
    println!("--- Pattern Matching ---");

    let spell_id = SpellId::new("fireball");

    // Use Deref to match against string patterns
    let damage_multiplier = match &*spell_id {
        "fireball" => 1.5,
        "ice_bolt" => 1.2,
        "lightning_strike" => 1.8,
        _ => 1.0,
    };

    println!("Spell '{spell_id}' has damage multiplier: {damage_multiplier}");

    // Also works in if-let patterns
    if spell_id.as_str() == "fireball" {
        println!("It's a fireball! Watch out for fire damage.");
    }

    println!();
}

#[cfg(not(feature = "serde"))]
fn demo_serialization() {
    println!("--- Serialization (skipped: `serde` feature disabled) ---\n");
}

#[cfg(feature = "serde")]
fn demo_serialization() {
    println!("--- Serialization ---");

    let spell_id = SpellId::new("ancient_magic");

    // Serialize to JSON
    let json = serde_json::to_string(&spell_id).expect("Failed to serialize");
    println!("Serialized: {json}");

    // Deserialize from JSON
    let deserialized: SpellId = serde_json::from_str(&json).expect("Failed to deserialize");
    println!("Deserialized: {deserialized}");

    // Verify roundtrip
    assert_eq!(spell_id, deserialized);
    println!("Roundtrip successful: {}", spell_id == deserialized);

    // Serialize a collection
    let spell_ids = vec![
        SpellId::new("spell_a"),
        SpellId::new("spell_b"),
        SpellId::new("spell_c"),
    ];
    let json_array = serde_json::to_string(&spell_ids).expect("Failed to serialize array");
    println!("Serialized array: {json_array}");

    println!();
}

fn demo_ecs_integration() {
    println!("--- ECS Integration ---");

    // Create a minimal Bevy world
    let mut world = World::new();

    // Spawn some enemies with IDs
    let goblin_entity = world
        .spawn((EnemyId::new("goblin"), Health(50), Damage(10)))
        .id();

    let dragon_entity = world
        .spawn((EnemyId::new("dragon"), Health(500), Damage(75)))
        .id();

    world.spawn((EnemyId::new("skeleton"), Health(30), Damage(15)));

    // Query entities by their ID
    let mut query = world.query::<(&EnemyId, &Health, &Damage)>();

    println!("Spawned enemies:");
    for (enemy_id, health, damage) in query.iter(&world) {
        println!("  - {}: {} HP, {} damage", enemy_id, health.0, damage.0);
    }

    // Get a specific entity's ID
    if let Some(id) = world.get::<EnemyId>(goblin_entity) {
        println!("\nGoblin entity has ID: {id}");
    }

    if let Some(id) = world.get::<EnemyId>(dragon_entity) {
        println!("Dragon entity has ID: {id}");
    }

    // Spawn some items
    world.spawn(ItemId::new("health_potion"));
    world.spawn(ItemId::new("mana_potion"));
    world.spawn(ItemId::new("sword_of_flames"));

    let mut item_query = world.query::<&ItemId>();
    println!("\nSpawned items:");
    for item_id in item_query.iter(&world) {
        println!("  - {item_id}");
    }

    println!();
}
