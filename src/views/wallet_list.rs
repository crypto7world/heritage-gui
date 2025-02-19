use std::sync::Arc;

use dioxus::prelude::*;

use super::TitledView;
use crate::{
    components::wallet::{KeyProviderBadge, OnlineWalletBadge},
    gui::Route,
    hook_helpers,
};
use btc_heritage_wallet::online_wallet::WalletStatus;

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
    log::debug!("WalletList Rendered");

    let wallet_names = hook_helpers::use_resource_wallet_names();

    use_drop(|| log::debug!("WalletList Dropped"));

    rsx! {
        div { class: "max-w-80 md:container mx-auto grid grid-cols-1 md:grid-cols-2 xl:grid-cols-3 2xl:grid-cols-4 gap-12",

            div {

                if let Some(ref wallet_names) = *wallet_names.read() {
                    for wallet_name in wallet_names.into_iter().cloned() {
                        div {
                            key: "{wallet_name}",
                            class: "w-full aspect-square content-center",
                            WalletItem { wallet_name }
                        }
                    }
                }
            }
            for i in 0..30 {
                div { key: "{i}", class: "w-full aspect-square content-center",
                    div { class: "card aspect-square border p-6 mx-auto w-fit", "Pouet {i}" }
                }
            }
        }
    }
}

#[component]
fn WalletItem(wallet_name: Arc<str>) -> Element {
    log::debug!("WalletItem Rendered");

    let navigator = use_navigator();

    let wallet = hook_helpers::use_resource_wallet(wallet_name.clone());
    let wallet_status = hook_helpers::use_resource_wallet_status(wallet);
    let fingerprint = hook_helpers::use_memo_fingerprint(wallet);

    use_drop(|| log::debug!("WalletItem Dropped"));

    rsx! {
        div {
            class: "card card-lg aspect-square border shadow-xl max-h-fit h-full mx-auto",
            onclick: move |_| {
                navigator
                    .push(Route::WalletView {
                        wallet_name: wallet_name.clone().into(),
                    });
            },
            div { class: "card-body",
                div {
                    div { class: "card-title text-3xl font-black", "{wallet_name}" }
                    div { class: "text-sm font-light", "{fingerprint}" }
                }
                div { class: "grow" }

                WalletItemBalance { wallet_status }

                div { class: "grow" }
                div { class: "mx-auto flex flex-row gap-6",
                    KeyProviderBadge { wallet }
                    OnlineWalletBadge { wallet, wallet_status }
                }
            
            }
        }
    }
}

#[component]
fn WalletItemBalance(wallet_status: Resource<Option<WalletStatus>>) -> Element {
    let last_synced = hook_helpers::use_memo_last_sync(wallet_status);
    let balance_strings = hook_helpers::use_memo_balance_strings(wallet_status);

    let is_skeleton = balance_strings.read().is_none();
    let hook_helpers::BalanceStrings {
        balance,
        cur_balance,
        obs_balance,
    } = balance_strings.cloned().unwrap_or_default();

    rsx! {
        div { class: "text-base", "Balance" }
        div {
            div {
                class: "text-nowrap text-3xl font-black",
                class: if is_skeleton { "skeleton text-transparent" },
                {balance}
            }
            div { class: "text-nowrap font-light text-sm",
                "Current: "
                span {
                    class: "font-bold",
                    class: if is_skeleton { "skeleton text-transparent" },
                    {cur_balance}
                }
            }
            div { class: "text-nowrap font-light text-sm",
                "Obsolete: "
                span {
                    class: "font-bold",
                    class: if is_skeleton { "skeleton text-transparent" },
                    {obs_balance}
                }
            }
        }
        div { class: "text-sm font-light text-left",
            "Last Sync: "
            span {
                class: "font-semibold",
                class: if last_synced.read().is_none() { "skeleton text-transparent" },
                {last_synced.cloned().unwrap_or_default().0}
            }
        
        }
    }
}
