use dioxus::{
    hooks::Resource,
    signals::{Memo, ReadOnlySignal, Readable},
};

use super::{
    super::{component::LoadedComponentInput, element::LoadedElement},
    RefInto,
};

impl<T: LoadedElement, U> From<Resource<U>> for LoadedComponentInput<T>
where
    Option<U>: RefInto<LoadedComponentInput<T>>,
{
    fn from(value: Resource<U>) -> Self {
        (&*value.read()).ref_into()
    }
}

impl<T: LoadedElement, U: RefInto<LoadedComponentInput<T>>> From<ReadOnlySignal<U>>
    for LoadedComponentInput<T>
{
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
