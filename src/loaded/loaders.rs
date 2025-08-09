use dioxus::prelude::*;

/// A trait for defining strategies to display components in loading state.
///
/// Loader implementations determine how placeholder content is rendered
/// when a component is in a loading state. Each `LoadedElement` type
/// associates itself with a specific loader strategy through its
/// `Loader` associated type.
///
/// # Examples
///
/// ```rust
/// use dioxus::prelude::*;
/// use crate::loaded::loaders::Loader;
///
/// struct MyCustomLoader;
/// impl Loader for MyCustomLoader {
///     fn load(children: Element) -> Element {
///         rsx! {
///             div { class: "custom-loading-animation",
///                 span { class: "hidden", {children} }
///             }
///         }
///     }
///
///     fn error(children: Element) -> Element {
///         rsx! {
///             div { class: "custom-error-styling",
///                 span { class: "text-error", {children} }
///             }
///         }
///     }
/// }
/// ```
pub trait Loader {
    /// Transforms a placeholder element into a loading representation.
    ///
    /// # Parameters
    ///
    /// * `children` - The placeholder element to transform
    ///
    /// # Returns
    ///
    /// A new element with loading styles/behaviors applied
    fn load(children: Element) -> Element;

    /// Transforms a placeholder element into an error representation.
    ///
    /// # Parameters
    ///
    /// * `children` - The placeholder element to transform
    ///
    /// # Returns
    ///
    /// A new element with error styles/behaviors applied
    fn error(children: Element) -> Element;
}
/// A loader that displays a skeleton UI during loading.
///
/// This loader wraps content in a skeleton animation that provides
/// a visual indication of content loading. It preserves the size and
/// shape of the content by making the original content invisible while
/// showing an animated loading effect in its place.
///
/// This is typically used for components where maintaining the layout
/// during loading is important for preventing layout shifts.
pub struct SkeletonLoader;
impl Loader for SkeletonLoader {
    #[inline(always)]
    fn load(children: Element) -> Element {
        rsx! {
            span { class: "skeleton",
                span { class: "invisible inline-block", {children} }
            }
        }
    }
    #[inline(always)]
    fn error(children: Element) -> Element {
        rsx! {
            span { class: "bg-error rounded-xs relative",
                span { class: "absolute top-0 left-0 size-full text-error-content text-center",
                    "ERROR"
                }
                span { class: "invisible inline-block", {children} }
            }
        }
    }
}
/// A loader that passes through the placeholder content without modification.
///
/// This loader simply returns the placeholder content as-is, without any
/// additional styling or animation. It's useful for composite components
/// where the child components already handle their own loading states.
///
/// This is typically used for container components or components that
/// delegate loading visualization to their children.
pub struct TransparentLoader;
impl Loader for TransparentLoader {
    #[inline(always)]
    fn load(children: Element) -> Element {
        children
    }
    #[inline(always)]
    fn error(children: Element) -> Element {
        children
    }
}
