use crate::prelude::*;

use btc_heritage_wallet::{AnyKeyProvider, BoundFingerprint};

use crate::{
    components::badge::{ExternalDependencyStatus, KeyProviderType},
    state_management::prelude::*,
    utils::CCStr,
};

pub(super) fn keyprovider_status(
    kp: &AnyKeyProvider,
    ledger_has_unregistered_policies: Option<bool>,
) -> (KeyProviderType, ExternalDependencyStatus) {
    match kp {
        AnyKeyProvider::None => (KeyProviderType::None, ExternalDependencyStatus::None),
        AnyKeyProvider::LocalKey(local_key) => (
            KeyProviderType::LocalKey,
            if local_key.is_ready() {
                ExternalDependencyStatus::Available
            } else {
                ExternalDependencyStatus::NeedUserAction
            },
        ),
        AnyKeyProvider::Ledger(_) => (
            KeyProviderType::Ledger,
            match state_management::LEDGER_STATUS() {
                Some(LedgerStatus::Ready(fg)) if fg == kp.fingerprint().unwrap() => {
                    if ledger_has_unregistered_policies.is_some_and(|b| b) {
                        ExternalDependencyStatus::NeedUserAction
                    } else {
                        ExternalDependencyStatus::Available
                    }
                }
                _ => ExternalDependencyStatus::Unavailable,
            },
        ),
    }
}

pub fn use_memo_resource<T: Clone + PartialEq>(r: Resource<T>) -> Memo<Option<T>> {
    use_memo(move || r())
}

pub type LResult<T> = Option<Result<T, CCStr>>;
pub type FResource<T> = Resource<Result<T, CCStr>>;
pub type FMemo<T> = Memo<Option<Result<T, CCStr>>>;

pub trait LoadableMapper<T> {
    fn lmap<R, F: FnOnce(&T) -> R>(&self, f: F) -> Option<R>;
}
impl<T> LoadableMapper<T> for Option<T> {
    fn lmap<R, F: FnOnce(&T) -> R>(&self, f: F) -> Option<R> {
        self.as_ref().map(f)
    }
}
impl<T: 'static> LoadableMapper<T> for AsyncSignal<T> {
    fn lmap<R, F: FnOnce(&T) -> R>(&self, f: F) -> Option<R> {
        self.read().as_ref().map(f)
    }
}
impl<T> LoadableMapper<T> for Resource<T> {
    fn lmap<R, F: FnOnce(&T) -> R>(&self, f: F) -> Option<R> {
        self.read().as_ref().map(f)
    }
}
impl<T: PartialEq> LoadableMapper<T> for Memo<Option<T>> {
    fn lmap<R, F: FnOnce(&T) -> R>(&self, f: F) -> Option<R> {
        self.read().as_ref().map(f)
    }
}
impl<T> LoadableMapper<T> for Signal<Option<T>> {
    fn lmap<R, F: FnOnce(&T) -> R>(&self, f: F) -> Option<R> {
        self.read().as_ref().map(f)
    }
}
impl<T> LoadableMapper<T> for GlobalSignal<Option<T>> {
    fn lmap<R, F: FnOnce(&T) -> R>(&self, f: F) -> Option<R> {
        self.read().as_ref().map(f)
    }
}

pub trait LoadableFaillibleMapper<T> {
    fn lrmap<R, F: FnOnce(&T) -> R>(&self, f: F) -> Option<Result<R, CCStr>>;
    fn lrmap_ok<R, F: FnOnce(&T) -> R>(&self, f: F) -> Option<R> {
        self.lrmap(f).map(Result::ok).flatten()
    }
}
impl<T> LoadableFaillibleMapper<T> for Option<Result<T, CCStr>> {
    fn lrmap<R, F: FnOnce(&T) -> R>(&self, f: F) -> Option<Result<R, CCStr>> {
        self.as_ref()
            .map(|inner_result| inner_result.as_ref().map(f).map_err(Clone::clone))
    }
}
impl<T: PartialEq> LoadableFaillibleMapper<T> for FMemo<T> {
    fn lrmap<R, F: FnOnce(&T) -> R>(&self, f: F) -> Option<Result<R, CCStr>> {
        self.read()
            .as_ref()
            .map(|inner_result| inner_result.as_ref().map(f).map_err(Clone::clone))
    }
}
impl<T> LoadableFaillibleMapper<T> for FResource<T> {
    fn lrmap<R, F: FnOnce(&T) -> R>(&self, f: F) -> Option<Result<R, CCStr>> {
        self.read()
            .as_ref()
            .map(|inner_result| inner_result.as_ref().map(f).map_err(Clone::clone))
    }
}
