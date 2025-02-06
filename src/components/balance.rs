use dioxus::prelude::*;

use btc_heritage_wallet::{
    bitcoin::{Amount, Denomination},
    btc_heritage::HeritageWalletBalance,
};

#[derive(Debug, Clone, PartialEq, Eq)]
enum DisplayBalance {
    NoValue,
    Refreshing,
    Value(HeritageWalletBalance),
}

fn amount_to_string(amount: Amount) -> String {
    if amount >= Amount::from_btc(0.1).unwrap() {
        format!("{} BTC", amount.display_in(Denomination::Bitcoin))
    } else if amount >= Amount::from_sat(10000) {
        format!("{} mBTC", amount.display_in(Denomination::MilliBitcoin))
    } else {
        format!("{} sat", amount.display_in(Denomination::Satoshi))
    }
}

#[component]
pub fn Balance(balance: DisplayBalance) -> Element {
    let wallet_balance = match wallet_balance {
        Some(a) => {
            if a >= Amount::from_btc(0.1).unwrap() {
                format!("{} BTC", a.display_in(Denomination::Bitcoin))
            } else if a >= Amount::from_sat(10000) {
                format!("{} mBTC", a.display_in(Denomination::MilliBitcoin))
            } else {
                format!("{} sat", a.display_in(Denomination::Satoshi))
            }
        }
        None => "-".to_owned(),
    };

    rsx! {
        div { class: "text-4xl font-black text-center", {wallet_balance} }
    }
}
