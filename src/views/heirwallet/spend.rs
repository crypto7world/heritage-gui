use crate::prelude::*;

use std::collections::HashMap;

use btc_heritage_wallet::{
    bitcoin::Amount, btc_heritage::utils::timestamp_now, HeirWallet, Heritage,
};

use crate::{
    components::{
        badge::{ExternalDependencyStatus, HeritageProviderType, KeyProviderType},
        misc::BackButton,
        spend::{SpendTabs, SpendTabsType},
        svg::{AlertOutline, DrawSvg, InfoCircle},
    },
    utils::{CCStr, CheapClone},
    Route,
};

#[component]
pub fn HeirWalletSpendView(heirwallet_name: CCStr, heritage_id: CCStr) -> Element {
    log::debug!("HeirWalletSpendView Rendered");

    let heirwallet = use_context::<AsyncSignal<HeirWallet>>();
    let fingerprint = helper_hooks::use_memo_heirwallet_fingerprint(heirwallet);

    let keyprovider_status =
        use_context::<Memo<Option<(KeyProviderType, ExternalDependencyStatus)>>>();
    let heritage_provider_status =
        use_context::<Memo<Option<(HeritageProviderType, ExternalDependencyStatus)>>>();

    let heirwallet_heritages =
        use_context::<FResource<HashMap<CCStr, Vec<CheapClone<Heritage>>>>>();

    let max_spendable_amount = {
        let heritage_id = heritage_id.clone();
        use_memo(move || {
            let now = timestamp_now();
            heirwallet_heritages.lrmap(|heirwallet_heritages| {
                heirwallet_heritages
                    .get(&heritage_id)
                    .expect("heritage_id should always exist in the heritage table")
                    .iter()
                    .filter_map(|h| {
                        h.maturity
                            .is_some_and(|m| m < now)
                            .then(|| h.value.expect("value always present if maturity present"))
                    })
                    .sum::<Amount>()
            })
        })
    };

    use_context_provider(|| max_spendable_amount);
    use_context_provider(|| OnboardingContextItemId::HeritageId.item(heritage_id.to_string()));

    let addresses_set = use_signal(|| None);

    let cannot_create_reason = use_memo(move || match heritage_provider_status() {
        Some((HeritageProviderType::None, _)) => {
            Some("Your heir wallet does not have an Heritage Provider component.")
        }
        Some((HeritageProviderType::Service, ExternalDependencyStatus::Unavailable)) => {
            Some("The Heritage Service cannot currently serve your heir wallet.")
        }
        None => Some("Heir Wallet is not loaded."),
        _ => None,
    });

    let cannot_sign_reason = use_memo(move || match keyprovider_status() {
        Some((KeyProviderType::None, _)) => Some((
            "Your heir wallet does not have a Key Provider component.",
            false,
        )),
        Some((KeyProviderType::LocalKey, ExternalDependencyStatus::NeedUserAction)) => Some((
            "Your heir wallet uses a Local Key that require a password, \
                but the password was not provided.",
            true,
        )),
        Some((KeyProviderType::Ledger, ExternalDependencyStatus::Unavailable)) => Some((
            "Your heir wallet uses a Ledger Hardware Wallet device, \
               but none can currently serve your heir wallet.",
            false,
        )),
        Some((KeyProviderType::Ledger, ExternalDependencyStatus::NeedUserAction)) => Some((
            "Your heir wallet uses a Ledger Hardware Wallet device, \
               but it is missing Ledger Policies to be able to sign transactions.",
            false,
        )),
        None => Some(("Heir Wallet is not loaded.", false)),
        _ => None,
    });

    let cannot_broadcast_reason = use_memo(move || match heritage_provider_status() {
        Some((HeritageProviderType::None, _)) => {
            Some("Your heir wallet does not have an Heritage Provider component.")
        }
        Some((HeritageProviderType::Service, ExternalDependencyStatus::Unavailable)) => {
            Some("The Heritage Service cannot currently serve your heir wallet.")
        }
        Some((HeritageProviderType::LocalWallet, ExternalDependencyStatus::Unavailable)) => {
            Some("Your heir wallet use a Blockchain provider but none is accessible.")
        }
        None => Some("Heir Wallet is not loaded."),
        _ => None,
    });

    use_drop(|| log::debug!("HeirWalletSpendView Dropped"));

    rsx! {
        super::super::TitledView {
            title: heirwallet_name,
            subtitle: fingerprint.cloned(),
            left: rsx! {
                BackButton { route: Route::HeirWalletListView {} }
            },
            SpendTabs::<HeirWallet> {
                spendtabs_type: SpendTabsType::Heir(heritage_id),
                cannot_create_reason,
                cannot_sign_reason,
                cannot_broadcast_reason,
                addresses_set,
            }
            OnboardingInfoModal { step: OnboardingStep::ModalExplainInheritanceSpend,
                div { class: "flex flex-col gap-4 max-w-xl text-base",
                    p {
                        "You are about to claim and transfer your Bitcoin inheritance. This is
                        an important step that requires careful attention to where you send
                        these funds. "
                        span { class: "font-bold",
                            "The receiving address you enter "
                            span { class: "font-black text-warning", "MUST" }
                            " be under your complete control"
                        }
                        " - never send Bitcoin to an address provided by someone else claiming
                        to \"help\" you, as this is a common scam targeting inheritance recipients."
                    }
                    p {
                        "If you're new to Bitcoin, we recommend creating an account with a
                        reputable exchange service like "
                        a {
                            href: "https://www.kraken.com",
                            target: "_blank",
                            class: "link link-primary",
                            "Kraken"
                        }
                        ", "
                        a {
                            href: "https://www.coinbase.com",
                            target: "_blank",
                            class: "link link-primary",
                            "Coinbase"
                        }
                        ", or "
                        a {
                            href: "https://www.binance.com",
                            target: "_blank",
                            class: "link link-primary",
                            "Binance"
                        }
                        ". Once your account is verified, these services will provide you with a
                        Bitcoin receiving address that you can safely use. More advanced users can
                        alternatively use a receiving address from their own Bitcoin wallet,
                        including an Heritage Wallet if desired."
                    }
                }
            }
            OnboardingInfoModal { step: OnboardingStep::ModalInheritanceVerifyTransaction,
                div { class: "flex flex-col gap-4 max-w-xl text-base",
                    div { class: "alert alert-warning text-base font-semibold",
                        DrawSvg::<AlertOutline> {}
                        p {
                            "You are about to broadcast your transaction to the Bitcoin network.
                            This action "
                            span { class: "bg-error font-black text-lg uppercase", "cannot be undone" }
                            "."
                        }
                    }
                    p {
                        "Before clicking the broadcast button, please "
                        span { class: "text-warning font-bold uppercase",
                            "carefully review the recipient address"
                        }
                        " a last time and "
                        span { class: "text-warning font-bold uppercase",
                            "ensure it is an address you own"
                        }
                        "!"
                    }
                    p { class: "font-medium text-warning",
                        "Triple-check it, as once broadcast, this transaction "
                        span { class: "font-black text-lg uppercase", "cannot" }
                        " be canceled or modified."
                    }
                    div { class: "alert alert-info",
                        DrawSvg::<InfoCircle> {}
                        div { class: "text-sm",
                            "After broadcasting, your transaction will be submitted to the Bitcoin network and should appear in block explorers within minutes."
                        }
                    }
                }
            }
            OnboardingInfoModal { step: OnboardingStep::ModalFinishClaimingFirstInheritance,
                div { class: "flex flex-col gap-4 max-w-xl text-base",
                    p { class: "text-lg font-semibold text-success", "🎉 Congratulations!" }
                    p {
                        "Your Bitcoin inheritance transaction has been broadcast to the network and is now being processed. "
                        "The funds should appear in your destination wallet or exchange account within approximately "
                        span { class: "font-semibold", "1 hour" }
                        ", though it may take longer during periods of high network activity."
                    }
                    p {
                        "You can track the progress of your transaction using the transaction ID provided. "
                        "If you sent the funds to an exchange like Kraken, Coinbase, or Binance, "
                        "you'll receive a notification once the transaction has been confirmed and the Bitcoin is available in your account."
                    }
                    p { class: "text-sm opacity-75",
                        "Thank you for using the Heritage Wallet to claim your inheritance. "
                        "Your Bitcoin is now safely in your control."
                    }
                }
            }
        }
    }
}
