use std::{convert::Infallible, ops::Deref, str::FromStr, sync::Arc};

use btc_heritage_wallet::bitcoin::{Amount, Denomination, SignedAmount};
use dioxus::signals::ReadableOptionExt;

use crate::helper_hooks::async_init::AsyncSignal;

pub fn log_error<E: core::fmt::Display>(error: E) -> String {
    log::error!("{error}");
    error.to_string()
}

pub fn amount_to_signed_string(amount: SignedAmount) -> String {
    let abs_amount = amount
        .checked_abs()
        .unwrap_or_default()
        .to_unsigned()
        .unwrap();
    if abs_amount >= Amount::from_btc(0.1).unwrap() {
        format!("{:+} BTC", amount.display_in(Denomination::Bitcoin))
    } else if abs_amount >= Amount::from_sat(100000) {
        format!("{:+} mBTC", amount.display_in(Denomination::MilliBitcoin))
    } else {
        format!("{:+} sat", amount.display_in(Denomination::Satoshi))
    }
}

pub fn timestamp_to_string(ts: u64) -> String {
    chrono::DateTime::from_timestamp(ts as i64, 0)
        .expect("invalid timestamp")
        .to_string()
}

pub fn timestamp_to_date_string(ts: u64) -> String {
    chrono::DateTime::from_timestamp(ts as i64, 0)
        .expect("invalid timestamp")
        .date_naive()
        .to_string()
}

pub async fn wait_resource<T: 'static>(resource: dioxus::hooks::Resource<T>) {
    while !resource.finished() {
        tokio::time::sleep(std::time::Duration::from_millis(1)).await;
    }
}

pub async fn wait_async_signal<T: 'static>(async_signal: AsyncSignal<T>) {
    while !async_signal.finished() {
        tokio::time::sleep(std::time::Duration::from_millis(1)).await;
    }
}

#[derive(Debug)]
pub struct EqArcType<T: ?Sized>(ArcType<T>);
impl<T> Clone for EqArcType<T> {
    fn clone(&self) -> Self {
        Self(ArcType::clone(&self.0))
    }
}
impl<T> PartialEq for EqArcType<T> {
    fn eq(&self, other: &Self) -> bool {
        Arc::ptr_eq(&self.0 .0, &other.0 .0)
    }
}
impl<T> From<ArcType<T>> for EqArcType<T> {
    fn from(value: ArcType<T>) -> Self {
        EqArcType(value)
    }
}
impl<T> From<EqArcType<T>> for ArcType<T> {
    fn from(value: EqArcType<T>) -> Self {
        value.0
    }
}
impl<T> Deref for EqArcType<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        self.0 .0.deref()
    }
}
#[derive(Debug, PartialEq, Eq, Hash)]
pub struct ArcType<T: ?Sized>(Arc<T>);

impl<T: ?Sized> Clone for ArcType<T> {
    fn clone(&self) -> Self {
        Self(Arc::clone(&self.0))
    }
}

impl<T: ?Sized + core::fmt::Display> core::fmt::Display for ArcType<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}
impl<T> ArcType<T> {
    pub fn new(value: T) -> Self {
        Self(Arc::new(value))
    }
}
impl<T> From<T> for ArcType<T> {
    fn from(value: T) -> Self {
        Self(Arc::from(value))
    }
}
impl<T, I: IntoIterator<Item = T>> From<I> for ArcType<[T]> {
    fn from(value: I) -> Self {
        Self(Arc::from_iter(value))
    }
}
impl<T> FromIterator<T> for ArcType<[T]> {
    fn from_iter<I: IntoIterator<Item = T>>(iter: I) -> Self {
        Self(Arc::from_iter(iter))
    }
}

impl<T> From<ArcType<T>> for Arc<T> {
    fn from(value: ArcType<T>) -> Self {
        value.0
    }
}
impl<T: ?Sized> Deref for ArcType<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        self.0.deref()
    }
}
impl<T: ?Sized> AsRef<T> for ArcType<T> {
    fn as_ref(&self) -> &T {
        self.0.as_ref()
    }
}

pub type ArcStr = ArcType<str>;
impl FromStr for ArcType<str> {
    type Err = Infallible;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(ArcType(Arc::from(s)))
    }
}
impl From<String> for ArcType<str> {
    fn from(value: String) -> Self {
        Self::from_str(value.as_str()).unwrap()
    }
}
impl From<&String> for ArcType<str> {
    fn from(value: &String) -> Self {
        Self::from_str(value.as_str()).unwrap()
    }
}
impl From<&str> for ArcType<str> {
    fn from(value: &str) -> Self {
        Self::from_str(value).unwrap()
    }
}
