use std::sync::Arc;

use btc_heritage_wallet::{
    btc_heritage::bitcoincore_rpc::jsonrpc::serde_json, online_wallet::WalletStatus, Wallet,
};
use dioxus::prelude::*;

use crate::{hook_helpers, utils::ArcStr, views::TitledView};

#[component]
pub fn WalletView(wallet_name: ArcStr) -> Element {
    log::debug!("WalletView Rendered");

    let wallet_name: Arc<str> = wallet_name.into();

    let wallet = hook_helpers::use_resource_wallet(wallet_name.clone());
    let wallet_status = hook_helpers::use_resource_wallet_status(wallet);
    let fingerprint = hook_helpers::use_memo_fingerprint(wallet);
    // Provide the wallet to all child that want it
    use_context_provider(|| wallet);
    use_context_provider(|| wallet_status);

    let last_synced = hook_helpers::use_memo_last_sync(wallet_status);

    use_drop(|| log::debug!("WalletView Dropped"));

    rsx! {
        TitledView {
            title: "{wallet_name}",
            subtitle: "{fingerprint}",
            left: rsx! {
                div { class: "h-full flex gap-2 items-center",
                    button { class: "btn btn-circle btn-outline btn-primary btn-lg p-2",
                        svg {
                            class: "h-full w-full fill-current",
                            xmlns: "http://www.w3.org/2000/svg",
                            view_box: "0 0 24 24",
                            path { d: "M12,18A6,6 0 0,1 6,12C6,11 6.25,10.03 6.7,9.2L5.24,7.74C4.46,8.97 4,10.43 4,12A8,8 0 0,0 12,20V23L16,19L12,15M12,4V1L8,5L12,9V6A6,6 0 0,1 18,12C18,13 17.75,13.97 17.3,14.8L18.76,16.26C19.54,15.03 20,13.57 20,12A8,8 0 0,0 12,4Z" }
                        }
                    }
                    div { class: "h-fit",
                        div { class: "text-base font-light", "Last synced:" }
                        div {
                            class: "text-base font-bold",
                            class: if last_synced.read().is_none() { "skeleton text-transparent" },
                            {last_synced.cloned().unwrap_or_default().0}
                        }
                    }
                }
            },
            right: rsx! {
                div { class: "h-full content-center",
                    button { class: "btn btn-circle btn-outline btn-primary btn-lg p-2",
                        svg {
                            class: "min-h-full h-full w-full fill-current",
                            xmlns: "http://www.w3.org/2000/svg",
                            view_box: "0 0 24 24",
                            path {
                                d: "M12,8A4,4 0 0,1 16,12A4,4 0 0,1 12,16A4,4 0 0,1 8,12A4,4 0 0,1 12,8M12,10A2,2 0 0,0 10,12A2,2 0 0,0 12,14A2,2 0 0,0 14,12A2,2 0 0,0 12,10M10,22C9.75,22 9.54,21.82 9.5,21.58L9.13,18.93C8.5,18.68 7.96,18.34 7.44,17.94L4.95,18.95C4.73,19.03 4.46,18.95 4.34,18.73L2.34,15.27C2.21,15.05 2.27,14.78 2.46,14.63L4.57,12.97L4.5,12L4.57,11L2.46,9.37C2.27,9.22 2.21,8.95 2.34,8.73L4.34,5.27C4.46,5.05 4.73,4.96 4.95,5.05L7.44,6.05C7.96,5.66 8.5,5.32 9.13,5.07L9.5,2.42C9.54,2.18 9.75,2 10,2H14C14.25,2 14.46,2.18 14.5,2.42L14.87,5.07C15.5,5.32 16.04,5.66 16.56,6.05L19.05,5.05C19.27,4.96 19.54,5.05 19.66,5.27L21.66,8.73C21.79,8.95 21.73,9.22 21.54,9.37L19.43,11L19.5,12L19.43,13L21.54,14.63C21.73,14.78 21.79,15.05 21.66,15.27L19.66,18.73C19.54,18.95 19.27,19.04 19.05,18.95L16.56,17.95C16.04,18.34 15.5,18.68 14.87,18.93L14.5,21.58C14.46,21.82 14.25,22 14,22H10M11.25,4L10.88,6.61C9.68,6.86 8.62,7.5 7.85,8.39L5.44,7.35L4.69,8.65L6.8,10.2C6.4,11.37 6.4,12.64 6.8,13.8L4.68,15.36L5.43,16.66L7.86,15.62C8.63,16.5 9.68,17.14 10.87,17.38L11.24,20H12.76L13.13,17.39C14.32,17.14 15.37,16.5 16.14,15.62L18.57,16.66L19.32,15.36L17.2,13.81C17.6,12.64 17.6,11.37 17.2,10.2L19.31,8.65L18.56,7.35L16.15,8.39C15.38,7.5 14.32,6.86 13.12,6.62L12.75,4H11.25Z",
                            }
                        }
                    }
                }
            },
            div { class: "flex flex-row justify-center gap-4",
                Balance {}
                button { class: "btn btn-secondary size-64 uppercase text-3xl font-black",
                    "Send"
                }
                button { class: "btn btn-secondary size-64 uppercase text-3xl font-black",
                    "Receive"
                }
            }
            UtxoList {}
            TransactionsHistory {}
            HeritageConfigurationsHistory {}
        }
    }
}

#[component]
fn Balance() -> Element {
    log::debug!("Balance Rendered");
    let wallet_status = use_context::<Resource<Option<WalletStatus>>>();

    let balance_strings = hook_helpers::use_memo_balance_strings(wallet_status);

    let is_skeleton = balance_strings.read().is_none();
    let hook_helpers::BalanceStrings {
        balance,
        cur_balance,
        obs_balance,
    } = balance_strings.cloned().unwrap_or_default();
    use_drop(|| log::debug!("Balance Dropped"));
    rsx! {
        div { class: "card card-xl h-64 bg-base-100 shadow-sm p-6",
            div { class: "card-title", "Balance" }
            div { class: "card-body",
                div {
                    class: "text-nowrap text-4xl font-black",
                    class: if is_skeleton { "skeleton text-transparent" },
                    {balance}
                }
                div { class: "font-light text-sm",
                    "Current Heritage Config: "
                    span {
                        class: "text-nowrap font-bold",
                        class: if is_skeleton { "skeleton text-transparent" },
                        {cur_balance}
                    }
                }
                div { class: "font-light text-sm",
                    "Previous Heritage Config: "
                    span {
                        class: "text-nowrap font-bold",
                        class: if is_skeleton { "skeleton text-transparent" },
                        {obs_balance}
                    }
                }
            }
        }
    }
}

#[component]
fn TransactionsHistory() -> Element {
    log::debug!("TransactionsHistory Rendered");
    let wallet = use_context::<Resource<Wallet>>();
    let wallet_transactions = hook_helpers::use_resource_wallet_transactions(wallet);
    use_drop(|| log::debug!("TransactionsHistory Dropped"));
    rsx! {
        div { class: "py-6",
            if let Some(ref wallet_transactions) = *wallet_transactions.read() {
                ul {
                    for wallet_transaction in wallet_transactions {
                        li { key: "{wallet_transaction.txid}", class: "p-2",
                            "{serde_json::to_string(wallet_transaction).unwrap()}"
                        }
                    }
                }
            }
        }
    }
}

#[component]
fn UtxoList() -> Element {
    log::debug!("UtxoList Rendered");
    let wallet = use_context::<Resource<Wallet>>();
    let wallet_utxos = hook_helpers::use_resource_wallet_utxos(wallet);
    use_drop(|| log::debug!("UtxoList Dropped"));
    rsx! {
        div { class: "py-6",
            if let Some(ref wallet_utxos) = *wallet_utxos.read() {
                ul {
                    for wallet_utxo in wallet_utxos {
                        li { key: "{wallet_utxo.outpoint}", class: "p-2",
                            "{serde_json::to_string(wallet_utxo).unwrap()}"
                        }
                    }
                }
            }
        }
    }
}

#[component]
fn HeritageConfigurationsHistory() -> Element {
    log::debug!("HeritageConfigurationsHistory Rendered");
    let wallet = use_context::<Resource<Wallet>>();
    let wallet_heritage_configs = hook_helpers::use_resource_wallet_heritage_configs(wallet);
    use_drop(|| log::debug!("HeritageConfigurationsHistory Dropped"));
    rsx! {
        div { class: "py-6",
            if let Some(ref wallet_heritage_configs) = *wallet_heritage_configs.read() {
                ul {
                    for (idx , wallet_heritage_config) in wallet_heritage_configs.into_iter().enumerate() {
                        li { key: "{idx}", class: "p-2",
                            "{serde_json::to_string(wallet_heritage_config).unwrap()}"
                        }
                    }
                }
            }
        }
    }
}
