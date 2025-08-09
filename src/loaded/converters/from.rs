use super::super::{component::LoadedComponentInput, element::LoadedElement};

/// Conversion implementations for `LoadedComponentInput` from various types.
///
/// This module provides `From` implementations to convert common types
/// (like `T`, `Result<T, E>`, and `Option<U>`) into `LoadedComponentInput<T>`,
/// making it easier to integrate with the loaded component framework.

impl<T: LoadedElement> From<T> for LoadedComponentInput<T> {
    /// Creates a success state `LoadedComponentInput<T>` from a `LoadedElement`.
    ///
    /// This implementation allows direct conversion from any `LoadedElement` type
    /// to a `LoadedComponentInput` in the success state.
    ///
    /// # Parameters
    ///
    /// * `value` - The component to wrap in a success state
    ///
    /// # Returns
    ///
    /// A `LoadedComponentInput` in the success state containing the provided component
    ///
    /// # Examples
    ///
    /// ```rust
    /// use crate::loaded::prelude::*;
    ///
    /// let component = MyComponent { /* ... */ };
    /// let input: LoadedComponentInput<MyComponent> = component.into();
    ///
    /// rsx! {
    ///     LoadedComponent { input }
    /// }
    /// ```
    #[inline(always)]
    fn from(value: T) -> Self {
        Self::LoadedSuccess(value)
    }
}
impl<T: LoadedElement, E: core::fmt::Display> From<Result<T, E>> for LoadedComponentInput<T> {
    /// Creates a `LoadedComponentInput<T>` from a `Result` containing a `LoadedElement`.
    ///
    /// This implementation:
    /// - Converts `Ok(t)` to a success state
    /// - Converts `Err(e)` to an error state with the error message
    ///
    /// # Parameters
    ///
    /// * `value` - The result to convert
    ///
    /// # Returns
    ///
    /// A `LoadedComponentInput` in either success or error state based on the result
    ///
    /// # Examples
    ///
    /// ```rust
    /// use crate::loaded::prelude::*;
    ///
    /// fn get_component() -> Result<MyComponent, String> {
    ///     // Some operation that might fail
    ///     Err("Failed to load data".to_string())
    /// }
    ///
    /// let input: LoadedComponentInput<MyComponent> = get_component().into();
    /// // input will be LoadedComponentInput::LoadedError("Failed to load data")
    /// ```
    #[inline(always)]
    fn from(value: Result<T, E>) -> Self {
        match value {
            Ok(t) => Self::LoadedSuccess(t),
            Err(e) => Self::LoadedError(e.to_string()),
        }
    }
}
impl<T: LoadedElement, U: Into<LoadedComponentInput<T>>> From<Option<U>>
    for LoadedComponentInput<T>
{
    /// Creates a `LoadedComponentInput<T>` from an `Option` containing a convertible type.
    ///
    /// This implementation:
    /// - Converts `Some(u)` by applying the `Into<LoadedComponentInput<T>>` implementation of `U`
    /// - Converts `None` to a loading state
    ///
    /// This allows for seamless integration with asynchronous data sources that
    /// may not yet have returned a value.
    ///
    /// # Parameters
    ///
    /// * `value` - The option to convert
    ///
    /// # Returns
    ///
    /// A `LoadedComponentInput` based on the option's state
    ///
    /// # Examples
    ///
    /// ```rust
    /// use crate::loaded::prelude::*;
    ///
    /// // Resource that hasn't loaded yet
    /// let data: Option<MyData> = None;
    ///
    /// // Convert to LoadedComponentInput (will be Loading)
    /// let input: LoadedComponentInput<MyComponent> = data.map(MyComponent::from).into();
    ///
    /// // Later when data is available
    /// let data: Option<MyData> = Some(my_data);
    /// let input: LoadedComponentInput<MyComponent> = data.map(MyComponent::from).into();
    /// // input will be LoadedComponentInput::LoadedSuccess(...)
    /// ```
    #[inline(always)]
    fn from(value: Option<U>) -> Self {
        match value {
            Some(u) => u.into(),
            None => Self::Loading,
        }
    }
}
