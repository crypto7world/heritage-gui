//! Reference-based conversion implementations for `LoadedComponentInput`.
//!
//! This module provides `FromRef` implementations for various types,
//! enabling reference-based conversions to `LoadedComponentInput`
//! without taking ownership of the source data.

use std::{collections::HashMap, marker::PhantomData};

use crate::{loaded::element::Display, utils::CheapClone};

use super::{
    super::{component::LoadedComponentInput, element::LoadedElement},
    FromRef, RefInto,
};

mod private {
    pub trait LoadedSuccessConversionMarkerSeal {}
    impl<From: ?Sized, To: super::LoadedElement + 'static> LoadedSuccessConversionMarkerSeal
        for super::TypeCouple<From, To>
    {
    }
}
pub struct TypeCouple<From: ?Sized, To: LoadedElement + 'static>(
    PhantomData<From>,
    PhantomData<To>,
);
pub trait LoadedSuccessConversionMarker: private::LoadedSuccessConversionMarkerSeal {}

impl<Fr, To: LoadedElement + 'static> LoadedSuccessConversionMarker
    for TypeCouple<CheapClone<Fr>, To>
where
    TypeCouple<Fr, To>: LoadedSuccessConversionMarker,
{
}
impl<Fr, To: LoadedElement + 'static> LoadedSuccessConversionMarker for TypeCouple<[Fr], Vec<To>> where
    TypeCouple<Fr, To>: LoadedSuccessConversionMarker
{
}
impl<Fr, To: LoadedElement + 'static> LoadedSuccessConversionMarker
    for TypeCouple<[Fr], CheapClone<[To]>>
where
    TypeCouple<Fr, To>: LoadedSuccessConversionMarker,
{
}
impl<
        K: Clone + core::hash::Hash + Eq + Default + core::fmt::Display,
        Fr,
        To: LoadedElement + 'static,
    > LoadedSuccessConversionMarker for TypeCouple<HashMap<K, Fr>, HashMap<K, To>>
where
    TypeCouple<Fr, To>: LoadedSuccessConversionMarker,
{
}

impl<Fr, To: LoadedElement + 'static> LoadedSuccessConversionMarker for TypeCouple<Vec<Fr>, To> where
    TypeCouple<[Fr], To>: LoadedSuccessConversionMarker
{
}
impl<Fr, To: LoadedElement + 'static> LoadedSuccessConversionMarker
    for TypeCouple<CheapClone<[Fr]>, To>
where
    TypeCouple<[Fr], To>: LoadedSuccessConversionMarker,
{
}

impl<T: LoadedElement, U: RefInto<T>> FromRef<U> for LoadedComponentInput<T>
where
    TypeCouple<U, T>: LoadedSuccessConversionMarker,
{
    #[inline(always)]
    fn from_ref(value: &U) -> Self {
        Self::LoadedSuccess(value.ref_into())
    }
}

/// Creates a `LoadedComponentInput<T>` from a reference to a `Result`.
///
/// This implementation:
/// - Converts `Ok(u)` to a success state using the `RefInto` implementation of `U`
/// - Converts `Err(e)` to an error state with the error message
///
/// # Type Parameters
///
/// * `T` - The component type that implements `LoadedElement`
/// * `U` - The success type that can be converted to `T`
/// * `E` - The error type that can be displayed
///
/// # Examples
///
/// ```rust
/// use crate::loaded::prelude::*;
///
/// struct ApiResult {
///     result: Result<UserData, ApiError>
/// }
///
/// // In component:
/// let input: LoadedComponentInput<UserView> = api_result.result.ref_into();
/// ```
impl<T: LoadedElement, U: RefInto<LoadedComponentInput<T>>, E: core::fmt::Display>
    FromRef<Result<U, E>> for LoadedComponentInput<T>
{
    #[inline(always)]
    fn from_ref(value: &Result<U, E>) -> Self {
        match value {
            Ok(u) => u.ref_into(),
            Err(e) => Self::LoadedError(e.to_string()),
        }
    }
}
/// Creates a `LoadedComponentInput<T>` from a reference to an `Option`.
///
/// This implementation:
/// - Converts `Some(u)` using the `RefInto<LoadedComponentInput<T>>` implementation of `U`
/// - Converts `None` to a loading state
///
/// # Type Parameters
///
/// * `T` - The component type that implements `LoadedElement`
/// * `U` - A type that can be converted to `LoadedComponentInput<T>`
///
/// # Examples
///
/// ```rust
/// use crate::loaded::prelude::*;
///
/// // In component:
/// let resource: Option<Result<UserData, Error>> = None;
/// let input: LoadedComponentInput<UserView> = resource.ref_into();
/// // input will be Loading
/// ```
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

/// Creates a `LoadedComponentInput<T>` from a reference to an `Arc<U>`.
///
/// This implementation converts the inner value of the `ArcType` to `T`
/// using its `RefInto<T>` implementation.
///
/// # Type Parameters
///
/// * `T` - The component type that implements `LoadedElement`
/// * `U` - A type that can be converted to `T`
///
/// # Examples
///
/// ```rust
/// use crate::loaded::prelude::*;
/// use crate::utils::ArcType;
///
/// let data: Arc<UserData> = /* ... */;
/// let input: LoadedComponentInput<UserView> = data.ref_into();
/// ```
// impl<T: LoadedElement, U: RefInto<LoadedComponentInput<T>>> FromRef<CheapClone<U>>
//     for LoadedComponentInput<T>
// {
//     #[inline(always)]
//     fn from_ref(value: &CheapClone<U>) -> Self {
//         value.as_ref().ref_into()
//     }
// }

/// Creates a `LoadedComponentInput<Arc<[T]>>` from a reference to an `Arc<[U]>`.
///
/// This implementation converts each element in the source slice to type `T`
/// using their `RefInto<T>` implementations, and collects them into a new `Arc<[T]>`.
///
/// # Type Parameters
///
/// * `T` - The component type that implements `LoadedElement`
/// * `U` - A type that can be converted to `T`
///
/// # Examples
///
/// ```rust
/// use crate::loaded::prelude::*;
/// use crate::utils::ArcType;
///
/// let user_list: Arc<[UserData]> = /* ... */;
/// let input: LoadedComponentInput<Arc<[UserView]>> = user_list.ref_into();
/// ```
// impl<T: LoadedElement, U: RefInto<T>> FromRef<CheapClone<[U]>>
//     for LoadedComponentInput<CheapClone<[T]>>
// {
//     #[inline(always)]
//     fn from_ref(value: &CheapClone<[U]>) -> Self {
//         Self::LoadedSuccess(value.iter().map(|u| u.ref_into()).collect())
//     }
// }

/// Creates a `LoadedComponentInput<Vec<T>>` from a reference to a `Vec<U>`.
///
/// This implementation converts each element in the source vector to type `T`
/// using their `RefInto<T>` implementations, and collects them into a new `Vec<T>`.
///
/// # Type Parameters
///
/// * `T` - The component type that implements `LoadedElement`
/// * `U` - A type that can be converted to `T`
///
/// # Examples
///
/// ```rust
/// use crate::loaded::prelude::*;
///
/// let transactions: Vec<Transaction> = /* ... */;
/// let input: LoadedComponentInput<Vec<TransactionRow>> = transactions.ref_into();
/// ```
// impl<T: LoadedElement, U: RefInto<T>> FromRef<Vec<U>> for LoadedComponentInput<Vec<T>> {
//     #[inline(always)]
//     fn from_ref(value: &Vec<U>) -> Self {
//         Self::LoadedSuccess(value.iter().map(|u| u.ref_into()).collect())
//     }
// }

/// Creates a `LoadedComponentInput<T>` from a reference to a 1-uple `(U,)`.
// impl<T: LoadedElement, U> FromRef<(U,)> for LoadedComponentInput<T>
// where
//     U: RefInto<T>,
// {
//     #[inline(always)]
//     fn from_ref(value: &(U,)) -> Self {
//         Self::LoadedSuccess(value.0.ref_into())
//     }
// }

/// Creates a `LoadedComponentInput<T>` from a reference to a tuple `(U, V)`.
///
/// This implementation uses the `RefInto<T>` implementation of the tuple
/// to convert it to a component of type `T`.
///
/// # Type Parameters
///
/// * `T` - The component type that implements `LoadedElement`
/// * `U`, `V` - Types that together can be converted to `T`
///
/// # Examples
///
/// ```rust
/// use crate::loaded::prelude::*;
///
/// // In component, where StatusBadge can be created from (StatusType, ConnectionState)
/// let status_data = (StatusType::Online, ConnectionState::Secure);
/// let input: LoadedComponentInput<StatusBadge> = status_data.ref_into();
/// ```
// impl<T: LoadedElement, U, V> FromRef<(U, V)> for LoadedComponentInput<T>
// where
//     (U, V): RefInto<T>,
// {
//     #[inline(always)]
//     fn from_ref(value: &(U, V)) -> Self {
//         Self::LoadedSuccess(value.ref_into())
//     }
// }

/// Creates a `LoadedComponentInput<Display<T>>` from a reference to a `Display<U>`.
///
/// This implementation preserves the `Display` structure, converting the inner
/// value from `U` to `T` using its `RefInto<T>` implementation if it exists.
///
/// # Type Parameters
///
/// * `T` - The component type that implements `LoadedElement`
/// * `U` - A type that can be converted to `T`
///
/// # Examples
///
/// ```rust
/// use crate::loaded::prelude::*;
///
/// let detail = Display::Show(UserDetail { /* ... */ });
/// let input: LoadedComponentInput<Display<UserDetailView>> = detail.ref_into();
///
/// // Or with None:
/// let no_detail: Display<UserDetail> = Display::None;
/// let input: LoadedComponentInput<Display<UserDetailView>> = no_detail.ref_into();
/// ```
impl<T: LoadedElement, U: RefInto<T>> FromRef<Display<U>> for LoadedComponentInput<Display<T>> {
    #[inline(always)]
    fn from_ref(value: &Display<U>) -> Self {
        Self::LoadedSuccess(value.as_ref().map(|u| u.ref_into()))
    }
}
