use std::error::Error;

use btc_heritage_wallet::bitcoin::{Amount, Denomination};
use chrono::DateTime;

pub fn log_error<E: Error>(error: E) -> String {
    log::error!("{error}");
    error.to_string()
}

pub fn amount_to_string(amount: Amount) -> String {
    if amount >= Amount::from_btc(0.1).unwrap() {
        format!("{} BTC", amount.display_in(Denomination::Bitcoin))
    } else if amount >= Amount::from_sat(10000) {
        format!("{} mBTC", amount.display_in(Denomination::MilliBitcoin))
    } else {
        format!("{} sat", amount.display_in(Denomination::Satoshi))
    }
}

pub fn timestamp_to_string(ts: u64) -> String {
    DateTime::from_timestamp(ts as i64, 0)
        .expect("invalid timestamp")
        .to_string()
}
