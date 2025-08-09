use crate::prelude::*;

use btc_heritage_wallet::{bitcoin::Amount, btc_heritage::utils::timestamp_now};

use std::collections::HashMap;

use crate::{
    components::{badge::UIBadge, balance::UIBtcAmount, heritages::UIHeritage},
    utils::CCStr,
    views::CreateLinkButton,
    Route,
};

#[component]
pub fn HeirWalletListView() -> Element {
    let service_heritages = helper_hooks::use_resource_service_heritages();
    let service_only_heritages = helper_hooks::use_memo_service_only_heritages(service_heritages);

    let display_service_section = use_memo(move || {
        match (
            &*state_management::SERVICE_STATUS.read(),
            &*service_only_heritages.read(),
        ) {
            (Some(ServiceStatus::Connected(_)), Some(Ok(h))) if !h.is_empty() => true,
            (Some(ServiceStatus::Connected(_)), Some(Err(_))) => true,
            _ => false,
        }
    });

    use_context_provider(|| service_heritages);

    rsx! {
        super::TitledView {
            title: CCStr::from("Heir Wallets"),
            subtitle: CCStr::from("Restricted wallets for spending inheritances."),
            HeirWalletList {}
        }
        if display_service_section() {
            hr { class: "my-12 border-none" }
            super::TitledView {
                title: CCStr::from("Service Inheritances"),
                subtitle: CCStr::from("Orphan inheritances found on the Heritage Service."),
                div { class: "flex flex-col gap-6 max-w-7xl mx-auto",
                    LoadedComponent::<HashMap<CCStr,UIHeritage>> { input: service_only_heritages.into() }
                }
            }
        }
    }
}

#[component]
fn HeirWalletList() -> Element {
    log::debug!("HeirWalletList Rendered");

    let heirwallet_names = helper_hooks::use_resource_heirwallet_names();

    let service_user_id = state_management::use_service_key();

    use_drop(|| log::debug!("HeirWalletList Dropped"));

    rsx! {
        div { class: "container mx-auto grid grid-cols-[repeat(auto-fill,var(--container-xs))] gap-6 justify-center",
            if let Some(ref heirwallet_names) = *heirwallet_names.read() {
                for heirwallet_name in heirwallet_names {
                    HeirWalletItem {
                        key: "{heirwallet_name}-{service_user_id()}",
                        heirwallet_name: heirwallet_name.clone(),
                    }
                }
            }
            MaybeHighlight {
                step: OnboardingStep::ClickCreateHeirWalletCard,
                context_callback: || Some((
                    OnboardingContextItemId::KeyProviderCreationRoute
                        .item((Route::HeirWalletCreateView {}).to_string()),
                    true,
                )),
                CreateLinkButton {
                    route: Route::HeirWalletCreateView {},
                    label: CCStr::from("Create Heir Wallet"),
                    size_classes: Some(CCStr::from("w-xs aspect-square")),
                }
            }
        }
    }
}

#[component]
fn HeirWalletItem(heirwallet_name: CCStr) -> Element {
    log::debug!("HeirWalletItem Rendered");

    let heirwallet = helper_hooks::use_async_heirwallet(heirwallet_name.clone());
    let heirwallet_heritages = helper_hooks::use_resource_heirwallet_heritages(heirwallet);

    let spendable = use_memo(move || {
        let now = timestamp_now();
        heirwallet_heritages.lrmap(|heirwallet_heritages| {
            heirwallet_heritages
                .values()
                .flatten()
                .cloned()
                .filter_map(|heritage| {
                    heritage
                        .maturity
                        .is_some_and(|ts| ts <= now)
                        .then(|| heritage.value)
                        .flatten()
                })
                .sum::<Amount>()
        })
    });
    let total = use_memo(move || {
        heirwallet_heritages.lrmap(|heirwallet_heritages| {
            heirwallet_heritages
                .values()
                .flatten()
                .cloned()
                .filter_map(|heritage| heritage.value)
                .sum::<Amount>()
        })
    });

    let keyprovider_status = helper_hooks::use_memo_heirwallet_keyprovider_status(heirwallet);
    let heritageprovider_status = helper_hooks::use_memo_heritage_provider_status(heirwallet);

    let fingerprint = helper_hooks::use_memo_heirwallet_fingerprint(heirwallet);

    let hwn = heirwallet_name.clone();
    let click = move |_| {
        navigator().push(Route::HeirWalletView {
            heirwallet_name: hwn.clone(),
        });
    };

    use_drop(|| log::debug!("HeirWalletItem Dropped"));

    rsx! {
        div {
            class: "card card-lg border shadow-xl w-xs aspect-square cursor-pointer transition-transform hover:scale-105",
            onclick: click,
            div { class: "card-body",
                div { class: "flex flex-col",
                    div { class: "card-title text-3xl font-black", {heirwallet_name} }
                    div { class: "text-sm font-light", {fingerprint()} }
                }
                div { class: "grow" }

                div { class: "text-base", "Known inheritances" }
                div { class: "flex flex-col",
                    div { class: "text-3xl font-black",
                        LoadedComponent::<UIBtcAmount> { input: total.into() }
                    }
                    div { class: "text-nowrap font-light text-sm",
                        "Spendable: "
                        span { class: "font-bold",
                            LoadedComponent::<UIBtcAmount> { input: spendable.into() }
                        }
                    }
                }

                div { class: "grow" }

                div { class: "mx-auto grid grid-cols-2 gap-6",
                    LoadedComponent::<UIBadge> { input: keyprovider_status.into() }
                    LoadedComponent::<UIBadge> { input: heritageprovider_status.into() }
                }
            }
        }
    }
}
