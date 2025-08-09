use std::borrow::Borrow;
use std::convert::Infallible;

use btc_heritage_wallet::heritage_service_api_client::Fingerprint;
use serde::{Deserialize, Serialize};

pub fn log_error<E: core::fmt::Display>(error: E) -> String {
    log::error!("{error}");
    error.to_string()
}
pub fn log_error_ccstr<E: core::fmt::Display>(error: E) -> CCStr {
    let e = error.to_string();
    log::error!("{e}");
    CCStr::from(e)
}

pub async fn async_sleep(timeout_ms: u64) {
    tokio::time::sleep(tokio::time::Duration::from_millis(timeout_ms)).await
}

use btc_heritage_wallet::bitcoin::{Amount, Denomination};
pub fn amount_to_signed(amount: Amount) -> btc_heritage_wallet::bitcoin::SignedAmount {
    amount
        .to_signed()
        .expect("No legal amount of sat can be bigger than MAX_MONEY")
}

pub fn denomination_for_amount(amount: Amount) -> Denomination {
    if amount >= Amount::from_btc(0.1).unwrap() {
        Denomination::Bitcoin
    } else if amount >= Amount::from_sat(100000) {
        Denomination::MilliBitcoin
    } else {
        Denomination::Satoshi
    }
}

use btc_heritage_wallet::bitcoin::FeeRate;
pub fn feerate_sat_per_vb(fee_rate: FeeRate) -> f32 {
    fee_rate.to_sat_per_kwu() as f32 / 250.0
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
pub fn timestamp_to_file_string(ts: u64) -> String {
    chrono::DateTime::from_timestamp(ts as i64, 0)
        .expect("invalid timestamp")
        .format("%Y%m%d%H%M%S")
        .to_string()
}

use btc_heritage_wallet::btc_heritage::HeirConfig;
pub fn heir_config_type_to_string(hc: &HeirConfig) -> &'static str {
    match hc {
        HeirConfig::SingleHeirPubkey(_) => "Public Key (deprecated)",
        HeirConfig::HeirXPubkey(_) => "Extended Public Key",
    }
}

use btc_heritage_wallet::bitcoin::psbt::{Input as PsbtInput, PartiallySignedTransaction};
/// Checks if a PSBT input has taproot signatures
///
/// This function checks both key path and script path signatures for taproot inputs.
pub fn is_taproot_input_signed(input: &PsbtInput) -> bool {
    // Check for taproot key path signature
    // Check for taproot script path signatures
    input.tap_key_sig.is_some() || !input.tap_script_sigs.is_empty()
}
/// Checks if a PSBT input has taproot signatures
///
/// This function checks both key path and script path signatures for taproot inputs.
pub fn is_psbt_fully_signed(psbt: &PartiallySignedTransaction) -> bool {
    psbt.inputs.iter().all(is_taproot_input_signed)
}

// pub type CheapClone<T> = std::sync::Arc<T>;
pub type CheapClone<T> = std::rc::Rc<T>;

#[derive(Debug)]
pub struct EqCheapClone<T: ?Sized>(CheapClone<T>);
impl<T> Clone for EqCheapClone<T> {
    fn clone(&self) -> Self {
        Self(CheapClone::clone(&self.0))
    }
}
impl<T> PartialEq for EqCheapClone<T> {
    fn eq(&self, other: &Self) -> bool {
        CheapClone::ptr_eq(&self.0, &other.0)
    }
}
impl<T> From<CheapClone<T>> for EqCheapClone<T> {
    fn from(value: CheapClone<T>) -> Self {
        EqCheapClone(value)
    }
}
impl<T> From<EqCheapClone<T>> for CheapClone<T> {
    fn from(value: EqCheapClone<T>) -> Self {
        value.0
    }
}
impl<T> core::ops::Deref for EqCheapClone<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        // self.0 .0.deref()
        self.0.deref()
    }
}

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub struct CCStr(CheapClone<str>);
impl Clone for CCStr {
    fn clone(&self) -> Self {
        Self(CheapClone::clone(&self.0))
    }
}

impl Serialize for CCStr {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(self.0.as_ref())
    }
}

impl<'de> Deserialize<'de> for CCStr {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        String::deserialize(deserializer).map(|s| CCStr(CheapClone::from(s)))
    }
}

impl core::str::FromStr for CCStr {
    type Err = Infallible;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(CCStr(CheapClone::from(s)))
    }
}
impl From<String> for CCStr {
    fn from(value: String) -> Self {
        value.parse().unwrap()
    }
}
impl From<&String> for CCStr {
    fn from(value: &String) -> Self {
        value.parse().unwrap()
    }
}
impl From<&str> for CCStr {
    fn from(value: &str) -> Self {
        value.parse().unwrap()
    }
}
impl core::ops::Deref for CCStr {
    type Target = str;

    fn deref(&self) -> &Self::Target {
        self.0.as_ref()
    }
}
impl AsRef<str> for CCStr {
    fn as_ref(&self) -> &str {
        self.0.as_ref()
    }
}

impl core::fmt::Display for CCStr {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}

impl Borrow<str> for CCStr {
    fn borrow(&self) -> &str {
        self.0.as_ref()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct FutureFingerprints {
    pub key_provider: Option<Fingerprint>,
    pub online_wallet: Option<Fingerprint>,
}
impl FutureFingerprints {
    pub fn coherents(self) -> bool {
        match (self.key_provider, self.online_wallet) {
            (Some(fp1), Some(fp2)) => fp1 == fp2,
            _ => true,
        }
    }
}
