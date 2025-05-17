use super::{component::LoadedComponentInput, element::LoadedElement};

/// A trait for transforming loaded elements based on parent component state.
///
/// This trait allows parent components to control how child components should be rendered
/// based on the parent's loading state. It's used internally by the `element` method
/// of the `LoadedElement` trait.
///
/// Implementations of this trait correspond to the different loading states:
/// - `Loading`: Maps child components to a loading state
/// - `LoadedSuccess`: Maps child components to a success state
/// - `LoadedError`: Maps child components to an error state
///
/// # Examples
///
/// ```rust
/// use crate::loaded::prelude::*;
///
/// // Normally used through the LoadedElement::element method
/// // This example shows manual usage for illustration
/// struct ParentComponent {
///     child: ChildComponent,
///     state: LoadingState,
/// }
///
/// impl LoadedElement for ParentComponent {
///     // ...
///     fn element<M: LoadedComponentInputMapper>(self, mapper: M) -> Element {
///         // Use the parent mapper to determine how to render the child
///         let child_input = mapper.map(self.child);
///         
///         rsx! {
///             div {
///                 LoadedComponent { input: child_input }
///             }
///         }
///     }
/// }
/// ```
pub trait LoadedComponentInputMapper {
    /// Maps a loaded element to the appropriate loading state.
    ///
    /// # Type Parameters
    ///
    /// * `T` - The component type that implements `LoadedElement`
    ///
    /// # Parameters
    ///
    /// * `t` - The component to map
    ///
    /// # Returns
    ///
    /// A `LoadedComponentInput` representing the mapped component state
    fn map<T: LoadedElement>(&self, t: T) -> LoadedComponentInput<T>;
}

/// A mapper that sets components to loading state.
///
/// This mapper transforms any component into a loading state,
/// regardless of the component's actual data.
///
/// It's used to display placeholders for child components when
/// the parent component is in a loading state.
pub struct Loading;
impl LoadedComponentInputMapper for Loading {
    #[inline(always)]
    fn map<T: LoadedElement>(&self, _t: T) -> LoadedComponentInput<T> {
        LoadedComponentInput::Loading
    }
}

/// A mapper that sets components to error state.
///
/// This mapper transforms any component into an error state
/// with the provided error message.
///
/// It's used to propagate error states from parent components
/// to their children.
///
/// # Fields
///
/// * `0` - The error message to display
pub struct LoadedError(pub String);
impl LoadedComponentInputMapper for LoadedError {
    #[inline(always)]
    fn map<T: LoadedElement>(&self, _t: T) -> LoadedComponentInput<T> {
        LoadedComponentInput::LoadedError(self.0.clone())
    }
}

/// A mapper that sets components to success state.
///
/// This mapper preserves the component's data and marks it as
/// successfully loaded.
///
/// It's used when a parent component is in a success state and
/// its children should display their actual content.
pub struct LoadedSuccess;
impl LoadedComponentInputMapper for LoadedSuccess {
    #[inline(always)]
    fn map<T: LoadedElement>(&self, t: T) -> LoadedComponentInput<T> {
        LoadedComponentInput::LoadedSuccess(t)
    }
}
