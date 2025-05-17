//! Conversion traits and implementations for transforming between data types.
//!
//! This module provides traits and implementations for converting between
//! data types, with a focus on reference-based conversions to avoid unnecessary
//! cloning when working with immutable data.

mod from;
mod from_dioxus;
mod ref_from;

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
pub trait FromRef<R> {
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
impl<T: Clone> FromRef<T> for T {
    #[inline(always)]
    fn from_ref(value: &T) -> Self {
        value.clone()
    }
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
impl<R, T: FromRef<R>> RefInto<T> for R {
    #[inline(always)]
    fn ref_into(&self) -> T {
        T::from_ref(self)
    }
}
