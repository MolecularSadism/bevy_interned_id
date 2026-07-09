//! The crate-level documentation below is sourced directly from `README.md`
//! via [`include_str!`], keeping the README and the rustdoc landing page in
//! sync from a single source of truth. The Rust examples in the README are
//! compiled and executed as doc tests (see the `bevy` dev-dependency).
#![doc = include_str!("../README.md")]

use proc_macro::TokenStream;
use proc_macro2::TokenStream as TokenStream2;
use quote::{format_ident, quote};
use syn::{DeriveInput, Ident, parse_macro_input};

/// Generate the interner and basic methods for an ID type.
fn generate_core_impl(name: &Ident, interner_name: &Ident) -> TokenStream2 {
    quote! {
        static #interner_name: bevy::ecs::intern::Interner<str> =
            bevy::ecs::intern::Interner::new();

        impl #name {
            /// Create a new ID from a string.
            /// The string is interned for efficient comparison.
            #[must_use]
            pub fn new(id: &str) -> Self {
                Self(#interner_name.intern(id))
            }

            /// Get the string value of this ID.
            /// Returns the interned static string.
            #[must_use]
            pub fn as_str(&self) -> &'static str {
                self.0.0
            }
        }
    }
}

/// Generate standard trait implementations (Display, From, Deref, Default).
fn generate_standard_traits(name: &Ident) -> TokenStream2 {
    quote! {
        impl std::fmt::Display for #name {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                write!(f, "{}", self.as_str())
            }
        }

        impl From<&str> for #name {
            fn from(s: &str) -> Self {
                Self::new(s)
            }
        }

        impl From<String> for #name {
            fn from(s: String) -> Self {
                Self::new(&s)
            }
        }

        impl std::ops::Deref for #name {
            type Target = str;

            fn deref(&self) -> &Self::Target {
                self.0.0
            }
        }

        impl Default for #name {
            fn default() -> Self {
                Self::new("")
            }
        }
    }
}

/// Generate serde serialization implementations.
fn generate_serde_impls(name: &Ident) -> TokenStream2 {
    quote! {
        impl serde::Serialize for #name {
            fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
            where
                S: serde::Serializer,
            {
                serializer.serialize_str(self.as_str())
            }
        }

        impl<'de> serde::Deserialize<'de> for #name {
            fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
            where
                D: serde::Deserializer<'de>,
            {
                let s = String::deserialize(deserializer)?;
                Ok(#name::new(&s))
            }
        }
    }
}

/// Generate `PartialReflect` trait implementation.
fn generate_partial_reflect_impl(name: &Ident, name_str: &str) -> TokenStream2 {
    quote! {
        impl bevy::reflect::PartialReflect for #name {
            fn get_represented_type_info(&self) -> Option<&'static bevy::reflect::TypeInfo> {
                Some(<Self as bevy::reflect::Typed>::type_info())
            }

            fn into_partial_reflect(self: Box<Self>) -> Box<dyn bevy::reflect::PartialReflect> {
                self
            }

            fn as_partial_reflect(&self) -> &dyn bevy::reflect::PartialReflect {
                self
            }

            fn as_partial_reflect_mut(&mut self) -> &mut dyn bevy::reflect::PartialReflect {
                self
            }

            fn try_into_reflect(
                self: Box<Self>,
            ) -> Result<Box<dyn bevy::reflect::Reflect>, Box<dyn bevy::reflect::PartialReflect>>
            {
                Ok(self)
            }

            fn try_as_reflect(&self) -> Option<&dyn bevy::reflect::Reflect> {
                Some(self)
            }

            fn try_as_reflect_mut(&mut self) -> Option<&mut dyn bevy::reflect::Reflect> {
                Some(self)
            }

            fn apply(&mut self, value: &dyn bevy::reflect::PartialReflect) {
                if let Some(other) = value.try_downcast_ref::<Self>() {
                    *self = *other;
                }
            }

            fn try_apply(
                &mut self,
                value: &dyn bevy::reflect::PartialReflect,
            ) -> Result<(), bevy::reflect::ApplyError> {
                if let Some(other) = value.try_downcast_ref::<Self>() {
                    *self = *other;
                    Ok(())
                } else {
                    Err(bevy::reflect::ApplyError::MismatchedTypes {
                        from_type: value.reflect_type_path().to_string().into_boxed_str(),
                        to_type: Self::type_path().to_string().into_boxed_str(),
                    })
                }
            }

            fn reflect_kind(&self) -> bevy::reflect::ReflectKind {
                bevy::reflect::ReflectKind::Opaque
            }

            fn reflect_ref(&self) -> bevy::reflect::ReflectRef<'_> {
                bevy::reflect::ReflectRef::Opaque(self)
            }

            fn reflect_mut(&mut self) -> bevy::reflect::ReflectMut<'_> {
                bevy::reflect::ReflectMut::Opaque(self)
            }

            fn reflect_owned(self: Box<Self>) -> bevy::reflect::ReflectOwned {
                bevy::reflect::ReflectOwned::Opaque(self)
            }

            fn reflect_hash(&self) -> Option<u64> {
                use std::hash::{Hash, Hasher};
                let mut hasher = std::collections::hash_map::DefaultHasher::new();
                self.hash(&mut hasher);
                Some(hasher.finish())
            }

            fn reflect_partial_eq(
                &self,
                value: &dyn bevy::reflect::PartialReflect,
            ) -> Option<bool> {
                value.try_downcast_ref::<Self>().map(|other| self == other)
            }

            fn debug(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                write!(f, "{}(\"{}\")", #name_str, self.as_str())
            }

            fn reflect_clone(&self) -> Result<Box<dyn bevy::reflect::Reflect>, bevy::reflect::ReflectCloneError> {
                Ok(Box::new(*self))
            }
        }
    }
}

/// Generate `Reflect` trait implementation.
fn generate_reflect_impl(name: &Ident) -> TokenStream2 {
    quote! {
        impl bevy::reflect::Reflect for #name {
            fn into_any(self: Box<Self>) -> Box<dyn std::any::Any> {
                self
            }

            fn as_any(&self) -> &dyn std::any::Any {
                self
            }

            fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
                self
            }

            fn into_reflect(self: Box<Self>) -> Box<dyn bevy::reflect::Reflect> {
                self
            }

            fn as_reflect(&self) -> &dyn bevy::reflect::Reflect {
                self
            }

            fn as_reflect_mut(&mut self) -> &mut dyn bevy::reflect::Reflect {
                self
            }

            fn set(
                &mut self,
                value: Box<dyn bevy::reflect::Reflect>,
            ) -> Result<(), Box<dyn bevy::reflect::Reflect>> {
                *self = *value.downcast()?;
                Ok(())
            }
        }
    }
}

/// Generate `Typed`, `TypePath`, `FromReflect`, and `GetTypeRegistration` implementations.
fn generate_reflection_meta_impls(name: &Ident, name_str: &str) -> TokenStream2 {
    quote! {
        impl bevy::reflect::Typed for #name {
            fn type_info() -> &'static bevy::reflect::TypeInfo {
                static CELL: bevy::reflect::utility::NonGenericTypeInfoCell =
                    bevy::reflect::utility::NonGenericTypeInfoCell::new();
                CELL.get_or_set(|| {
                    bevy::reflect::TypeInfo::Opaque(bevy::reflect::OpaqueInfo::new::<Self>())
                })
            }
        }

        impl bevy::reflect::TypePath for #name {
            fn type_path() -> &'static str {
                concat!(module_path!(), "::", #name_str)
            }

            fn short_type_path() -> &'static str {
                #name_str
            }
        }

        impl bevy::reflect::FromReflect for #name {
            fn from_reflect(reflect: &dyn bevy::reflect::PartialReflect) -> Option<Self> {
                reflect.try_downcast_ref::<Self>().copied()
            }
        }

        impl bevy::reflect::GetTypeRegistration for #name {
            fn get_type_registration() -> bevy::reflect::TypeRegistration {
                let mut registration = bevy::reflect::TypeRegistration::of::<Self>();
                registration.insert::<bevy::reflect::ReflectFromReflect>(
                    bevy::reflect::FromType::<Self>::from_type(),
                );
                registration.insert::<bevy::reflect::ReflectFromPtr>(
                    bevy::reflect::FromType::<Self>::from_type(),
                );
                registration.insert::<bevy::prelude::ReflectDefault>(
                    bevy::reflect::FromType::<Self>::from_type(),
                );
                registration
            }
        }
    }
}

/// Generate inspector UI implementation for dev feature.
#[cfg(feature = "dev")]
fn generate_inspector_impl(name: &Ident) -> TokenStream2 {
    quote! {
        impl bevy_inspector_egui::inspector_egui_impls::InspectorPrimitive for #name {
            fn ui(
                &mut self,
                ui: &mut bevy_inspector_egui::egui::Ui,
                _options: &dyn std::any::Any,
                _id: bevy_inspector_egui::egui::Id,
                _env: bevy_inspector_egui::reflect_inspector::InspectorUi<'_, '_>,
            ) -> bool {
                ui.label(self.as_str());
                false // ID types are not editable
            }

            fn ui_readonly(
                &self,
                ui: &mut bevy_inspector_egui::egui::Ui,
                _options: &dyn std::any::Any,
                _id: bevy_inspector_egui::egui::Id,
                _env: bevy_inspector_egui::reflect_inspector::InspectorUi<'_, '_>,
            ) {
                ui.label(self.as_str());
            }
        }
    }
}

/// Derive macro for generating interned string ID types.
///
/// This macro generates a complete ID type with interner, methods, and trait implementations.
///
/// # Requirements
///
/// The struct must:
/// - Be a newtype wrapping `bevy::ecs::intern::Interned<str>`
/// - Manually derive: `Clone`, `Copy`, `PartialEq`, `Eq`, `Hash`, `Debug`
///
/// # Generated Code
///
/// The macro generates:
/// 1. A static interner unique to this type
/// 2. Core methods: `new()` and `as_str()`
/// 3. Standard traits: Display, From, Deref, Default
/// 4. Serialization: Serialize, Deserialize
/// 5. Bevy reflection: Full reflection hierarchy
/// 6. Inspector UI (dev feature only)
///
/// # Examples
///
/// ## Basic Usage
///
/// ```rust
/// use msg_interned_id::InternedId;
/// use bevy::prelude::*;
///
/// #[derive(InternedId, Clone, Copy, PartialEq, Eq, Hash, Debug)]
/// pub struct SpellId(bevy::ecs::intern::Interned<str>);
///
/// let id = SpellId::new("fireball");
/// assert_eq!(id.as_str(), "fireball");
/// assert_eq!(&*id, "fireball"); // Deref to &str
/// ```
///
/// ## As ECS Component
///
/// ```rust
/// use msg_interned_id::InternedId;
/// use bevy::prelude::*;
///
/// #[derive(Component, InternedId, Clone, Copy, PartialEq, Eq, Hash, Debug)]
/// pub struct ItemId(bevy::ecs::intern::Interned<str>);
///
/// fn spawn_item(mut commands: Commands) {
///     commands.spawn(ItemId::new("health_potion"));
/// }
///
/// // `spawn_item` is a valid Bevy system:
/// bevy::ecs::system::assert_is_system(spawn_item);
/// ```
///
/// ## With Serialization
///
/// ```rust
/// use msg_interned_id::InternedId;
/// use bevy::prelude::*;
///
/// #[derive(InternedId, Clone, Copy, PartialEq, Eq, Hash, Debug)]
/// pub struct QuestId(bevy::ecs::intern::Interned<str>);
///
/// // Serializes as: "main_quest", deserializes from: "main_quest"
/// let quest = QuestId::new("main_quest");
/// let json = serde_json::to_string(&quest).unwrap();
/// assert_eq!(json, "\"main_quest\"");
/// let restored: QuestId = serde_json::from_str(&json).unwrap();
/// assert_eq!(quest, restored);
/// ```
#[proc_macro_derive(InternedId)]
pub fn derive_interned_id(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let name = &input.ident;
    let interner_name = format_ident!("{}_INTERNER", name.to_string().to_uppercase());
    let name_str = name.to_string();

    // Generate each section using helper functions
    let core = generate_core_impl(name, &interner_name);
    let standard_traits = generate_standard_traits(name);
    let serde = generate_serde_impls(name);
    let partial_reflect = generate_partial_reflect_impl(name, &name_str);
    let reflect = generate_reflect_impl(name);
    let reflection_meta = generate_reflection_meta_impls(name, &name_str);
    #[cfg(feature = "dev")]
    let inspector = generate_inspector_impl(name);
    #[cfg(not(feature = "dev"))]
    let inspector = quote! {};

    let expanded = quote! {
        #core
        #standard_traits
        #serde
        #partial_reflect
        #reflect
        #reflection_meta
        #inspector
    };

    TokenStream::from(expanded)
}
