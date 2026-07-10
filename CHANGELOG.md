# Changelog

All notable changes to this project are documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.4.0] - 2026-07-10

Renamed the crate to **`bevy_interned_id`** and bumped compatibility from Bevy
0.18 to **Bevy 0.19**. This is the first release under the `bevy_interned_id`
name.

### Changed

- **Renamed the crate from `msg_interned_id` to `bevy_interned_id`** to make its
  Bevy focus explicit. Update your dependency name in `Cargo.toml` and your
  imports from `use msg_interned_id::InternedId;` to
  `use bevy_interned_id::InternedId;`. The derive macro, generated API, and all
  behavior are otherwise unchanged.
- Updated the development/target Bevy version to `0.19`
  (`bevy_ecs` and `bevy_reflect` dev-dependencies bumped from `0.18` to `0.19`).
- Now requires **Rust 1.95 or newer**, as mandated by Bevy 0.19.
- United the crate-level rustdoc with `README.md`: `src/lib.rs` now pulls its
  landing-page documentation from the README via
  `#![doc = include_str!("../README.md")]`, giving a single source of truth for
  both crates.io and docs.rs.

### Added

- The Rust examples in `README.md` are now compiled and executed as doc tests.
  A `bevy` dev-dependency (with `default-features = false`, so no windowing,
  render, or system dependencies are pulled in) was added so the README's
  `bevy::*` examples build exactly as a downstream user's would.
- The `InternedId` derive macro's own doc examples are now runnable doc tests
  instead of `ignore`d snippets.
- New `bevy_0_19_migration` test module covering the surfaces most likely to be
  affected by the upgrade, per the Bevy 0.18 -> 0.19 migration guide:
  - `str: Internable` interner pointer-stability across heavy churn.
  - Opaque `TypeInfo` stability through the reorganized `bevy_reflect`
    (`utility::NonGenericTypeInfoCell`).
  - `ReflectFromReflect` and `ReflectDefault` type-data round-trips through a
    `TypeRegistry`.
  - Derived-component mutability: insert / mutate-in-place / remove.
  - `reflect_clone` producing a fully-typed `Box<dyn Reflect>`.

### Notes

- **No breaking changes** to the public API of generated types. `new`,
  `as_str`, `Display`, `From`, `Deref`, `Default`, serde, and the reflection
  hierarchy are all unchanged.
- Bevy 0.19 reorganized `bevy_reflect` into kind-specific modules at the crate
  root and now requires the interned type to implement `Internable`. The paths
  the macro emits remain re-exported and `str` implements `Internable`, so
  generated code continues to compile without changes.

## [0.3.0] - 2026-02-18

Compatibility bump from Bevy 0.17 to **Bevy 0.18**.

### Changed

- Updated compatibility to Bevy 0.18.
- The optional `bevy-inspector-egui` inspector integration is now gated behind a
  proper `dev` feature on this crate rather than emitting
  `#[cfg(feature = "dev")]` into user code.

### Notes

- No breaking changes to the public API of generated types.

## [0.2.0] - 2026-01-18

Compatibility bump from Bevy 0.16 to **Bevy 0.17**.

### Changed

- Updated the reflection implementations to Bevy 0.17's reflection system.
- Removed the deprecated `clone_value` method from the generated
  `PartialReflect` implementation.

### Added

- Added the `reflect_clone` method for proper cloning support.

### Notes

- No breaking changes to the public API of generated types.

## [0.1.0] - 2026-01-18

Initial release, compatible with **Bevy 0.16**.

### Added

- The `InternedId` derive macro generating a complete interned string ID type:
  interner, `new` / `as_str`, `Display`, `From<&str>` / `From<String>`, `Deref`,
  `Default`, serde `Serialize` / `Deserialize`, the full Bevy reflection
  hierarchy, and optional `bevy-inspector-egui` support.

[0.4.0]: https://github.com/MolecularSadism/bevy_interned_id/compare/v0.3.0...v0.4.0
[0.3.0]: https://github.com/MolecularSadism/bevy_interned_id/compare/v0.2.0...v0.3.0
[0.2.0]: https://github.com/MolecularSadism/bevy_interned_id/compare/v0.1.0...v0.2.0
[0.1.0]: https://github.com/MolecularSadism/bevy_interned_id/releases/tag/v0.1.0
