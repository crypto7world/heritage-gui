use std::collections::{BTreeMap, HashMap};

use dioxus::prelude::*;

/// A trait representing UI components that can handle loading states.
///
/// This trait is the foundation of the loaded framework. By implementing this trait,
/// a type declares that it can be displayed in different loading states:
/// loading, loaded successfully, or error.
///
/// # Type Parameters
///
/// * `Loader` - The loader strategy used to display the component when in a loading state
///
/// # Examples
///
/// ```rust
/// use crate::loaded::prelude::*;
///
/// #[derive(Debug, Clone, PartialEq)]
/// struct UserBadge {
///     name: String,
///     role: String,
/// }
///
/// impl LoadedElement for UserBadge {
///     type Loader = SkeletonLoader;
///
///     fn element<M: LoadedComponentInputMapper>(self, _m: M) -> Element {
///         rsx! {
///             div { class: "badge",
///                 span { {self.name} }
///                 span { class: "role", {self.role} }
///             }
///         }
///     }
///
///     fn place_holder() -> Self {
///         Self {
///             name: "Loading...".to_string(),
///             role: "...".to_string(),
///         }
///     }
/// }
/// ```
pub trait LoadedElement: Clone + PartialEq + 'static {
    /// The loader strategy to use when displaying this component in a loading state.
    ///
    /// Common options are:
    /// - `SkeletonLoader`: Shows a skeleton UI with animated loading effect
    /// - `TransparentLoader`: Passes through the placeholder without additional styling
    type Loader: super::loaders::Loader;

    /// Renders the element to UI with the given mapper.
    ///
    /// This method defines how the component should be rendered when in a successful state.
    /// The mapper parameter is used to transform nested components based on the parent's state.
    ///
    /// # Parameters
    ///
    /// * `self` - The component instance to render
    /// * `m` - A mapper that can convert nested components to their appropriate loading state
    ///
    /// # Returns
    ///
    /// A Dioxus Element representing the rendered UI
    fn element<M: super::mapper::LoadedComponentInputMapper>(self, m: M) -> Element;

    /// Creates a placeholder instance to show during loading.
    ///
    /// This method should return a representation of the component with sensible
    /// placeholder values that can be shown while the actual data is loading.
    ///
    /// # Returns
    ///
    /// A placeholder instance of the component
    fn place_holder() -> Self;
}

impl LoadedElement for () {
    type Loader = super::loaders::SkeletonLoader;
    #[inline(always)]
    fn element<M: super::mapper::LoadedComponentInputMapper>(self, _m: M) -> Element {
        rsx! {}
    }

    fn place_holder() -> Self {
        ()
    }
}

// Default and blancket impls
macro_rules! loaded_str {
    ($t:ty ) => {
        impl LoadedElement for $t {
            type Loader = super::loaders::SkeletonLoader;
            #[inline(always)]
            fn element<M: super::mapper::LoadedComponentInputMapper>(self, _m: M) -> Element {
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
loaded_str!(crate::utils::CCStr);
loaded_str!(&'static str);

macro_rules! loaded_integers {
    ($t:ty ) => {
        impl LoadedElement for $t {
            type Loader = super::loaders::SkeletonLoader;
            #[inline(always)]
            fn element<M: super::mapper::LoadedComponentInputMapper>(self, _m: M) -> Element {
                rsx! {
                    "{self}"
                }
            }

            fn place_holder() -> Self {
                123
            }
        }
    };
}
loaded_integers!(u8);
loaded_integers!(i8);
loaded_integers!(u16);
loaded_integers!(i16);
loaded_integers!(u32);
loaded_integers!(i32);
loaded_integers!(u64);
loaded_integers!(i64);
loaded_integers!(usize);
loaded_integers!(u128);
loaded_integers!(i128);

macro_rules! loaded_iter {
    ($t:ty $(, $clone:expr)?) => {
        impl<T: LoadedElement> LoadedElement for $t {
            type Loader = super::loaders::TransparentLoader;
            #[inline(always)]
            fn element<M: super::mapper::LoadedComponentInputMapper>(self, m: M) -> Element {
                rsx! {
                    for item in self {
                        super::component::LoadedComponent::<T> { input: m.map(item$(. $clone())?) }
                    }
                }
            }

            fn place_holder() -> Self {
                <$t>::from_iter(Some(T::place_holder()))
            }
        }
    };
}
loaded_iter!(Vec<T>);
loaded_iter!(crate::utils::CheapClone<[T]>, clone);

macro_rules! loaded_maps {
    ($t:ty $(, $k_bound:path)*) => {
        impl<
                K: core::fmt::Display + Default + Clone + 'static $(+ $k_bound)*,
                T: LoadedElement,
            > LoadedElement for $t
        {
            type Loader = super::loaders::TransparentLoader;
            #[inline(always)]
            fn element<M: super::mapper::LoadedComponentInputMapper>(self, m: M) -> Element {
                rsx! {
                    for (key , item) in self {
                        super::component::LoadedComponent::<T> { key: "{key}", input: m.map(item) }
                    }
                }
            }

            fn place_holder() -> Self {
                Self::from([(K::default(), T::place_holder())])
            }
        }
    };
}
loaded_maps!(HashMap<K, T>, core::hash::Hash, Eq);
loaded_maps!(BTreeMap<K, T>, Ord);

/// A wrapper type that conditionally displays a component.
///
/// `Display<T>` provides a way to conditionally display a component of type `T`.
/// It can either show the component (`Show` variant) or display nothing (`None` variant).
///
/// This is useful for cases where a component may or may not be present based on some condition.
///
/// # Type Parameters
///
/// * `T` - The type of the component to conditionally display
///
/// # Examples
///
/// ```rust
/// use crate::loaded::prelude::*;
///
/// #[component]
/// fn ConditionalUI(show_details: bool) -> Element {
///     let details = if show_details {
///         Display::Show(DetailComponent { /* ... */ })
///     } else {
///         Display::None
///     };
///
///     rsx! {
///         div {
///             h1 { "Main Content" }
///             LoadedComponent { input: details }
///         }
///     }
/// }
/// ```
#[derive(Debug, Clone, PartialEq, Default)]
pub enum Display<T> {
    /// Display the contained value
    Show(T),
    /// Display nothing
    #[default]
    None,
}
impl<T: LoadedElement> LoadedElement for Display<T> {
    type Loader = super::loaders::TransparentLoader;
    #[inline(always)]
    fn element<M: super::mapper::LoadedComponentInputMapper>(self, m: M) -> Element {
        rsx! {
            if let Display::Show(t) = self {
                super::component::LoadedComponent::<T> { input: m.map(t) }
            }
        }
    }
    fn place_holder() -> Self {
        Display::Show(T::place_holder())
    }
}

impl<T: LoadedElement> From<Option<T>> for Display<T> {
    fn from(value: Option<T>) -> Self {
        match value {
            Some(t) => Self::Show(t),
            None => Self::None,
        }
    }
}

impl<T> Display<T> {
    /// Maps the contained value using the provided function.
    ///
    /// This is similar to `Option::map`. If the `Display` is `Show`, it applies the function to the
    /// contained value and returns a new `Display::Show` with the result. If it's `None`, it returns
    /// `Display::None`.
    ///
    /// # Parameters
    ///
    /// * `f` - The function to apply to the contained value
    ///
    /// # Returns
    ///
    /// A new `Display` with the mapped value
    ///
    /// # Examples
    ///
    /// ```rust
    /// use crate::loaded::prelude::*;
    ///
    /// let display = Display::Show("hello");
    /// let mapped = display.map(|s| s.to_uppercase());
    /// assert_eq!(mapped, Display::Show("HELLO"));
    /// ```
    pub fn map<U, F: FnOnce(T) -> U>(self, f: F) -> Display<U> {
        match self {
            Display::Show(x) => Display::Show(f(x)),
            Display::None => Display::None,
        }
    }

    /// Creates a reference view of the `Display`.
    ///
    /// This method creates a new `Display` that contains a reference to the original value,
    /// rather than taking ownership of it.
    ///
    /// # Returns
    ///
    /// A new `Display` containing a reference to the original value
    ///
    /// # Examples
    ///
    /// ```rust
    /// use crate::loaded::prelude::*;
    ///
    /// let display = Display::Show("hello".to_owned());
    /// let ref_display = display.as_ref();
    /// // Now we can use both display and ref_display
    /// ```
    pub fn as_ref(&self) -> Display<&T> {
        match self {
            Display::Show(x) => Display::Show(x),
            Display::None => Display::None,
        }
    }
}
