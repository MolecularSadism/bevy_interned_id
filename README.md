# `bevy_interned_id`

[![CI](https://github.com/MolecularSadism/bevy_interned_id/workflows/CI/badge.svg)](https://github.com/MolecularSadism/bevy_interned_id/actions)
[![Crates.io](https://img.shields.io/crates/v/bevy_interned_id.svg)](https://crates.io/crates/bevy_interned_id)
[![Docs.rs](https://docs.rs/bevy_interned_id/badge.svg)](https://docs.rs/bevy_interned_id)
[![License](https://img.shields.io/badge/license-MIT%2FApache--2.0-blue.svg)](https://github.com/MolecularSadism/bevy_interned_id#license)
[![Bevy](https://img.shields.io/badge/Bevy-0.19-blue.svg)](https://bevyengine.org/)
[![Rust](https://img.shields.io/badge/rust-2024%20edition-orange.svg)](https://www.rust-lang.org/)

Ergonomic, type-safe ID types built on Bevy's string interning, with reflection
and serde generated for you.

## What this crate adds

Bevy already ships string interning: `bevy::ecs::intern::Interned<str>` gives you
`Copy` IDs with O(1) pointer comparison, pointer hashing, and automatic
deduplication. **Those performance properties come from Bevy, not from this
crate.** What `bevy_interned_id` adds is the per-ID-type boilerplate you'd
otherwise hand-write on top of that primitive:

- **A one-liner declaration**: `interned_id!(pub SpellId);` — no newtype, no
  manual derive list, no spelling out `Interned<str>`.
- **Type safety**: distinct types (`SpellId` vs `ItemId`) that can't be mixed up,
  each with its own interner.
- **Full Bevy reflection**: the complete
  `Reflect`/`PartialReflect`/`Typed`/`TypePath`/`FromReflect`/`GetTypeRegistration`
  hierarchy — the part you *can't* `#[derive(Reflect)]` on an interned pointer and
  would otherwise write (and maintain across Bevy versions) by hand.
- **serde**: `Serialize`/`Deserialize` as a plain string.
- **Ergonomics**: `new`, `as_str`, `Display`, `Deref<Target = str>`, `From`, `Default`.
- **Inspector UI**: read-only display in bevy-inspector-egui (with the `dev` feature).

## What is String Interning?

String interning is a technique where identical strings are stored only once in memory. When you create an interned string, the system checks if that string already exists. If it does, you get a reference to the existing string; otherwise, a new one is created. This makes:

- **String comparison extremely fast**: Just compare pointers instead of comparing each character
- **Memory usage lower**: No duplicate strings in memory
- **IDs perfect for games**: Great for asset IDs, event types, configuration keys, etc.

## Installation

Add this to your `Cargo.toml`:

```toml
[dependencies]
bevy_interned_id = "0.5"
bevy = "0.19"
serde = "1" # needed by the default `serde` feature
```

`bevy_interned_id` is a pure proc-macro crate: it adds **no** Bevy or serde
dependency to your tree. The generated code resolves `bevy::…` (and, with the
`serde` feature, `serde::…`) paths at the expansion site, so those crates must
be direct dependencies of the crate that declares the ID types. If you depend
on `bevy_ecs`/`bevy_reflect` instead of the full `bevy` facade, see
[Using without the full `bevy` facade](#using-without-the-full-bevy-facade).

### Feature flags

| Feature | Default | Effect |
|---------|---------|--------|
| `serde` | ✅ | Emit `Serialize`/`Deserialize` impls (serialize as a plain string). Requires `serde` in your dependencies. |
| `dev` | ❌ | Emit a read-only `InspectorPrimitive` impl. Requires `bevy-inspector-egui` in your dependencies. |

## Quick Start

```rust
use bevy_interned_id::interned_id;
use bevy::prelude::*;

// Define your ID type — one line, no boilerplate.
interned_id!(pub SpellId);

// Use it
let fireball = SpellId::new("fireball");
let ice_bolt = SpellId::new("ice_bolt");

// Fast comparison (pointer equality, courtesy of Bevy's interner)
assert_eq!(fireball, SpellId::new("fireball"));
assert_ne!(fireball, ice_bolt);

// Access the string value
println!("Casting: {}", fireball); // Prints: "Casting: fireball"
assert_eq!(fireball.as_str(), "fireball");
```

`interned_id!(pub SpellId)` simply expands to the lower-level derive form below,
which you can still write by hand when you need a different struct shape:

```rust
use bevy_interned_id::InternedId;
use bevy::prelude::*;

#[derive(InternedId, Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct SpellId(bevy::ecs::intern::Interned<str>);
```

## Usage Examples

### As ECS Component

```rust
use bevy_interned_id::interned_id;
use bevy::prelude::*;

// Attributes placed before the visibility are forwarded to the generated
// struct, so `#[derive(Component)]` makes the ID a first-class component.
interned_id!(#[derive(Component)] pub ItemId);

fn spawn_item(mut commands: Commands) {
    commands.spawn((
        ItemId::new("health_potion"),
        Transform::default(),
    ));
}

fn query_items(q_items: Query<&ItemId>) {
    for item_id in &q_items {
        println!("Found item: {}", item_id);
    }
}
```

### With HashMap/HashSet

```rust
use std::collections::HashMap;
use bevy_interned_id::InternedId;
use bevy::prelude::*;

#[derive(InternedId, Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct EnemyId(bevy::ecs::intern::Interned<str>);

let mut enemy_hp: HashMap<EnemyId, u32> = HashMap::new();
enemy_hp.insert(EnemyId::new("goblin"), 50);
enemy_hp.insert(EnemyId::new("dragon"), 500);

assert_eq!(enemy_hp[&EnemyId::new("goblin")], 50);
```

### Serialization

```rust
use bevy_interned_id::InternedId;
use bevy::prelude::*;
use serde::{Serialize, Deserialize};

// `Serialize` and `Deserialize` are generated by `InternedId` itself, so you do
// NOT list them in the derive for the ID type.
#[derive(InternedId, Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct QuestId(bevy::ecs::intern::Interned<str>);

#[derive(Serialize, Deserialize)]
struct SaveData {
    active_quest: QuestId,
    completed_quests: Vec<QuestId>,
}

let save = SaveData {
    active_quest: QuestId::new("main_quest"),
    completed_quests: vec![QuestId::new("tutorial"), QuestId::new("fetch_quest")],
};

// Serializes as JSON:
// {
//   "active_quest": "main_quest",
//   "completed_quests": ["tutorial", "fetch_quest"]
// }
let json = serde_json::to_string(&save).unwrap();
assert_eq!(
    json,
    r#"{"active_quest":"main_quest","completed_quests":["tutorial","fetch_quest"]}"#
);
```

### With Match and Deref

```rust
use bevy_interned_id::InternedId;
use bevy::prelude::*;

#[derive(InternedId, Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct DamageType(bevy::ecs::intern::Interned<str>);

fn calculate_damage(damage_type: DamageType, base_damage: f32) -> f32 {
    // Can use as &str through Deref
    match &*damage_type {
        "fire" => base_damage * 1.5,
        "ice" => base_damage * 1.2,
        "lightning" => base_damage * 1.8,
        _ => base_damage,
    }
}
```

## Using without the full `bevy` facade

Library crates often depend on `bevy_ecs`/`bevy_reflect` directly instead of
the `bevy` facade. Because the generated code resolves `bevy::…` paths at the
expansion site, a three-line facade module makes the macro work there too —
this crate's own tests, example, and benchmarks use exactly this pattern:

```rust
// Facade so the generated `bevy::…` paths resolve to the sub-crates.
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

use bevy_interned_id::interned_id;

interned_id!(pub SpellId);

fn main() {
    let id = SpellId::new("fireball");
    assert_eq!(id.as_str(), "fireball");
}
```

## Generated API

Whether you declare the type with `interned_id!(pub MyId);` or with
`#[derive(InternedId)]` directly, you get:

### Methods
- `MyId::new(s: &str) -> Self` - Create ID from string (interns automatically)
- `id.as_str() -> &'static str` - Get the string value

### Trait Implementations
- `Display` - Format as the string value
- `From<&str>` and `From<String>` - Convenient conversions
- `Deref<Target = str>` - Use as string slice with deref coercion
- `Default` - Empty string default
- `Serialize`, `Deserialize` - Serde support (as string; `serde` feature, on by default)
- Full Bevy reflection hierarchy

The per-type interner static itself is emitted inside an anonymous `const`
block, so it never appears in your module's namespace.

### Declaring the type
- **Recommended**: `interned_id!(pub MyId);` writes the newtype, the
  `Interned<str>` field, and all six required derives for you. Pass extra
  attributes before the visibility to forward them, e.g.
  `interned_id!(#[derive(Component)] pub MyId);`.
- **Manual**: with `#[derive(InternedId)]` directly you must also add
  `Clone`, `Copy`, `PartialEq`, `Eq`, `Hash`, `Debug` yourself, and wrap
  `bevy::ecs::intern::Interned<str>` explicitly.

## Use Cases

Perfect for:
- **Asset identifiers**: `SpellId`, `ItemId`, `EnemyId`, `SoundId`
- **Configuration keys**: Settings, feature flags
- **Event types**: `GameEvent`, `NetworkMessage`
- **State machine states**: `PlayerState`, `AIState`
- **Any string-based identifier needing frequent comparison**

## Performance

String interning provides significant performance benefits for ID types:

- **Comparison**: O(1) pointer comparison instead of O(n) string comparison
- **Memory**: Shared storage for duplicate strings
- **Hashing**: Hash the pointer instead of the string content
- **Copy**: Copy a single pointer instead of string data

These claims are measurable: `cargo bench --bench id_performance` compares
interned IDs against `String` for equality, hashing, and `HashMap` lookup.

## Bevy Integration

The generated types work seamlessly with Bevy's systems:

- **Reflection**: Full support for Bevy's reflection system
- **Inspector**: Read-only display in bevy-inspector-egui (with `dev` feature)
- **Serialization**: Works with Bevy scenes and save systems
- **Type Registration**: Automatic registration with `ReflectDefault`

## Best Practices

1. **One interner per ID type**: Each ID type gets its own interner (no cross-contamination)
2. **Use for identifiers**: Best for values compared frequently, not for arbitrary user text
3. **Not for dynamic content**: Interned strings live for the program lifetime
4. **Type safety**: Create separate types (`SpellId`, `ItemId`) instead of generic `Id` type

## Comparison with Alternatives

| Approach | Comparison | Memory | Type Safety | Comment |
|----------|------------|--------|-------------|---------|
| `String` | O(n) | High (duplicates) | Low | Each call site stores a duplicate |
| `&'static str` | O(n) | Low | Low | |
| `enum` | O(1) | Lowest | Highest | Changes require recompilation |
| `InternedId` | O(1) | Low | High | Hot reloadable |

The `InternedId` row's O(1) comparison and low memory are inherited from Bevy's
`Interned<str>`; this crate layers type safety, reflection, serde, and the
`interned_id!` ergonomics on top. Choose it when you need the flexibility of
strings with the performance of enums.

## Bevy Version Compatibility

| `bevy_interned_id` | Bevy |
|-------------------|------|
| 0.3, 0.4, 0.5     | 0.18, 0.19 |
| 0.2               | 0.17 |
| 0.1               | 0.16 |

The Bevy column is the version each release is developed and tested against.
Because this is a pure proc-macro crate, it adds no Bevy dependency of its
own.

### Migration from 0.4 to 0.5

No changes to the generated API. Points to be aware of:

- **`serde` became a default cargo feature.** Builds with default features are
  unchanged. If you use `default-features = false`, add `features = ["serde"]`
  back to keep the `Serialize`/`Deserialize` impls.
- **The per-type interner static is no longer visible** in the declaring
  module (it is emitted inside an anonymous `const` block). Code referencing
  the undocumented `<TYPENAME>_INTERNER` static directly must switch to the
  public `new`/`as_str` API.
- The generated code now uses fully-qualified `::std` paths, so ID types can
  be declared in modules that shadow names like `fmt` or `ops`.

### New: the `interned_id!` macro

The `interned_id!` declarative macro is **additive** — it is the recommended way
to declare an ID type, but existing `#[derive(InternedId)]` code is unchanged and
keeps working exactly as before. `interned_id!(pub Foo);` expands to the derive
form, so the two are fully interchangeable.

### Migration from 0.3 to 0.4

The 0.4 release renames the crate and updates compatibility from Bevy 0.18 to
Bevy 0.19. Key points:

- **Crate renamed** from `msg_interned_id` to `bevy_interned_id`. Rename the
  dependency in your `Cargo.toml` and update imports from
  `use msg_interned_id::InternedId;` to `use bevy_interned_id::InternedId;`.
- **No breaking changes** to the public API of generated types. The `new`,
  `as_str`, `Display`, `From`, `Deref`, `Default`, serde, and reflection
  surfaces are unchanged.
- **Bevy 0.19 reflection reorganization**: `bevy_reflect` moved many items into
  kind-specific modules at the crate root. The paths the macro emits
  (`bevy::reflect::TypeInfo`, `bevy::reflect::OpaqueInfo`,
  `bevy::reflect::utility::NonGenericTypeInfoCell`, and the
  `PartialReflect`/`Reflect` hierarchy) are still re-exported, so generated code
  continues to compile without changes.
- **Interned string requirement**: Bevy 0.19 requires the interned type to
  implement `Internable`. `str` satisfies this out of the box, so
  `Interned<str>` newtypes are unaffected.
- **Requires Rust 1.95 or newer**, as mandated by Bevy 0.19.

To migrate, update your `Cargo.toml`:

```toml
bevy_interned_id = "0.4"
bevy = "0.19"
```

### Migration from 0.2 to 0.3

The 0.3 release updates compatibility from Bevy 0.17 to Bevy 0.18. Key changes:

- **No breaking changes** to the public API of generated types
- **`dev` feature**: The optional `bevy-inspector-egui` inspector integration is now
  gated behind a proper `dev` feature on this crate rather than emitting
  `#[cfg(feature = "dev")]` into user code. Enable it with:

  ```toml
  bevy_interned_id = { version = "0.3", features = ["dev"] }
  ```

To migrate, simply update your `Cargo.toml`:

```toml
bevy_interned_id = "0.3"
bevy = "0.18"
```

### Migration from 0.1 to 0.2

The 0.2 release updates compatibility from Bevy 0.16 to Bevy 0.17. Key changes:

- **Reflection API**: Updated to use Bevy 0.17's reflection system
  - Removed deprecated `clone_value` method from `PartialReflect`
  - Added `reflect_clone` method for proper cloning support
- **No breaking changes** to the public API of generated types

To migrate, simply update your `Cargo.toml`:

```toml
bevy_interned_id = "0.2"
bevy = "0.17"
```

## License

Licensed under either of:

- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE))
- MIT license ([LICENSE-MIT](LICENSE-MIT))

at your option.

## Contributing

Contributions are welcome! This crate is part of the [MolecularSadism](https://github.com/MolecularSadism) game development libraries.
