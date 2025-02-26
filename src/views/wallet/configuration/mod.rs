use dioxus::prelude::*;

use btc_heritage_wallet::Wallet;

use crate::{helper_hooks, utils::RcStr, Route};

#[component]
pub fn WalletConfigurationView(wallet_name: RcStr) -> Element {
    log::debug!("WalletConfigurationView Rendered");

    let navigator = use_navigator();

    let wallet = use_context::<Resource<Wallet>>();

    let fingerprint = helper_hooks::use_memo_fingerprint(wallet);

    use_drop(|| log::debug!("WalletConfigurationView Dropped"));

    rsx! {
        super::super::TitledView {
            title: "{wallet_name}",
            subtitle: "{fingerprint}",
            left: rsx! {
                div { class: "h-full content-center",
                    button {
                        class: "btn btn-outline btn-primary btn-lg",
                        onclick: move |_| {
                            navigator
                                .push(Route::WalletView {
                                    wallet_name: wallet_name.clone(),
                                });
                        },
                        svg {
                            class: "h-full fill-current",
                            xmlns: "http://www.w3.org/2000/svg",
                            view_box: "0 0 24 24",
                            path { d: "M20,9V15H12V19.84L4.16,12L12,4.16V9H20Z" }
                        }
                        "Back"
                    }
                }
            },
            div { "Now I need content...." }
        }
    }
}
