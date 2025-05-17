use dioxus::prelude::*;

use super::prelude::{FromRef, RefInto};

pub trait LoadedElement: Clone + PartialEq + 'static {
    type Loader: super::loaders::Loader;

    fn element<M: super::mapper::LoadedComponentInputMapper>(self, m: M) -> Element;

    fn place_holder() -> Self;
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
loaded_str!(crate::utils::ArcStr);
loaded_str!(String);
loaded_str!(&'static str);

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
loaded_iter!(crate::utils::ArcType<[T]>, clone);

#[derive(Debug, Clone, PartialEq)]
pub enum Display<T> {
    Show(T),
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

impl<T> Display<T> {
    pub fn map<U, F: FnOnce(T) -> U>(self, f: F) -> Display<U> {
        match self {
            Display::Show(x) => Display::Show(f(x)),
            Display::None => Display::None,
        }
    }
    pub fn as_ref(&self) -> Display<&T> {
        match self {
            Display::Show(x) => Display::Show(x),
            Display::None => Display::None,
        }
    }
}
