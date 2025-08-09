use crate::prelude::*;

mod account_xpubs;
mod block_inclusion_objective;
mod current_heritage_config;
mod delete;
mod ledger_policies;

use btc_heritage_wallet::Wallet;

use crate::{
    components::{
        badge::{ExternalDependencyStatus, OnlineWalletType},
        inputs::RenameDatabaseItem,
        misc::BackButton,
    },
    utils::CCStr,
    Route,
};

#[component]
pub fn WalletConfigurationView(wallet_name: CCStr) -> Element {
    log::debug!("WalletConfigurationView Rendered");

    let wallet = use_context::<AsyncSignal<Wallet>>();
    let fingerprint = helper_hooks::use_memo_fingerprint(wallet);

    let online_status = use_context::<Memo<Option<(OnlineWalletType, ExternalDependencyStatus)>>>();
    let not_sign_only =
        use_memo(move || !matches!(online_status(), Some((OnlineWalletType::None, _))));

    use_drop(|| log::debug!("WalletConfigurationView Dropped"));

    rsx! {
        super::super::TitledView {
            title: wallet_name.clone(),
            subtitle: fingerprint.cloned(),
            left: rsx! {
                MaybeHighlight {
                    step: OnboardingStep::ClickWalletBackFromConfig,
                    context_filter: consume_onboarding_context(),
                    BackButton {
                        route: Route::WalletView {
                            wallet_name: wallet_name.clone(),
                        },
                    }
                }
            },
            if not_sign_only() {
                current_heritage_config::CurrentHeritageConfig {}
            }
            ledger_policies::LedgerPoliciesConfig { wallet_name: wallet_name.clone() }
            if not_sign_only() {
                block_inclusion_objective::BlockInclusionObjectiveConfig {}
            }
            account_xpubs::AccountXPubConfig {}
            RenameDatabaseItem::<Wallet> {}
            delete::DeleteWalletConfig {}

            OnboardingInfoModal { step: OnboardingStep::ModalExplainHeritageConfiguration,
                div { class: "flex flex-col gap-4 max-w-xl text-base",
                    p {
                        "The Heritage Configuration is the central component that controls how
                        your inheritance works. It determines the order in which your heirs can
                        access funds and the timing requirements for each inheritance level."
                    }
                    p {
                        "Your previously created heirs have been pre-filled in the configuration
                        form. You can now adjust the inheritance timing to match your preferences:"
                    }
                    ul { class: "list-disc list-inside space-y-2 ml-4",
                        li {
                            strong { "First heir delay: " }
                            "How long funds must remain untouched before your first heir
                            (\"Backup\") can inherit them"
                        }
                        li {
                            strong { "Subsequent heirs delay: " }
                            "Use it to space the heirs, 90 days between heirs is a reasonnable default"
                        }
                    }
                    div { class: "bg-warning/10 p-4 rounded-lg border border-warning/20",
                        p { class: "font-semibold mb-2 text-warning", "Important Trade-off:" }
                        p { class: "mb-2",
                            "Shorter delays mean your heirs (and your backup) can access funds more
                            quickly, but you'll need to move your funds more frequently to reset
                            the inheritance countdown while you're still active."
                        }
                        p {
                            "A reasonable default is 1 or 2 years before the first heir
                            can inherit, with 3-month intervals between subsequent heirs.
                            This provides adequate protection while not requiring to move
                            fund too often."
                        }
                    }
                    div { class: "alert alert-info",
                        div { class: "flex items-start gap-2",
                            "💡"
                            div {
                                "You can always create a new Heritage Configuration later and move your
                            bitcoins to adjust these timings as your needs change."
                            }
                        }
                    }
                }
            }

            OnboardingInfoModal { step: OnboardingStep::ModalExplainLedgerPolicies,
                div { class: "flex flex-col gap-4 max-w-2xl text-base",
                    div { class: "flex flex-col gap-4",
                        h3 { class: "text-lg font-semibold", "Ledger Policy Approval Required" }
                        p {
                            "Your Heritage Configuration has been created successfully. However,
                            Ledger devices only accept to sign transactions using "
                            strong { "pre-approved Bitcoin scripts" }
                            " for security reasons."
                        }
                        p {
                            "You now need to approve your new Heritage Configuration on your Ledger device.
                            This will register the Bitcoin scripts associated with your Heritage Configuration,
                            allowing you to sign transactions from this Heritage Wallet."
                        }
                        div { class: "p-4 bg-base-200 rounded-lg border",
                            h4 { class: "font-semibold mb-2", "Next Steps:" }
                            ol { class: "list-decimal list-inside ml-4 space-y-1",
                                li { "Ensure Ledger device is still connected and ready" }
                                li { "Launch the registration process" }
                                li {
                                    "Follow the on-screen prompts to approve the Heritage Wallet policy"
                                }
                                li {
                                    "Verify that the Heritage Wallet app and the Ledger display the same informations"
                                }
                            }
                        }
                        details { class: "collapse collapse-arrow bg-base-200 mt-2",
                            summary { class: "collapse-title font-medium",
                                "Learn more about Ledger wallet policies"
                            }
                            div { class: "collapse-content",
                                div { class: "flex flex-col gap-3 pt-2",
                                    p { class: "text-sm opacity-75",
                                        "Wallet policies allows the user to securely approve
                                        complex Bitcoin scripts once, without compromising
                                        useability by requiring explicit user re-approval
                                        at each transaction."
                                    }
                                    div {
                                        strong { "BIP 388 - Wallet Policies: " }
                                        a {
                                            href: "https://github.com/bitcoin/bips/blob/master/bip-0388.mediawiki",
                                            target: "_blank",
                                            rel: "noopener noreferrer",
                                            class: "text-primary hover:text-primary-focus underline",
                                            "Technical specification for wallet policies"
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}
