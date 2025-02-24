use std::{convert::Infallible, ops::Deref, rc::Rc, str::FromStr};

use btc_heritage_wallet::{
    bitcoin::{self, Amount, Denomination, SignedAmount},
    btc_heritage::bdk_types::BlockTime,
};

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

pub trait PlaceHolder {
    fn place_holder() -> Self;
}
impl PlaceHolder for SignedAmount {
    fn place_holder() -> Self {
        SignedAmount::from_btc(12345.0).expect("12345.0 cannot fail")
    }
}
impl PlaceHolder for bitcoin::Txid {
    fn place_holder() -> Self {
        bitcoin::Txid::from_str("5df6e0e2761359d30a8275058e299fcc0381534545f55cf43e41983f5d4c9456")
            .expect("is a valid txid, cannot fail")
    }
}
impl PlaceHolder for Option<BlockTime> {
    fn place_holder() -> Self {
        BlockTime::new(Some(0), Some(0))
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum LoadedElement<T: PlaceHolder> {
    Loaded(T),
    Loading,
}
impl<T: PlaceHolder + Copy> Copy for LoadedElement<T> {}

impl<T: PlaceHolder> LoadedElement<T> {
    pub fn map<U: PlaceHolder, F>(self, f: F) -> LoadedElement<U>
    where
        F: FnOnce(T) -> U,
    {
        match self {
            LoadedElement::Loaded(e) => LoadedElement::Loaded(f(e)),
            LoadedElement::Loading => LoadedElement::Loading,
        }
    }
    /// Consume the [LoadedElement] and return a tuple (is_placeholder, value) of type [(bool, T)]
    pub fn extract(self) -> (bool, T) {
        match self {
            LoadedElement::Loaded(value) => (false, value),
            LoadedElement::Loading => (true, T::place_holder()),
        }
    }
}

impl<T: PlaceHolder + Default> Default for LoadedElement<T> {
    fn default() -> Self {
        LoadedElement::Loaded(T::default())
    }
}

impl<T: PlaceHolder> From<T> for LoadedElement<T> {
    fn from(value: T) -> Self {
        LoadedElement::Loaded(value)
    }
}

#[derive(Debug, Clone)]
pub struct EqRcType<T>(RcType<T>);
impl<T> PartialEq for EqRcType<T> {
    fn eq(&self, other: &Self) -> bool {
        Rc::ptr_eq(&self.0 .0, &other.0 .0)
    }
}
impl<T> From<RcType<T>> for EqRcType<T> {
    fn from(value: RcType<T>) -> Self {
        EqRcType(value)
    }
}
impl<T> From<EqRcType<T>> for RcType<T> {
    fn from(value: EqRcType<T>) -> Self {
        value.0
    }
}

#[derive(Debug, PartialEq, Eq, Hash)]
pub struct RcType<T>(Rc<T>);

impl<T> Clone for RcType<T> {
    fn clone(&self) -> Self {
        Self(Rc::clone(&self.0))
    }
}

impl<T: core::fmt::Display> core::fmt::Display for RcType<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}

impl<T> From<T> for RcType<T> {
    fn from(value: T) -> Self {
        Self(Rc::new(value))
    }
}
impl<T> From<RcType<T>> for Rc<T> {
    fn from(value: RcType<T>) -> Self {
        value.0
    }
}
impl<T> Deref for RcType<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        self.0.deref()
    }
}

#[derive(Debug, PartialEq)]
pub struct RcStr(Rc<str>);
impl core::fmt::Display for RcStr {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}
impl Clone for RcStr {
    fn clone(&self) -> Self {
        Self(Rc::clone(&self.0))
    }
}
impl FromStr for RcStr {
    type Err = Infallible;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(RcStr(Rc::from(s)))
    }
}
impl From<RcStr> for Rc<str> {
    fn from(value: RcStr) -> Self {
        value.0
    }
}
impl From<Rc<str>> for RcStr {
    fn from(value: Rc<str>) -> Self {
        RcStr(value)
    }
}
impl From<String> for RcStr {
    fn from(value: String) -> Self {
        RcStr(Rc::from(value))
    }
}
impl Deref for RcStr {
    type Target = str;

    fn deref(&self) -> &Self::Target {
        self.0.deref()
    }
}
