use dioxus::prelude::*;

use super::{element::LoadedElement, loaders::Loader};

/// Represents the different states of a component during loading.
///
/// This enum is used by the `LoadedComponent` to determine how to display
/// a component based on its current loading state.
///
/// # Type Parameters
///
/// * `T` - The component type that implements `LoadedElement`
///
/// # Examples
///
/// ```rust
/// use crate::loaded::prelude::*;
///
/// // Direct usage in a component
/// let input = if is_loading {
///     LoadedComponentInput::Loading
/// } else if let Some(data) = processed_data {
///     LoadedComponentInput::LoadedSuccess(MyComponent::from(data))
/// } else {
///     LoadedComponentInput::LoadedError("Failed to load data".to_string())
/// };
///
/// rsx! {
///     LoadedComponent { input }
/// }
/// ```
#[derive(Debug, Clone, PartialEq)]
pub enum LoadedComponentInput<T: LoadedElement + Clone + PartialEq> {
    /// The component is in loading state and should display a placeholder
    Loading,
    /// The component has successfully loaded and contains the actual data
    LoadedSuccess(T),
    /// The component failed to load and contains an error message
    LoadedError(String),
}

/// A component that handles displaying UI elements in different loading states.
///
/// This component is the main interface for the loaded framework. It takes a
/// `LoadedComponentInput` and renders the appropriate UI based on the loading state:
/// - For loading state: displays a placeholder with the component's configured loader
/// - For success state: displays the actual component
/// - For error state: logs the error and displays a placeholder with the component's configured loader
///
/// # Type Parameters
///
/// * `T` - The component type that implements `LoadedElement`
///
/// # Parameters
///
/// * `input` - The loading state and data for the component
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
///         div { class: "profile-card",
///             LoadedComponent { input: user_data.map(|data| UserProfileView::from(data)).into() }
///         }
///     }
/// }
/// ```
#[component]
pub fn LoadedComponent<T: LoadedElement>(input: LoadedComponentInput<T>) -> Element {
    match input {
        LoadedComponentInput::Loading => {
            T::Loader::load(T::place_holder().element(super::mapper::Loading))
        }
        LoadedComponentInput::LoadedSuccess(c) => c.element(super::mapper::LoadedSuccess),
        LoadedComponentInput::LoadedError(e) => {
            log::error!("{e}");
            T::Loader::error(T::place_holder().element(super::mapper::LoadedError(e)))
        }
    }
}

/// A simplified component that always displays content in the loaded state.
///
/// This component is useful when you have data that is always available and doesn't
/// need to go through a loading state, but you still want to use the `LoadedElement`
/// framework for consistency.
///
/// # Type Parameters
///
/// * `T` - The component type that implements `LoadedElement`
///
/// # Parameters
///
/// * `input` - The component to display
///
/// # Examples
///
/// ```rust
/// use crate::loaded::prelude::*;
///
/// #[component]
/// fn StaticContent() -> Element {
///     let static_data = StaticBanner {
///         title: "Welcome".to_string(),
///         subtitle: "Explore our platform".to_string(),
///     };
///
///     rsx! {
///         div { class: "banner",
///             AlwaysLoadedComponent { input: static_data }
///         }
///     }
/// }
/// ```
#[component]
pub fn AlwaysLoadedComponent<T: LoadedElement>(input: T) -> Element {
    input.element(super::mapper::LoadedSuccess)
}

/// A component that handles displaying static children content in different loading states.
///
/// This component is similar to `LoadedComponent` but instead of rendering a `LoadedElement`,
/// it renders provided children elements. It takes a `LoadedComponentInput<()>` to determine
/// the loading state and renders the children accordingly:
/// - For loading state: displays the children wrapped with a skeleton loader
/// - For success state: displays the children as-is
/// - For error state: logs the error and displays the children wrapped with a skeleton loader
///
/// This is useful when you have static content that needs to participate in loading states
/// but doesn't need to implement the full `LoadedElement` trait.
///
/// # Parameters
///
/// * `input` - The loading state (using unit type `()` as placeholder)
/// * `children` - The static content to display
///
/// # Examples
///
/// ```rust
/// use crate::loaded::prelude::*;
///
/// #[component]
/// fn StaticBanner(is_loading: bool) -> Element {
///     let state = if is_loading {
///         LoadedComponentInput::Loading
///     } else {
///         LoadedComponentInput::LoadedSuccess(())
///     };
///
///     rsx! {
///         StaticLoadedComponent { input: state,
///             div { class: "banner",
///                 h1 { "Welcome to our platform" }
///                 p { "This content is static but participates in loading states" }
///             }
///         }
///     }
/// }
/// ```
#[component]
pub fn StaticLoadedComponent(input: LoadedComponentInput<()>, children: Element) -> Element {
    match input {
        LoadedComponentInput::Loading => <() as LoadedElement>::Loader::load(children),
        LoadedComponentInput::LoadedSuccess(_) => children,
        LoadedComponentInput::LoadedError(e) => {
            log::error!("{e}");
            <() as LoadedElement>::Loader::error(children)
        }
    }
}
