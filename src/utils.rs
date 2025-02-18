use btc_heritage_wallet::bitcoin::{Amount, Denomination};

pub fn log_error<E: core::fmt::Display>(error: E) -> String {
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
    chrono::DateTime::from_timestamp(ts as i64, 0)
        .expect("invalid timestamp")
        .to_string()
}

pub async fn wait_resource<T: 'static>(resource: dioxus::hooks::Resource<T>) {
    while !resource.finished() {
        tokio::time::sleep(std::time::Duration::from_millis(1)).await;
    }
}
