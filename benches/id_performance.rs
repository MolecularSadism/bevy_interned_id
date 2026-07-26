//! Benchmarks backing the performance claims in the README: O(1) equality,
//! pointer hashing, and cheap `HashMap` lookups compared to `String` keys.
//!
//! Run with: `cargo bench --bench id_performance`

// Facade module mirroring the `bevy` crate structure, so the generated code
// (which resolves `bevy::*` paths at the expansion site) builds against the
// individual `bevy_ecs`/`bevy_reflect` dev-dependencies. This is the same
// pattern a downstream crate without the full `bevy` facade would use.
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

use std::collections::HashMap;
use std::hash::{BuildHasher, RandomState};

use bevy::prelude::*;
use bevy_interned_id::interned_id;
use criterion::{Criterion, black_box, criterion_group, criterion_main};

interned_id!(pub BenchId);

/// A realistically long identifier where O(n) string comparison costs show up.
const LONG_ID: &str = "environment_forest_autumn_deciduous_tree_variant_07_windy_lod2";

fn bench_intern(c: &mut Criterion) {
    let mut group = c.benchmark_group("intern");

    // Interning a string that is already in the interner (the steady-state
    // path: IDs are created from a fixed vocabulary at runtime).
    let _ = BenchId::new(LONG_ID);
    group.bench_function("hit_long", |b| {
        b.iter(|| BenchId::new(black_box(LONG_ID)));
    });

    group.finish();
}

fn bench_equality(c: &mut Criterion) {
    let mut group = c.benchmark_group("equality");

    let a = BenchId::new(LONG_ID);
    let b_id = BenchId::new(LONG_ID);
    group.bench_function("interned_id", |b| {
        b.iter(|| black_box(a) == black_box(b_id));
    });

    let s1 = LONG_ID.to_string();
    let s2 = LONG_ID.to_string();
    group.bench_function("string", |b| {
        b.iter(|| black_box(&s1) == black_box(&s2));
    });

    group.finish();
}

fn bench_hashing(c: &mut Criterion) {
    let mut group = c.benchmark_group("hash");
    let state = RandomState::new();

    let id = BenchId::new(LONG_ID);
    group.bench_function("interned_id", |b| {
        b.iter(|| state.hash_one(black_box(id)));
    });

    let s = LONG_ID.to_string();
    group.bench_function("string", |b| {
        b.iter(|| state.hash_one(black_box(&s)));
    });

    group.finish();
}

fn bench_hashmap_lookup(c: &mut Criterion) {
    let mut group = c.benchmark_group("hashmap_lookup");

    // 1000 entries with a shared long prefix, as asset-style IDs tend to have.
    let keys: Vec<String> = (0..1000).map(|i| format!("{LONG_ID}_{i:04}")).collect();

    let id_map: HashMap<BenchId, u32> = keys
        .iter()
        .enumerate()
        .map(|(i, k)| (BenchId::new(k), i as u32))
        .collect();
    let probe_id = BenchId::new(&keys[500]);
    group.bench_function("interned_id_key", |b| {
        b.iter(|| id_map.get(&black_box(probe_id)));
    });

    let string_map: HashMap<String, u32> = keys
        .iter()
        .enumerate()
        .map(|(i, k)| (k.clone(), i as u32))
        .collect();
    let probe_string = keys[500].clone();
    group.bench_function("string_key", |b| {
        b.iter(|| string_map.get(black_box(&probe_string)));
    });

    group.finish();
}

criterion_group!(
    benches,
    bench_intern,
    bench_equality,
    bench_hashing,
    bench_hashmap_lookup
);
criterion_main!(benches);
