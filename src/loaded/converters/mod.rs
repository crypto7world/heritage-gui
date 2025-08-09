//! Conversion traits and implementations for transforming between data types.
//!
//! This module provides traits and implementations for converting between
//! data types, with a focus on reference-based conversions to avoid unnecessary
//! cloning when working with immutable data.

use std::collections::HashMap;

use crate::utils::CheapClone;

mod from;
mod from_dioxus;
mod ref_from;

pub use ref_from::{LoadedSuccessConversionMarker, TypeCouple};

/// A trait for creating a value from a reference to another value.
///
/// This trait is similar to `From`, but it works with references to avoid
/// unnecessary cloning when the source value doesn't need to be consumed.
///
/// # Type Parameters
///
/// * `R` - The reference type to convert from
///
/// # Examples
///
/// ```rust
/// use crate::loaded::prelude::*;
///
/// struct UserViewModel {
///     display_name: String,
///     role_badge: String,
/// }
///
/// impl FromRef<UserData> for UserViewModel {
///     fn from_ref(data: &UserData) -> Self {
///         Self {
///             display_name: format!("{} {}", data.first_name, data.last_name),
///             role_badge: data.role.badge_name(),
///         }
///     }
/// }
///
/// // Now you can use:
/// // let view_model = user_data.ref_into();
/// ```
pub trait FromRef<R: ?Sized> {
    /// Creates a new instance from a reference to `R`.
    ///
    /// # Parameters
    ///
    /// * `value` - The reference to convert from
    ///
    /// # Returns
    ///
    /// A new instance of `Self`
    fn from_ref(value: &R) -> Self;
}

/// A trait for converting a value into another type by reference.
///
/// This is the reciprocal of `FromRef` and provides a convenient method
/// for performing reference-based conversions.
///
/// # Type Parameters
///
/// * `T` - The type to convert into
///
/// # Examples
///
/// ```rust
/// use crate::loaded::prelude::*;
///
/// // Using the UserViewModel from the FromRef example
/// fn render_user(data: &UserData) -> Element {
///     let view_model: UserViewModel = data.ref_into();
///
///     rsx! {
///         div {
///             span { {view_model.display_name} }
///             span { class: "badge", {view_model.role_badge} }
///         }
///     }
/// }
/// ```
pub trait RefInto<T> {
    /// Converts this value to the specified type by reference.
    ///
    /// # Returns
    ///
    /// A value of type `T` created from a reference to `self`
    fn ref_into(&self) -> T;
}

/// Blanket implementation of `RefInto` for any type that can be a source
/// for a `FromRef` conversion.
impl<R: ?Sized, T: FromRef<R>> RefInto<T> for R {
    #[inline(always)]
    fn ref_into(&self) -> T {
        T::from_ref(self)
    }
}

// QoL blanket implementations
// impl<T: FromRef<U>, U> FromRef<CheapClone<U>> for T {
//     #[inline(always)]
//     fn from_ref(value: &CheapClone<U>) -> Self {
//         value.as_ref().ref_into()
//     }
// }

impl<T: FromRef<[U]>, U> FromRef<Vec<U>> for T {
    #[inline(always)]
    fn from_ref(value: &Vec<U>) -> Self {
        value.as_slice().ref_into()
    }
}
impl<T: FromRef<U>, U> FromRef<Vec<U>> for Vec<T> {
    #[inline(always)]
    fn from_ref(value: &Vec<U>) -> Self {
        value.iter().map(U::ref_into).collect()
    }
}

impl<T: FromRef<[U]>, U> FromRef<CheapClone<[U]>> for T {
    #[inline(always)]
    fn from_ref(value: &CheapClone<[U]>) -> Self {
        value.as_ref().ref_into()
    }
}
impl<T: FromRef<U>, U> FromRef<CheapClone<[U]>> for CheapClone<[T]> {
    #[inline(always)]
    fn from_ref(value: &CheapClone<[U]>) -> Self {
        value.iter().map(U::ref_into).collect()
    }
}

impl<K: core::hash::Hash + Eq + Clone, T: FromRef<U>, U> FromRef<HashMap<K, U>> for HashMap<K, T> {
    #[inline(always)]
    fn from_ref(value: &HashMap<K, U>) -> Self {
        value
            .iter()
            .map(|(k, u)| (k.clone(), u.ref_into()))
            .collect()
    }
}
