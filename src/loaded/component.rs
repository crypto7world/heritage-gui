use dioxus::prelude::*;

use super::{element::LoadedElement, loaders::Loader};

#[derive(Debug, Clone, PartialEq)]
pub enum LoadedComponentInput<T: LoadedElement + Clone + PartialEq> {
    Loading,
    LoadedSuccess(T),
    LoadedError(String),
}

#[component]
pub fn LoadedComponent<T: LoadedElement>(input: LoadedComponentInput<T>) -> Element {
    match input {
        LoadedComponentInput::Loading => {
            T::Loader::load(T::place_holder().element(super::mapper::Loading))
        }
        LoadedComponentInput::LoadedSuccess(c) => c.element(super::mapper::LoadedSuccess),
        LoadedComponentInput::LoadedError(e) => {
            log::error!("{e}");
            T::Loader::load(T::place_holder().element(super::mapper::LoadedError(e)))
        }
    }
}

#[component]
pub fn AlwaysLoadedComponent<T: LoadedElement>(input: T) -> Element {
    input.element(super::mapper::LoadedSuccess)
}
