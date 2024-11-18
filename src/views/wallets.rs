use btc_heritage_wallet::{DatabaseItem, Wallet};
use dioxus::prelude::*;

use super::TitledView;
use crate::clients::database_mut;

#[component]
pub fn WalletListView() -> Element {
    rsx! {
        TitledView {
            title: "Wallets",
            subtitle: "Heritage wallets with simple Heritage configurations instead of complex Bitcoin scripts.",
            WalletList {}
        }
    }
}

#[component]
fn WalletList() -> Element {
    let mut db = database_mut();
    let wallet_names = Wallet::list_names(&db).unwrap_or_default();

    Wallet::set_default_item_name(&mut db, "test".to_owned()).unwrap_or_default();

    rsx! {
        ul {
            for wallet_name in wallet_names {
                li { {wallet_name} }
            }
        }
    }
}
