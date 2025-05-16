//! A framework for managing UI components with loading, success, and error states.
//!
//! This module provides a set of traits and implementations that make it easier to work with
//! asynchronously loaded data in Dioxus applications. The primary goal is to provide a
//! clean and consistent way to display components that need to:
//!
//! 1. Show a placeholder while data is loading
//! 2. Render the actual component when data is successfully loaded
//! 3. Show an error state when loading fails
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
//!     fn element(&self) -> Element {
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

pub mod badge;
pub mod balance;
pub mod timestamp;

use dioxus::prelude::*;

use crate::utils::{ArcStr, ArcType};

/// The ImplDirectIntoLoadedElementInputMarker trait that designate types for which we want to implement:
/// - impl<T: LoadedElement + Clone + PartialEq, R: RefInto<T> + ImplDirectIntoLoadedElementInputMarker> FromRef<R> for LoadedComponentInput<T>
/// - impl<T: LoadedElement + Clone + PartialEq, R: Into<T> + ImplDirectIntoLoadedElementInputMarker> From<R> for LoadedComponentInput<T>
/// This marker is there to prevent overlapping with the impl for Option<T> and Result<T, E> (because, no specialization or negative trait bound is possible right now)
pub trait ImplDirectIntoLoadedElementInputMarker {}
impl ImplDirectIntoLoadedElementInputMarker for btc_heritage_wallet::Wallet {}

/// A trait for elements that can be displayed in both loaded and placeholder states.
///
/// This trait is implemented by UI components that need to handle loading states,
/// providing both a fully rendered view and a placeholder view.
pub trait LoadedElement: Clone + PartialEq + 'static {
    /// Renders the component as a Dioxus Element.
    ///
    /// This method is called when the component is in the loaded state.
    fn element<CM: ComponentMapper>(self, mapper: CM) -> Element;

    /// Creates a placeholder instance of this component.
    ///
    /// The placeholder is used when data is still loading or when an error occurs.
    fn place_holder() -> Self;

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
/// This trait is similar to the standard `From` trait, but works with references.
/// It's particularly useful when working with Dioxus state primitives like
/// `Resource`, `Signal`, and `Memo`, which provide access to their values through
/// references.
///
/// # Example
///
/// ```
/// impl FromRef<UserData> for UIUserProfile {
///     fn from_ref(data: &UserData) -> Self {
///         Self {
///             display_name: data.name.clone(),
///             avatar_url: data.avatar.clone(),
///         }
///     }
/// }
/// ```
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
/// This trait is the complement to `FromRef` and works like the standard
/// `Into` trait but for references. It's automatically implemented for any
/// type that implements `FromRef`.
///
/// # Example
///
/// ```
/// // With FromRef implemented for UIUserProfile:
/// let user_data = get_user_data();
/// let ui_profile: UIUserProfile = user_data.ref_into();
/// ```
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
/// This enum serves as an intermediate representation for components that
/// can be in one of three states:
/// - `Loading`: The data is currently being loaded
/// - `LoadedSuccess`: The data was successfully loaded
/// - `LoadedError`: An error occurred while loading the data
///
/// It's designed to work with the `LoadedComponent` to provide a consistent
/// UI experience for asynchronously loaded components.
///
/// # Type Parameters
///
/// * `T` - A type that implements `LoadedElement` and can be cloned and compared
#[derive(Debug, Clone, PartialEq)]
pub enum LoadedComponentInput<T: LoadedElement + Clone + PartialEq> {
    /// The component is in a loading state.
    Loading,
    /// The component has been successfully loaded.
    LoadedSuccess(T),
    /// An error occurred while loading the component.
    LoadedError(String),
}

pub trait ComponentMapper {
    fn map<T: LoadedElement>(&self, t: T) -> LoadedComponentInput<T>;
}
pub struct LoadingMapper;
impl ComponentMapper for LoadingMapper {
    #[inline(always)]
    fn map<T: LoadedElement>(&self, _t: T) -> LoadedComponentInput<T> {
        LoadedComponentInput::Loading
    }
}
pub struct ErrorMapper(String);
impl ComponentMapper for ErrorMapper {
    #[inline(always)]
    fn map<T: LoadedElement>(&self, _t: T) -> LoadedComponentInput<T> {
        LoadedComponentInput::LoadedError(self.0.clone())
    }
}
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
/// `LoadedComponentInput` and renders the appropriate UI based on the input's state:
///
/// - For the `Loading` state, it displays a placeholder with optional skeleton loading effect
/// - For the `LoadedSuccess` state, it displays the actual component
/// - For the `LoadedError` state, it logs the error and displays the placeholder
///
/// The skeleton effect is controlled by the `grim_place_holder()` method on the
/// `LoadedElement` trait. When it returns `true` (the default), the placeholder is
/// wrapped in a skeleton div that provides a loading animation. When it returns `false`,
/// the placeholder is displayed directly.
///
/// # Type Parameters
///
/// * `T` - A type that implements `LoadedElement` and can be cloned and compared
///
/// # Examples
///
/// ```
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

#[component]
pub fn AlwaysLoadedComponent<T: LoadedElement>(input: T) -> Element {
    input.element(SuccessMapper)
}
