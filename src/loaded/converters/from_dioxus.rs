//! Conversions from Dioxus reactive types to `LoadedComponentInput`.
//!
//! This module provides implementations to convert Dioxus reactive types
//! (like `Resource`, `ReadOnlySignal`, and `Memo`) into `LoadedComponentInput`,
//! enabling seamless integration with the reactive system.

use dioxus::{
    hooks::Resource,
    signals::{Memo, ReadOnlySignal, Readable},
};

use super::{
    super::{component::LoadedComponentInput, element::LoadedElement},
    RefInto,
};

/// Converts a Dioxus `Resource` into a `LoadedComponentInput`.
///
/// This implementation allows direct use of Dioxus resources with the loaded component system.
/// It reads the current value of the resource and converts it using `RefInto`.
///
/// # Type Parameters
///
/// * `T` - The component type that implements `LoadedElement`
/// * `U` - The resource content type
///
/// # Parameters
///
/// * `value` - The resource to convert
///
/// # Returns
///
/// A `LoadedComponentInput` representing the current state of the resource
///
/// # Examples
///
/// ```rust
/// use crate::loaded::prelude::*;
///
/// #[component]
/// fn UserProfile(user_id: String) -> Element {
///     let user_data = use_resource(move || async move {
///         fetch_user_data(user_id).await
///     });
///
///     rsx! {
///         LoadedComponent::<UserProfileView> { input: user_data.into() }
///     }
/// }
/// ```
impl<T: LoadedElement, U> From<Resource<U>> for LoadedComponentInput<T>
where
    Option<U>: RefInto<LoadedComponentInput<T>>,
{
    fn from(value: Resource<U>) -> Self {
        (&*value.read()).ref_into()
    }
}

/// Converts a Dioxus `ReadOnlySignal` into a `LoadedComponentInput`.
///
/// This implementation allows use of Dioxus signals with the loaded component system.
/// It reads the current value of the signal and converts it using `RefInto`.
///
/// # Type Parameters
///
/// * `T` - The component type that implements `LoadedElement`
/// * `U` - The signal content type that can be converted to a `LoadedComponentInput<T>`
///
/// # Parameters
///
/// * `value` - The signal to convert
///
/// # Returns
///
/// A `LoadedComponentInput` representing the current state of the signal
///
/// # Examples
///
/// ```rust
/// use crate::loaded::prelude::*;
///
/// #[component]
/// fn StatusIndicator() -> Element {
///     let status = use_signal(|| ConnectionStatus::Connecting);
///
///     rsx! {
///         LoadedComponent::<StatusIndicatorView> { input: status.into() }
///     }
/// }
/// ```
impl<T: LoadedElement, U: RefInto<LoadedComponentInput<T>>> From<ReadOnlySignal<U>>
    for LoadedComponentInput<T>
{
    fn from(value: ReadOnlySignal<U>) -> Self {
        (&*value.read()).ref_into()
    }
}

/// Converts a Dioxus `Memo` into a `LoadedComponentInput`.
///
/// This implementation allows use of Dioxus memoized values with the loaded component system.
/// It reads the current value of the memo and converts it using `RefInto`.
///
/// # Type Parameters
///
/// * `T` - The component type that implements `LoadedElement`
/// * `U` - The memo content type that can be converted to a `LoadedComponentInput<T>`
///
/// # Parameters
///
/// * `value` - The memo to convert
///
/// # Returns
///
/// A `LoadedComponentInput` representing the current state of the memo
///
/// # Examples
///
/// ```rust
/// use crate::loaded::prelude::*;
///
/// #[component]
/// fn TransactionSummary(transactions: Vec<Transaction>) -> Element {
///     let processed_data = use_memo(move || {
///         // Complex computation based on transactions
///         TransactionViewModel::process(transactions)
///     });
///
///     rsx! {
///         LoadedComponent::<TransactionSummaryView> { input: processed_data.into() }
///     }
/// }
/// ```
impl<T: LoadedElement, U: RefInto<LoadedComponentInput<T>> + PartialEq> From<Memo<U>>
    for LoadedComponentInput<T>
{
    fn from(value: Memo<U>) -> Self {
        (&*value.read()).ref_into()
    }
}
