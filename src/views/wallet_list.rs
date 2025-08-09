use crate::prelude::*;

use btc_heritage_wallet::heritage_service_api_client::HeritageWalletMeta;

use crate::{
    components::{
        badge::{ExternalDependencyStatus, KeyProviderType, OnlineWalletType, UIBadge},
        balance::UIBalanceSummary,
        timestamp::LastSyncSpan,
    },
    utils::{CCStr, CheapClone, EqCheapClone},
    views::CreateLinkButton,
    Route,
};

#[component]
pub fn WalletListView() -> Element {
    rsx! {
        super::TitledView {
            title: CCStr::from("Wallets"),
            subtitle: CCStr::from(
                "Heritage wallets with simple Heritage configurations instead of complex Bitcoin scripts.",
            ),
            WalletList {}
        }
    }
}

#[component]
fn WalletList() -> Element {
    log::debug!("WalletList Rendered");

    let wallet_names = helper_hooks::use_resource_wallet_names();
    let service_only_wallets = helper_hooks::use_resource_service_only_wallets();

    let service_user_id = state_management::use_service_key();

    use_drop(|| log::debug!("WalletList Dropped"));

    rsx! {
        div { class: "container mx-auto grid grid-cols-[repeat(auto-fill,var(--container-xs))] gap-6 justify-center",
            if let Some(ref wallet_names) = *wallet_names.read() {
                for wallet_name in wallet_names {
                    MaybeHighlight {
                        step: OnboardingStep::ClickWalletCardAfterHeirsCreation,
                        context_filter: OnboardingContextItemId::WalletName.item(wallet_name.to_string()),
                        WalletItem {
                            key: "{wallet_name}-{service_user_id()}",
                            wallet_name: wallet_name.clone(),
                        }
                    }
                }
            }
            if let Some(ref service_only_wallets) = *service_only_wallets.read() {
                for service_only_wallet in service_only_wallets {
                    ServiceOnlyWalletItem {
                        key: "{service_only_wallet.name}-{service_user_id()}",
                        wallet_meta: service_only_wallet.clone().into(),
                    }
                }
            }

            MaybeHighlight {
                step: OnboardingStep::ClickCreateWalletCard,
                context_callback: || Some((
                    OnboardingContextItemId::KeyProviderCreationRoute
                        .item((Route::WalletCreateView {}).to_string()),
                    true,
                )),
                CreateLinkButton {
                    route: Route::WalletCreateView {},
                    label: CCStr::from("Create Wallet"),
                    size_classes: Some(CCStr::from("w-xs aspect-square")),
                }
            }
        }
    }
}

#[component]
fn WalletItem(wallet_name: CCStr) -> Element {
    log::debug!("WalletItem Rendered");

    let wallet = helper_hooks::use_async_wallet(wallet_name.clone());
    let wallet_status = helper_hooks::use_resource_wallet_status(wallet);
    let fingerprint = helper_hooks::use_memo_fingerprint(wallet);

    let wn = wallet_name.clone();
    let click = move |_| {
        navigator().push(Route::WalletView {
            wallet_name: wn.clone(),
        });
    };
    let keyprovider_status = helper_hooks::use_memo_wallet_keyprovider_status(wallet, None);
    let online_status = helper_hooks::use_memo_wallet_online_status(wallet);

    let not_sign_only =
        use_memo(move || !matches!(online_status(), Some((OnlineWalletType::None, _))));

    use_drop(|| log::debug!("WalletItem Dropped"));

    rsx! {
        div {
            class: "card card-lg border shadow-xl w-xs aspect-square cursor-pointer transition-transform hover:scale-105",
            onclick: click,
            div { class: "card-body",
                div {
                    div { class: "card-title text-3xl font-black text-nowrap overflow-auto",
                        {wallet_name}
                    }
                    div { class: "text-sm font-light", {fingerprint()} }
                }

                div { class: "grow" }

                if not_sign_only() {
                    LoadedComponent::<UIBalanceSummary> { input: wallet_status.into() }
                    div { class: "text-sm font-light text-left",
                        "Last Sync: "
                        span { class: "font-semibold",
                            LoadedComponent::<LastSyncSpan> { input: wallet_status.into() }
                        }
                    }

                    div { class: "grow" }
                }

                div { class: "mx-auto grid grid-cols-2 gap-6",
                    LoadedComponent::<UIBadge> { input: keyprovider_status.into() }
                    LoadedComponent::<UIBadge> { input: online_status.into() }
                }
            
            }
        }
    }
}

#[component]
fn ServiceOnlyWalletItem(wallet_meta: EqCheapClone<HeritageWalletMeta>) -> Element {
    log::debug!("ServiceOnlyWalletItem Rendered");

    let wallet_meta: CheapClone<HeritageWalletMeta> = wallet_meta.into();
    let wallet_name = wallet_meta.name.as_str();
    let fingerprint = wallet_meta
        .fingerprint
        .map(|f| CCStr::from(f.to_string()))
        .unwrap_or_else(|| CCStr::from("-"));

    use_drop(|| log::debug!("ServiceOnlyWalletItem Dropped"));

    rsx! {
        div { class: "relative",
            div { class: "card card-lg border shadow-xl w-xs aspect-square opacity-40",
                div { class: "card-body",
                    div {
                        div { class: "card-title text-3xl font-black text-nowrap overflow-clip",
                            {wallet_name}
                        }
                        div { class: "text-sm font-light", {fingerprint} }
                    }
                    div { class: "grow" }

                    LoadedComponent::<UIBalanceSummary> { input: wallet_meta.as_ref().ref_into() }
                    div { class: "text-sm font-light text-left",
                        "Last Sync: "
                        span { class: "font-semibold",
                            LoadedComponent::<LastSyncSpan> { input: wallet_meta.as_ref().ref_into() }
                        }
                    }

                    div { class: "grow" }

                    div { class: "mx-auto grid grid-cols-2 gap-6",
                        LoadedComponent::<UIBadge> { input: (KeyProviderType::None, ExternalDependencyStatus::None).ref_into() }
                        LoadedComponent::<UIBadge> { input: (OnlineWalletType::Service, ExternalDependencyStatus::None).ref_into() }
                    }
                }
            }
            div { class: "text-3xl text-secondary font-black absolute top-0 left-0 h-full w-full text-center content-center -rotate-45",
                "Only on Service"
            }
        }
    }
}
