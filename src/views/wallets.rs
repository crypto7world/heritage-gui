use std::{ops::Deref, sync::Arc};

use dioxus::prelude::*;

use super::TitledView;
use crate::{clients::database_mut, utils::log_error};
use btc_heritage_wallet::{DatabaseItem, Wallet};

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

    let wallets = Wallet::all_in_db(&db)
        .map_err(log_error)
        .unwrap_or_default();
    Wallet::set_default_item_name(&mut db, "test".to_owned())
        .map_err(log_error)
        .unwrap_or_default();
    rsx! {
        div { class: "max-w-80 sm:container mx-auto grid grid-cols-1 sm:grid-cols-2 md:grid-cols-3 lg:grid-cols-4 xl:grid-cols-5 2xl:grid-cols-6 gap-12",
            for wallet in wallets {
                WalletItem { key: "{wallet.name()}", wallet: wallet.into() }
            }
            for i in 0..30 {

                div { class: "aspect-square border rounded", "Pouet {i}" }
            }
        }
    }
}

#[derive(Debug, Clone)]
struct WalletWrapper(Arc<Wallet>);
impl PartialEq for WalletWrapper {
    fn eq(&self, other: &Self) -> bool {
        self.0.name() == other.0.name()
    }
}
impl From<Wallet> for WalletWrapper {
    fn from(value: Wallet) -> Self {
        Self(Arc::new(value))
    }
}
impl Deref for WalletWrapper {
    type Target = Wallet;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

#[component]
fn WalletItem(wallet: WalletWrapper) -> Element {
    rsx! {
        div { class: "aspect-square border rounded", "{wallet.name()}" }
    }
}
