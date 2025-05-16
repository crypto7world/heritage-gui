//! A comprehensive framework for managing UI components with loading, success, and error states.
//!
//! This module provides a robust set of traits and implementations that significantly simplify working
//! with asynchronously loaded data in Dioxus applications. It addresses the common challenge of
//! handling the lifecycle of data-dependent components by providing:
//!
//! 1. Consistent skeleton loading states with automatic placeholders
//! 2. Clean rendering of successfully loaded components
//! 3. Uniform error state handling with appropriate logging
//! 4. Seamless integration with Dioxus state primitives (`Resource`, `Signal`, `Memo`)
//!
//! The design follows Rust's type-driven approach, using traits and type conversions to make
//! the API ergonomic while preserving type safety at compile time.
//!
//! # Core Components
//!
//! - [`LoadedElement`]: A trait for UI components that can render in loaded and placeholder states
//! - [`LoadedComponentInput`]: An enum representing loading, success, and error states
//! - [`LoadedComponent`]: A Dioxus component that renders the appropriate UI based on the input state
//! - [`FromRef`]/[`RefInto`]: Traits for reference-based conversions between types
//!
//! # Example
//!
//! ```rust
//! use dioxus::prelude::*;
//! use crate::components::loaded::{LoadedElement, LoadedComponentInput, LoadedComponent};
//!
//! // Define a component that can be loaded
//! #[derive(Debug, Clone, PartialEq)]
//! struct UserProfile {
//!     name: String,
//!     email: String,
//! }
//!
//! // Implement LoadedElement for the component
//! impl LoadedElement for UserProfile {
//!     fn element<CM: ComponentMapper>(self, _mapper: CM) -> Element {
//!         rsx! {
//!             div {
//!                 h2 { "{self.name}" }
//!                 p { "{self.email}" }
//!             }
//!         }
//!     }
//!
//!     fn place_holder() -> Self {
//!         Self {
//!             name: "Loading...".to_string(),
//!             email: "...".to_string(),
//!         }
//!     }
//! }
//!
//! // Use the component with a resource
//! #[component]
//! fn ProfilePage() -> Element {
//!     let profile: Resource<Result<UserProfile, String>> = use_resource(|| async {
//!         // Fetch user data from API
//!         Ok(UserProfile {
//!             name: "John Doe".to_string(),
//!             email: "john@example.com".to_string(),
//!         })
//!     });
//!
//!     rsx! {
//!         LoadedComponent { input: profile.into() }
//!     }
//! }
//! ```
//!
//! This example demonstrates how the framework automatically handles the loading state
//! of the profile resource, displaying a placeholder while loading and the actual profile
//! or an error message once loading completes.

pub mod badge;
pub mod balance;
pub mod timestamp;

use dioxus::prelude::*;

use crate::utils::{ArcStr, ArcType};

/// A marker trait to enable direct conversion to [`LoadedComponentInput`] for specific types.
///
/// This trait is used to implement targeted type conversions:
/// - `impl<T: LoadedElement + Clone + PartialEq, R: RefInto<T> + ImplDirectIntoLoadedElementInputMarker> FromRef<R> for LoadedComponentInput<T>`
/// - `impl<T: LoadedElement + Clone + PartialEq, R: Into<T> + ImplDirectIntoLoadedElementInputMarker> From<R> for LoadedComponentInput<T>`
///
/// The marker trait enables working around Rust's lack of specialization or negative trait bounds,
/// allowing us to implement conversions for specific types without causing blanket implementation
/// conflicts with the implementations for `Option<T>` and `Result<T, E>`.
///
/// # Usage
///
/// Implement this trait for types that should be directly convertible to `LoadedComponentInput`:
///
/// ```rust
/// impl ImplDirectIntoLoadedElementInputMarker for MyCustomType {}
/// ```
pub trait ImplDirectIntoLoadedElementInputMarker {}
impl ImplDirectIntoLoadedElementInputMarker for btc_heritage_wallet::Wallet {}

/// A trait for UI elements that can be displayed in both loaded and placeholder states.
///
/// This core trait enables components to define both their fully loaded representation
/// and a placeholder representation for loading states. Components implementing this
/// trait work seamlessly with the [`LoadedComponent`] system, which automatically
/// manages transitions between loading, success, and error states.
///
/// # Requirements
///
/// Implementors must:
/// 1. Provide an `element<CM: ComponentMapper>` method that renders the component
/// 2. Define a `place_holder()` method that returns a sensible placeholder
/// 3. Optionally customize `visible_place_holder()` to control skeleton UI behavior
///
/// The trait requires `Clone` and `PartialEq` to enable efficient state management
/// and comparison in Dioxus's reactive context.
pub trait LoadedElement: Clone + PartialEq + 'static {
    /// Renders the component as a Dioxus Element.
    ///
    /// This method is called when the component is in the loaded state. It receives
    /// a [`ComponentMapper`] that can be used to propagate the current state to
    /// child components.
    ///
    /// # Parameters
    ///
    /// * `self` - The component instance to render
    /// * `mapper` - A mapper that converts child components to the appropriate state
    ///
    /// # Returns
    ///
    /// A Dioxus `Element` representing the rendered component
    ///
    /// # Examples
    ///
    /// ```rust
    /// fn element<CM: ComponentMapper>(self, _mapper: CM) -> Element {
    ///     rsx! {
    ///         div {
    ///             h2 { "{self.title}" }
    ///             p { "{self.description}" }
    ///         }
    ///     }
    /// }
    /// ```
    fn element<CM: ComponentMapper>(self, mapper: CM) -> Element;

    /// Creates a placeholder instance of this component.
    ///
    /// The placeholder is used when data is still loading or when an error occurs.
    /// It should represent the structure of the component with placeholder content.
    ///
    /// # Returns
    ///
    /// A placeholder instance of the component type
    ///
    /// # Examples
    ///
    /// ```rust
    /// fn place_holder() -> Self {
    ///     Self {
    ///         title: "Loading...".to_string(),
    ///         description: "...".to_string(),
    ///     }
    /// }
    /// ```
    fn place_holder() -> Self;

    /// Controls whether the placeholder should be directly visible.
    ///
    /// When `false` (the default), the placeholder is wrapped in a skeleton UI
    /// that provides a loading animation. When `true`, the placeholder is displayed
    /// directly without a skeleton wrapper.
    ///
    /// # Returns
    ///
    /// `true` if the placeholder should be directly visible, `false` otherwise
    ///
    /// # Examples
    ///
    /// ```rust
    /// fn visible_place_holder() -> bool {
    ///     true  // Show the placeholder directly without skeleton effect
    /// }
    /// ```
    #[inline(always)]
    fn visible_place_holder() -> bool {
        false
    }
}

macro_rules! loaded_str {
    ($t:ty ) => {
        impl LoadedElement for $t {
            #[inline(always)]
            fn element<CM: ComponentMapper>(self, _mapper: CM) -> Element {
                rsx! {
                    {self}
                }
            }

            fn place_holder() -> Self {
                "Loading...".into()
            }
        }
    };
}
loaded_str!(ArcStr);
loaded_str!(String);
loaded_str!(&'static str);

macro_rules! loaded_iter {
    ($t:ty $(, $clone:expr)?) => {
        impl<T: LoadedElement> LoadedElement for $t {
            #[inline(always)]
            fn element<CM: ComponentMapper>(self, mapper: CM) -> Element {
                rsx! {
                    for item in self {
                        LoadedComponent::<T> { input: mapper.map(item$(. $clone())?) }
                    }
                }
            }

            fn place_holder() -> Self {
                <$t>::from_iter(Some(T::place_holder()))
            }

            #[inline(always)]
            fn visible_place_holder() -> bool {
                true
            }
        }
    };
}
loaded_iter!(Vec<T>);
loaded_iter!(ArcType<[T]>, clone);

/// A trait for creating a value from a reference to another value.
///
/// This trait is the reference-based counterpart to the standard `From` trait. It enables
/// efficient conversions from references without unnecessary cloning of the entire source value.
/// This is particularly valuable when working with Dioxus state primitives like
/// `Resource`, `Signal`, and `Memo`, which provide access to their values through
/// references.
///
/// # Type Parameters
///
/// * `R` - The reference type to convert from
///
/// # Examples
///
/// ```rust
/// // Converting from domain model to UI model
/// impl FromRef<UserData> for UIUserProfile {
///     fn from_ref(data: &UserData) -> Self {
///         Self {
///             display_name: data.name.clone(),
///             avatar_url: data.avatar.clone(),
///         }
///     }
/// }
///
/// // Using the conversion with a Signal
/// let user_data = use_signal(|| fetch_user_data());
/// let ui_profile: UIUserProfile = user_data.read().from_ref();
/// ```
///
/// # Implementation Note
///
/// This trait allows the framework to automatically derive conversions between
/// related types, enabling the seamless flow of data from backend models to
/// UI components without manual conversion at each step.
pub trait FromRef<R> {
    /// Creates a new value from a reference to `I`.
    fn from_ref(value: &R) -> Self;
}
impl<T: Clone> FromRef<T> for T {
    fn from_ref(value: &T) -> Self {
        value.clone()
    }
}

// impl<I: Clone, T: From<I>> FromRef<I> for T {
//     fn from_ref(i: &I) -> Self {
//         T::from(i.clone())
//     }
// }
/// A trait for converting a value by reference into another type.
///
/// This trait is the reference-based complement to the standard `Into` trait and 
/// pairs with [`FromRef`]. It's automatically implemented for any type that 
/// implements [`FromRef`], allowing for an ergonomic conversion API similar to 
/// Rust's standard conversion traits.
///
/// # Type Parameters
///
/// * `T` - The target type to convert into
///
/// # Examples
///
/// ```rust
/// // With FromRef implemented for UIUserProfile:
/// let user_data = get_user_data();
/// 
/// // Convert using ref_into()
/// let ui_profile: UIUserProfile = user_data.ref_into();
/// 
/// // Use in a component
/// rsx! {
///     UserProfileView { profile: user_data.ref_into() }
/// }
/// ```
///
/// # Implementation Note
///
/// This trait enables cleaner, more ergonomic code by allowing reference-based
/// conversions at call sites without explicitly importing the [`FromRef`] trait.
pub trait RefInto<T> {
    /// Converts `self` by reference into the target type `T`.
    fn ref_into(&self) -> T;
}

impl<R, T: FromRef<R>> RefInto<T> for R {
    fn ref_into(&self) -> T {
        T::from_ref(self)
    }
}

/// An enum representing the possible states of a loaded component.
///
/// This enum is a core type in the loading framework, encapsulating the three
/// possible states of an asynchronously loaded component:
///
/// - `Loading`: The data is currently being loaded or not yet available
/// - `LoadedSuccess`: The data was successfully loaded and is ready to display
/// - `LoadedError`: An error occurred while loading the data
///
/// The enum works with the [`LoadedComponent`] to provide a consistent UI experience
/// for asynchronously loaded components, ensuring appropriate display for each state.
///
/// # Type Parameters
///
/// * `T` - A type that implements [`LoadedElement`] and can be cloned and compared
///
/// # Examples
///
/// ```rust
/// // Manual creation
/// let input = LoadedComponentInput::Loading;
/// let input = LoadedComponentInput::LoadedSuccess(profile);
/// let input = LoadedComponentInput::LoadedError("Failed to load".to_string());
///
/// // Automatic conversion from Resource
/// let profile_resource: Resource<Result<Profile, String>> = use_resource(fetch_profile);
/// let input: LoadedComponentInput<Profile> = profile_resource.into();
///
/// // Using with LoadedComponent
/// rsx! {
///     LoadedComponent { input }
/// }
/// ```
///
/// # Implementation Note
///
/// This enum is designed to be created automatically through various `From`/`Into`
/// implementations that handle different source types including `Resource`, `Signal`,
/// `Memo`, `Result`, and `Option`. This allows seamless integration with Dioxus's
/// reactive state system.
#[derive(Debug, Clone, PartialEq)]
pub enum LoadedComponentInput<T: LoadedElement + Clone + PartialEq> {
    /// The component is in a loading state.
    Loading,
    /// The component has been successfully loaded.
    LoadedSuccess(T),
    /// An error occurred while loading the component.
    LoadedError(String),
}

/// A trait for mapping components to their appropriate loaded state representation.
///
/// This trait enables propagating the loading state from parent components to their
/// children, ensuring consistent state handling throughout the component tree.
///
/// Different implementations of this trait handle the various states:
/// - [`SuccessMapper`]: Maps components to the success state
/// - [`LoadingMapper`]: Maps components to the loading state
/// - [`ErrorMapper`]: Maps components to the error state with an error message
///
/// # Type Parameters
///
/// * `T` - A type that implements [`LoadedElement`]
///
/// # Examples
///
/// ```rust
/// // Within a LoadedElement implementation:
/// fn element<CM: ComponentMapper>(self, mapper: CM) -> Element {
///     rsx! {
///         div {
///             // Child component inherits parent's loading state
///             LoadedComponent { input: mapper.map(self.child_component) }
///         }
///     }
/// }
/// ```
pub trait ComponentMapper {
    /// Maps a component to its appropriate loaded state representation.
    ///
    /// # Parameters
    ///
    /// * `t` - The component to map
    ///
    /// # Returns
    ///
    /// A [`LoadedComponentInput`] representing the component in the appropriate state
    fn map<T: LoadedElement>(&self, t: T) -> LoadedComponentInput<T>;
}
/// A mapper that transforms components to the loading state.
///
/// This mapper is used internally by [`LoadedComponent`] when rendering
/// child components in a loading parent context.
pub struct LoadingMapper;
impl ComponentMapper for LoadingMapper {
    #[inline(always)]
    fn map<T: LoadedElement>(&self, _t: T) -> LoadedComponentInput<T> {
        LoadedComponentInput::Loading
    }
}
/// A mapper that transforms components to the error state with a message.
///
/// This mapper is used internally by [`LoadedComponent`] when rendering
/// child components in an error parent context.
pub struct ErrorMapper(String);
impl ComponentMapper for ErrorMapper {
    #[inline(always)]
    fn map<T: LoadedElement>(&self, _t: T) -> LoadedComponentInput<T> {
        LoadedComponentInput::LoadedError(self.0.clone())
    }
}
/// A mapper that transforms components to the success state.
///
/// This mapper is used internally by [`LoadedComponent`] when rendering
/// child components in a successfully loaded parent context.
pub struct SuccessMapper;
impl ComponentMapper for SuccessMapper {
    #[inline(always)]
    fn map<T: LoadedElement>(&self, t: T) -> LoadedComponentInput<T> {
        LoadedComponentInput::LoadedSuccess(t)
    }
}

// From blanket impl

impl<T: LoadedElement, R: Into<T> + ImplDirectIntoLoadedElementInputMarker> From<R>
    for LoadedComponentInput<T>
{
    /// Create a LoadedComponentInput<T> from a raw value (not option, not result)
    fn from(value: R) -> Self {
        Self::LoadedSuccess(value.into())
    }
}
impl<T: LoadedElement, R: Into<T>, E: core::fmt::Display> From<Result<R, E>>
    for LoadedComponentInput<T>
{
    /// Create a LoadedComponentInput<T> from a Result of a RefInto<T>
    fn from(value: Result<R, E>) -> Self {
        match value {
            Ok(r) => Self::LoadedSuccess(r.into()),
            Err(e) => Self::LoadedError(e.to_string()),
        }
    }
}
impl<T: LoadedElement, R: Into<LoadedComponentInput<T>>> From<Option<R>>
    for LoadedComponentInput<T>
{
    /// Create a LoadedComponentInput<T> from an option of a RefInto<LoadedComponentInput<T>>
    fn from(value: Option<R>) -> Self {
        match value {
            Some(r) => r.into(),
            None => Self::Loading,
        }
    }
}
impl<T: LoadedElement, U: RefInto<T>> From<ArcType<U>> for LoadedComponentInput<T> {
    /// Create a LoadedComponentInput<T> from a raw value (not option, not result)
    fn from(value: ArcType<U>) -> Self {
        Self::LoadedSuccess(value.as_ref().ref_into())
    }
}
impl<T: LoadedElement, U: RefInto<T>> From<ArcType<[U]>> for LoadedComponentInput<ArcType<[T]>> {
    /// Create a LoadedComponentInput<T> from a raw value (not option, not result)
    fn from(value: ArcType<[U]>) -> Self {
        Self::LoadedSuccess(value.iter().map(|u| u.ref_into()).collect())
    }
}
impl<T: LoadedElement, U: Into<T>> From<Vec<U>> for LoadedComponentInput<Vec<T>> {
    /// Create a LoadedComponentInput<T> from a raw value (not option, not result)
    fn from(value: Vec<U>) -> Self {
        Self::LoadedSuccess(value.into_iter().map(|u| u.into()).collect())
    }
}

// FromRef blanket impl
impl<T: LoadedElement, R: RefInto<T> + ImplDirectIntoLoadedElementInputMarker> FromRef<R>
    for LoadedComponentInput<T>
{
    /// Create a LoadedComponentInput<T> from a raw value (not option, not result)
    fn from_ref(value: &R) -> Self {
        Self::LoadedSuccess(value.ref_into())
    }
}
impl<T: LoadedElement, R: RefInto<T>, E: core::fmt::Display> FromRef<Result<R, E>>
    for LoadedComponentInput<T>
{
    /// Create a LoadedComponentInput<T> from a Result of a RefInto<T>
    fn from_ref(value: &Result<R, E>) -> Self {
        match value {
            Ok(r) => Self::LoadedSuccess(r.ref_into()),
            Err(e) => Self::LoadedError(e.to_string()),
        }
    }
}
impl<T: LoadedElement, R: RefInto<LoadedComponentInput<T>>> FromRef<Option<R>>
    for LoadedComponentInput<T>
{
    /// Create a LoadedComponentInput<T> from an option of a RefInto<LoadedComponentInput<T>>
    fn from_ref(value: &Option<R>) -> Self {
        match value {
            Some(r) => r.ref_into(),
            None => Self::Loading,
        }
    }
}
impl<T: LoadedElement, U: RefInto<T>> FromRef<ArcType<U>> for LoadedComponentInput<T> {
    fn from_ref(value: &ArcType<U>) -> Self {
        LoadedComponentInput::from(value.clone())
    }
}
impl<T: LoadedElement, U: RefInto<T>> FromRef<ArcType<[U]>> for LoadedComponentInput<ArcType<[T]>> {
    /// Create a LoadedComponentInput<T> from a raw value (not option, not result)
    fn from_ref(value: &ArcType<[U]>) -> Self {
        LoadedComponentInput::from(value.clone())
    }
}
impl<T: LoadedElement, U: RefInto<T>> FromRef<Vec<U>> for LoadedComponentInput<Vec<T>> {
    /// Create a LoadedComponentInput<T> from a raw value (not option, not result)
    fn from_ref(value: &Vec<U>) -> Self {
        LoadedComponentInput::LoadedSuccess(value.iter().map(|u| u.ref_into()).collect())
    }
}

// Dioxus type into LoadedComponentInput blanket impls
impl<T: LoadedElement, U: RefInto<LoadedComponentInput<T>>> From<Resource<U>>
    for LoadedComponentInput<T>
{
    fn from(value: Resource<U>) -> Self {
        match &*value.read() {
            Some(u) => u.ref_into(),
            None => LoadedComponentInput::Loading,
        }
    }
}

impl<T: LoadedElement, U: RefInto<LoadedComponentInput<T>>> From<ReadOnlySignal<U>>
    for LoadedComponentInput<T>
{
    /// Creates a `LoadedComponentInput` from a Dioxus `ReadOnlySignal`.
    ///
    /// This allows direct integration with Dioxus signals, which represent reactive state.
    ///
    /// # Examples
    ///
    /// ```
    /// let profile_signal = use_signal(|| Result::<UserProfile, String>::Ok(
    ///     UserProfile::new("Frank", "frank@example.com")
    /// ));
    ///
    /// // Use the ReadOnlySignal with LoadedComponent
    /// rsx! {
    ///     LoadedComponent { input: profile_signal.into() }
    /// }
    /// ```
    fn from(value: ReadOnlySignal<U>) -> Self {
        (&*value.read()).ref_into()
    }
}

impl<T: LoadedElement, U: RefInto<LoadedComponentInput<T>> + PartialEq> From<Memo<U>>
    for LoadedComponentInput<T>
{
    fn from(value: Memo<U>) -> Self {
        (&*value.read()).ref_into()
    }
}

/// A component that handles the display of elements in different loading states.
///
/// This component is the central piece of the loading framework. It takes a
/// [`LoadedComponentInput`] and renders the appropriate UI based on the input's state:
///
/// - For the `Loading` state, it displays a placeholder with optional skeleton loading effect
/// - For the `LoadedSuccess` state, it displays the actual component
/// - For the `LoadedError` state, it logs the error and displays the placeholder
///
/// The skeleton effect is controlled by the [`LoadedElement::visible_place_holder()`] method.
/// When it returns `false` (the default), the placeholder is wrapped in a skeleton div that
/// provides a loading animation. When it returns `true`, the placeholder is displayed directly.
///
/// # Type Parameters
///
/// * `T` - A type that implements [`LoadedElement`] and can be cloned and compared
///
/// # Properties
///
/// * `input` - A [`LoadedComponentInput`] representing the current state of the component
///
/// # Examples
///
/// ```rust
/// // Using with a Resource
/// let profile_resource: Resource<Result<UserProfile, String>> = use_resource(|| async {
///     // Fetch user profile from API
///     Ok(UserProfile::new("Heidi", "heidi@example.com"))
/// });
///
/// rsx! {
///     LoadedComponent { input: profile_resource.into() }
/// }
///
/// // Using with a direct value
/// let profile = UserProfile::new("Ivan", "ivan@example.com");
///
/// rsx! {
///     LoadedComponent { input: profile.into() }
/// }
///
/// // Using with a Result
/// let result: Result<UserProfile, &str> = Err("Failed to load profile");
///
/// rsx! {
///     LoadedComponent { input: result.into() }
/// }
/// ```
///
/// # Implementation Note
///
/// The component automatically applies the appropriate mapper to child components based
/// on the current state, ensuring consistent state propagation throughout the component
/// tree. It also handles error logging for the error state.
#[component]
pub fn LoadedComponent<T: LoadedElement>(input: LoadedComponentInput<T>) -> Element {
    match input {
        LoadedComponentInput::Loading => {
            let ph = T::place_holder();
            if T::visible_place_holder() {
                ph.element(LoadingMapper)
            } else {
                rsx! {
                    span { class: "skeleton  ",
                        span { class: "invisible ", {ph.element(LoadingMapper)} }
                    }
                }
            }
        }
        LoadedComponentInput::LoadedSuccess(c) => c.element(SuccessMapper),
        LoadedComponentInput::LoadedError(e) => {
            let ph = T::place_holder();
            log::error!("{e}");
            if T::visible_place_holder() {
                ph.element(ErrorMapper(e))
            } else {
                rsx! {
                    span { class: "skeleton ",
                        span { class: "invisible ", {ph.element(ErrorMapper(e))} }
                    }
                }
            }
        }
    }
}

/// A component that renders a [`LoadedElement`] directly in the success state.
///
/// This component is useful for elements that are always available and don't need
/// loading state handling, but should still benefit from the [`LoadedElement`] trait
/// capabilities and styling consistency with other loaded components.
///
/// # Type Parameters
///
/// * `T` - A type that implements [`LoadedElement`]
///
/// # Properties
///
/// * `input` - The element to render directly in the success state
///
/// # Examples
///
/// ```rust
/// let profile = UserProfile::new("Alex", "alex@example.com");
///
/// rsx! {
///     AlwaysLoadedComponent { input: profile }
/// }
/// ```
#[component]
pub fn AlwaysLoadedComponent<T: LoadedElement>(input: T) -> Element {
    input.element(SuccessMapper)
}
