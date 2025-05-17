use super::super::{component::LoadedComponentInput, element::LoadedElement};

impl<T: LoadedElement> From<T> for LoadedComponentInput<T> {
    /// Create a LoadedComponentInput<T> from a LoadedElement
    #[inline(always)]
    fn from(value: T) -> Self {
        Self::LoadedSuccess(value)
    }
}
impl<T: LoadedElement, E: core::fmt::Display> From<Result<T, E>> for LoadedComponentInput<T> {
    /// Create a LoadedComponentInput<T> from a Result of a LoadedElement
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
    /// Create a LoadedComponentInput<T> from an Option of a Into<LoadedComponentInput<T>>
    #[inline(always)]
    fn from(value: Option<U>) -> Self {
        match value {
            Some(u) => u.into(),
            None => Self::Loading,
        }
    }
}
