use dioxus::prelude::*;

pub trait Loader {
    fn load(children: Element) -> Element;
}
pub struct SkeletonLoader;
impl Loader for SkeletonLoader {
    #[inline(always)]
    fn load(children: Element) -> Element {
        rsx! {
            span { class: "skeleton ",
                span { class: "invisible ", {children} }
            }
        }
    }
}
pub struct TransparentLoader;
impl Loader for TransparentLoader {
    #[inline(always)]
    fn load(children: Element) -> Element {
        children
    }
}
