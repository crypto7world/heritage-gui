use dioxus::prelude::*;

use btc_heritage_wallet::online_wallet::WalletStatus;

use crate::{components::wallet::BtcAmount, helper_hooks};

#[component]
pub(super) fn Balance() -> Element {
    log::debug!("Balance Rendered");
    let wallet_status = use_context::<Resource<Option<WalletStatus>>>();

    let balances = helper_hooks::use_memo_display_balances(wallet_status);

    let helper_hooks::Balances {
        balance,
        cur_balance,
        obs_balance,
    } = balances.cloned();
    use_drop(|| log::debug!("Balance Dropped"));
    rsx! {
        div { class: "card card-xl h-64 bg-base-100 shadow-sm p-6",
            div { class: "card-title", "Balance" }
            div { class: "card-body",
                div { class: "text-4xl font-black",
                    BtcAmount { amount: balance }
                }
                div { class: "font-light text-sm",
                    "Current Heritage Config: "
                    span { class: "font-bold",
                        BtcAmount { amount: cur_balance }
                    }
                }
                div { class: "font-light text-sm",
                    "Previous Heritage Config: "
                    span { class: "font-bold",
                        BtcAmount { amount: obs_balance }
                    }
                }
            }
        }
    }
}
