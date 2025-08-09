use std::collections::HashMap;

use crate::prelude::*;

pub mod configuration;
mod heritages_list;
pub mod spend;
mod sync;

use btc_heritage_wallet::{btc_heritage::utils::timestamp_now, HeirWallet};

use crate::{
    components::{
        badge::{ExternalDependencyStatus, HeritageProviderType, KeyProviderType, UIBadge},
        quick_actions::{ShowKeyProviderMnemonic, ShowKeyProviderMnemonicFlavor, UnlockLocalKey},
        svg::{Cog, DrawSvg, SvgSize::Full},
    },
    utils::CCStr,
    Route,
};

#[component]
pub fn HeirWalletWrapperLayout(heirwallet_name: CCStr) -> Element {
    log::debug!("HeirWalletWrapperLayout Rendered");

    let heirwallet = helper_hooks::use_async_heirwallet(heirwallet_name.clone());

    let keyprovider_status = helper_hooks::use_memo_heirwallet_keyprovider_status(heirwallet);
    let online_status = helper_hooks::use_memo_heritage_provider_status(heirwallet);

    let heirwallet_local_lastsync =
        helper_hooks::use_resource_heirwallet_local_lastsync(heirwallet);

    let service_heritages = helper_hooks::use_resource_service_heritages();
    let heirwallet_heritages = helper_hooks::use_resource_heirwallet_heritages(heirwallet);

    let heirwallet_contextualized_heritages =
        helper_hooks::use_memo_heirwallet_contextualized_heritages(
            heirwallet,
            heirwallet_heritages,
            service_heritages,
        );

    // Provide the heir wallet resources to all child that may want it
    use_context_provider(|| heirwallet);

    use_context_provider(|| keyprovider_status);
    use_context_provider(|| online_status);

    use_context_provider(|| heirwallet_local_lastsync);

    use_context_provider(|| service_heritages);
    use_context_provider(|| heirwallet_heritages);

    use_context_provider(|| heirwallet_contextualized_heritages);

    use_context_provider(|| {
        OnboardingContextItemId::HeirWalletName.item(heirwallet_name.to_string())
    });

    use_drop(|| log::debug!("HeirWalletWrapperLayout Dropped"));
    rsx! {
        Outlet::<crate::Route> {}
    }
}

#[component]
pub fn HeirWalletView(heirwallet_name: CCStr) -> Element {
    log::debug!("HeirWalletView Rendered");

    let heirwallet = use_context::<AsyncSignal<HeirWallet>>();
    let fingerprint = helper_hooks::use_memo_heirwallet_fingerprint(heirwallet);
    let keyprovider_status =
        use_context::<Memo<Option<(KeyProviderType, ExternalDependencyStatus)>>>();

    let local_key_need_password = use_memo(move || match keyprovider_status() {
        Some((KeyProviderType::LocalKey, ExternalDependencyStatus::NeedUserAction)) => true,
        _ => false,
    });
    let can_show_mnemo = use_memo(move || match keyprovider_status() {
        Some((KeyProviderType::LocalKey, _)) => true,
        _ => false,
    });

    let online_status =
        use_context::<Memo<Option<(HeritageProviderType, ExternalDependencyStatus)>>>();

    let hwn = heirwallet_name.clone();
    let click_config = move |_| {
        navigator().push(Route::HeirWalletConfigurationView {
            heirwallet_name: hwn.clone(),
        });
    };
    let heirwallet_contextualized_heritages =
        use_context::<FMemo<HashMap<CCStr, ContextualizedHeritages>>>();

    let heritage_to_display_count = use_memo(move || {
        heirwallet_contextualized_heritages
            .lrmap_ok(|h| h.len())
            .unwrap_or_default()
    });
    let any_heritage_to_spend = use_memo(move || {
        let now = timestamp_now();
        heirwallet_contextualized_heritages
            .lrmap_ok(|h| {
                h.values().any(|ch| {
                    ch.heritages
                        .iter()
                        .any(|h| h.maturity.is_some_and(|ts| ts < now))
                })
            })
            .unwrap_or_default()
    });

    use_effect(move || {
        let status_read_guard = state_management::ONBOARDING_STATUS.read();
        if let OnboardingStatus::InProgress(ref onboarding) = *status_read_guard {
            if let Some(OnboardingStep::ClickInheritanceSpendButton) = onboarding.current_step() {
                let can_spend = any_heritage_to_spend();
                let onboarding_paused = onboarding.is_paused();
                let need_toggle_pause =
                    can_spend && onboarding_paused || !can_spend && !onboarding_paused;
                if need_toggle_pause {
                    drop(status_read_guard);
                    let OnboardingStatus::InProgress(ref mut onboarding) =
                        *state_management::ONBOARDING_STATUS.write()
                    else {
                        unreachable!()
                    };
                    if onboarding_paused {
                        onboarding.resume()
                    } else {
                        onboarding.pause()
                    }
                }
            }
        }
    });

    use_drop(|| log::debug!("HeirWalletView Dropped"));

    rsx! {
        super::TitledView {
            title: heirwallet_name.clone(),
            subtitle: fingerprint.cloned(),
            left: rsx! {
                sync::HeirWalletLocalSync {}
            },
            right: rsx! {
                div { class: "h-full content-center",
                    button {
                        class: "btn btn-circle btn-outline btn-primary btn-lg p-2",
                        onclick: click_config,
                        DrawSvg::<Cog> { size: Full }
                    }
                }
            },

            div { class: "flex flex-row justify-center gap-8 m-4",
                div { class: "flex flex-col gap-1",
                    div { class: "flex flex-row flex-wrap justify-center gap-1",
                        span { "Key Provider: " }
                        LoadedComponent::<UIBadge> { input: keyprovider_status.into() }
                    }

                    if local_key_need_password() {
                        UnlockLocalKey::<HeirWallet> {}
                    }
                    if can_show_mnemo() {
                        ShowKeyProviderMnemonic::<HeirWallet> { flavor: ShowKeyProviderMnemonicFlavor::Wallet }
                    }
                
                }
                div { class: "flex flex-col gap-1",
                    div { class: "flex flex-row flex-wrap justify-center gap-1",
                        span { "Online Wallet: " }
                        LoadedComponent::<UIBadge> { input: online_status.into() }
                    }
                }
            }

            heritages_list::HeritagesList {}

            OnboardingInfoModal { step: OnboardingStep::ModalExplainInheritancesList,
                div { class: "flex flex-col gap-4 max-w-xl text-base",
                    p {
                        "This page displays all the inheritances you are eligible to receive.
                        Each inheritance may have details including the maturity date (when you
                        can claim it), the amount of bitcoin, or who configured it for you,
                        depending on the original owner's configuration."
                    }
                    p {
                        "The Heritage Wallet allows people to set up time-locked inheritances
                        that automatically become available to designated heirs after a specified
                        period. When an inheritance reaches its maturity date, you'll be able to
                        create and sign transactions to claim the bitcoins from here."
                    }
                    if heritage_to_display_count() == 0 {
                        p {
                            "You don't currently have any inheritances visible. This could mean
                            no inheritances have been configured for you yet, or, depending on the
                            original owner's privacy settings, you may not be able to see
                            inheritances until they become eligible for claiming. Some owners
                            choose to keep inheritance details private until the maturity
                            conditions are met."
                        }
                    }
                    if !any_heritage_to_spend() {
                        p {
                            "Since you don't currently have any mature inheritances, the onboarding
                            process will pause. Come back here when your inheritance becomes
                            mature to resume it."
                        }
                    }
                }
            }
        }
    }
}
