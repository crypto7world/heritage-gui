use crate::utils::ArcType;

use super::{
    super::{component::LoadedComponentInput, element::LoadedElement},
    FromRef, RefInto,
};

impl<T: LoadedElement, U: RefInto<T>, E: core::fmt::Display> FromRef<Result<U, E>>
    for LoadedComponentInput<T>
{
    #[inline(always)]
    fn from_ref(value: &Result<U, E>) -> Self {
        match value {
            Ok(u) => Self::LoadedSuccess(u.ref_into()),
            Err(e) => Self::LoadedError(e.to_string()),
        }
    }
}
impl<T: LoadedElement, U: RefInto<LoadedComponentInput<T>>> FromRef<Option<U>>
    for LoadedComponentInput<T>
{
    #[inline(always)]
    fn from_ref(value: &Option<U>) -> Self {
        match value {
            Some(u) => u.ref_into(),
            None => Self::Loading,
        }
    }
}

impl<T: LoadedElement, U: RefInto<T>> FromRef<ArcType<U>> for LoadedComponentInput<T> {
    #[inline(always)]
    fn from_ref(value: &ArcType<U>) -> Self {
        Self::LoadedSuccess(value.as_ref().ref_into())
    }
}
impl<T: LoadedElement, U: RefInto<T>> FromRef<ArcType<[U]>> for LoadedComponentInput<ArcType<[T]>> {
    #[inline(always)]
    fn from_ref(value: &ArcType<[U]>) -> Self {
        Self::LoadedSuccess(value.iter().map(|u| u.ref_into()).collect())
    }
}
impl<T: LoadedElement, U: RefInto<T>> FromRef<Vec<U>> for LoadedComponentInput<Vec<T>> {
    #[inline(always)]
    fn from_ref(value: &Vec<U>) -> Self {
        Self::LoadedSuccess(value.iter().map(|u| u.ref_into()).collect())
    }
}
impl<T: LoadedElement, U, V> FromRef<(U, V)> for LoadedComponentInput<T>
where
    (U, V): RefInto<T>,
{
    #[inline(always)]
    fn from_ref(value: &(U, V)) -> Self {
        Self::LoadedSuccess(value.ref_into())
    }
}

impl<T: LoadedElement, U: RefInto<T>> FromRef<(U,)> for LoadedComponentInput<T> {
    #[inline(always)]
    fn from_ref(value: &(U,)) -> Self {
        Self::LoadedSuccess(value.0.ref_into())
    }
}
