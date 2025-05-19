use dioxus::prelude::*;

use btc_heritage_wallet::heritage_service_api_client::HeritageWalletMeta;

use crate::{
    components::{
        badge::{
            ExternalDependencyStatus, KeyProviderType, OnlineWalletType, UIKeyProviderBadge,
            UIOnlineWalletBadge,
        },
        balance::UIBalanceSummary,
        timestamp::LastSyncSpan,
    },
    helper_hooks::{self, use_memo_keyprovider_status, use_memo_online_status},
    loaded::prelude::*,
    utils::{ArcStr, ArcType, EqArcType},
    Route,
};

#[component]
pub fn WalletListView() -> Element {
    rsx! {
        super::TitledView {
            title: "Wallets",
            subtitle: "Heritage wallets with simple Heritage configurations instead of complex Bitcoin scripts.",
            WalletList {}
        }
    }
}

#[component]
fn WalletList() -> Element {
    log::debug!("WalletList Rendered");

    let wallet_names = helper_hooks::use_resource_wallet_names();
    let service_only_wallets = helper_hooks::use_resource_service_only_wallets();

    use_drop(|| log::debug!("WalletList Dropped"));

    rsx! {
        div { class: "max-w-80 md:container mx-auto grid grid-cols-1 md:grid-cols-2 xl:grid-cols-3 2xl:grid-cols-4 gap-12",
            if let Some(ref wallet_names) = *wallet_names.read() {
                for wallet_name in wallet_names.into_iter().cloned() {
                    div {
                        key: "{wallet_name}",
                        class: "w-full aspect-square content-center",
                        WalletItem { wallet_name }
                    }
                }
            }
            if let Some(ref service_only_wallets) = *service_only_wallets.read() {
                for service_only_wallet in service_only_wallets.into_iter().cloned() {
                    div {
                        key: "{service_only_wallet.name}",
                        class: "w-full aspect-square content-center",
                        ServiceOnlyWalletItem { wallet_meta: service_only_wallet.into() }
                    }
                }
            }
        }
    }
}

#[component]
fn WalletItem(wallet_name: ArcStr) -> Element {
    log::debug!("WalletItem Rendered");

    let navigator = use_navigator();

    let wallet = helper_hooks::use_async_wallet(wallet_name.clone());
    let wallet_status = helper_hooks::use_resource_wallet_status(wallet);
    let fingerprint = helper_hooks::use_memo_fingerprint(wallet);
    // let last_synced = helper_hooks::use_memo_last_sync(wallet_status);
    // let balances = helper_hooks::use_memo_display_balances(wallet_status);
    //

    let keyprovider_status = use_memo_keyprovider_status(wallet, wallet_status);
    let online_status = use_memo_online_status(wallet, wallet_status);

    use_drop(|| log::debug!("WalletItem Dropped"));

    rsx! {
        div {
            class: "card card-lg border shadow-xl size-fit mx-auto cursor-pointer transition-transform hover:scale-105",
            onclick: move |_| {
                navigator
                    .push(Route::WalletView {
                        wallet_name: wallet_name.clone(),
                    });
            },
            div { class: "card-body aspect-square h-auto min-w-fit",
                div {
                    div { class: "card-title text-3xl font-black", "{wallet_name}" }
                    div { class: "text-sm font-light", "{fingerprint}" }
                }

                div { class: "grow" }

                LoadedComponent::<UIBalanceSummary> { input: wallet_status.into() }
                div { class: "text-sm font-light text-left",
                    "Last Sync: "
                    span { class: "font-semibold",
                        LoadedComponent::<LastSyncSpan> { input: wallet_status.into() }
                    }
                }

                div { class: "grow" }

                div { class: "mx-auto grid grid-cols-2 gap-6",
                    LoadedComponent::<UIKeyProviderBadge> { input: keyprovider_status.into() }
                    LoadedComponent::<UIOnlineWalletBadge> { input: online_status.into() }
                }
            
            }
        }
    }
}

#[component]
fn ServiceOnlyWalletItem(wallet_meta: EqArcType<HeritageWalletMeta>) -> Element {
    log::debug!("ServiceOnlyWalletItem Rendered");

    let wallet_meta: ArcType<HeritageWalletMeta> = wallet_meta.into();
    let wallet_name = wallet_meta.name.as_str();
    let fingerprint = wallet_meta
        .fingerprint
        .map(|f| f.to_string())
        .unwrap_or_else(|| "-".to_owned());

    use_drop(|| log::debug!("ServiceOnlyWalletItem Dropped"));

    rsx! {
        div { class: "relative",
            div { class: "card card-lg border shadow-xl size-fit mx-auto opacity-40",
                div { class: "card-body aspect-square h-auto min-w-fit",
                    div {
                        div { class: "card-title text-3xl font-black", "{wallet_name}" }
                        div { class: "text-sm font-light", "{fingerprint}" }
                    }
                    div { class: "grow" }

                    LoadedComponent::<UIBalanceSummary> { input: wallet_meta.ref_into() }
                    div { class: "text-sm font-light text-left",
                        "Last Sync: "
                        span { class: "font-semibold",
                            LoadedComponent::<LastSyncSpan> { input: wallet_meta.ref_into() }
                        }
                    }

                    div { class: "grow" }

                    div { class: "mx-auto grid grid-cols-2 gap-6",
                        LoadedComponent::<UIKeyProviderBadge> { input: (KeyProviderType::None, ExternalDependencyStatus::Available).ref_into() }
                        LoadedComponent::<UIOnlineWalletBadge> { input: (OnlineWalletType::Service, ExternalDependencyStatus::Available).ref_into() }
                    }
                }
            }
            div { class: "text-3xl text-secondary font-black absolute top-0 left-0 h-full w-full text-center content-center -rotate-45",
                "Only on Service"
            }
        }
    }
}
