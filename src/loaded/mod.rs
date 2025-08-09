//! # Loaded Module
//!
//! A comprehensive framework for handling UI component loading states in a declarative way.
//!
//! This module provides a structured approach to represent components that can be in different loading
//! states: loading, loaded successfully, or error. It helps in creating consistent UI experiences
//! when dealing with asynchronous data loading, without having to manually handle loading states
//! and placeholders for each component.
//!
//! ## Core Concepts
//!
//! - [`LoadedElement`](element::LoadedElement): Trait for components that can be displayed in different loading states
//! - [`LoadedComponent`](component::LoadedComponent): A wrapper component that handles displaying appropriate UI based on loading state
//! - [`FromRef`](converters::FromRef)/[`RefInto`](converters::RefInto): Conversion traits for transforming data into components
//! - Loaders: Different strategies for displaying loading states (skeleton, transparent)
//!
//! ## Example Usage
//!
//! ```rust
//! use crate::loaded::prelude::*;
//!
//! #[derive(Debug, Clone, PartialEq)]
//! struct MyComponent {
//!     text: String,
//! }
//!
//! impl LoadedElement for MyComponent {
//!     type Loader = SkeletonLoader;
//!
//!     fn element<M: LoadedComponentInputMapper>(self, _m: M) -> Element {
//!         rsx! {
//!             div { class: "my-component", {self.text} }
//!         }
//!     }
//!
//!     fn place_holder() -> Self {
//!         Self { text: "Loading...".to_string() }
//!     }
//! }
//! ```

pub mod component;
mod converters;
mod element;
pub mod loaders;
pub mod mapper;

/// Prelude module that re-exports commonly used types and traits.
///
/// Import this module to get access to all the essential components needed
/// to work with the loaded framework.
///
/// # Example
///
/// ```rust
/// use crate::loaded::prelude::*;
/// ```
pub mod prelude {
    pub use super::component::{AlwaysLoadedComponent, LoadedComponent, StaticLoadedComponent};
    pub use super::converters::{FromRef, LoadedSuccessConversionMarker, RefInto, TypeCouple};
    pub use super::element::{Display, Display::Show, LoadedElement};
    pub use super::loaders::{SkeletonLoader, TransparentLoader};
    pub use super::mapper::LoadedComponentInputMapper;
}
