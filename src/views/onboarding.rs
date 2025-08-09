use crate::prelude::*;

use crate::{
    components::svg::{BankPlus, Cog, DrawSvg, DrawableSvg, Seed, SvgSize::Custom},
    onboarding::Onboarding,
};

/// Onboarding answers for determining the best setup path
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct OnboardingAnswers {
    pub what: Option<WhatAnswer>,
    pub how_public: Option<HowPublicAnswer>,
    pub how_private: Option<HowPrivateAnswer>,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum WhatAnswer {
    CreateWallet,
    Inherit,
    NoOnboardingNeeded,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum HowPublicAnswer {
    HeritageService,
    OwnNode,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum HowPrivateAnswer {
    LedgerDevice,
    LocalStorage,
    RestoreSeed,
}

impl OnboardingAnswers {
    pub fn new() -> Self {
        Self {
            what: None,
            how_public: None,
            how_private: None,
        }
    }
}

fn heir_onboarding(how_public: HowPublicAnswer) -> Onboarding {
    use crate::onboarding::OnboardingStep::*;

    let obb = Onboarding::builder().add_steps(match how_public {
        HowPublicAnswer::HeritageService => &[ModalCreateAccountOnTheService, ClickConnectService],
        HowPublicAnswer::OwnNode => &[
            ModalLocalInheritance,
            ModalInstallBlockchainProviderNode,
            ConfigureBlockchainProvider,
        ],
    });

    // This is common to all
    let obb = obb.add_steps(&[ClickCreateHeirWalletCard, InputName, RestoreKeyProviderSeed]);

    let obb = match how_public {
        HowPublicAnswer::HeritageService => obb,
        // If local heritage, extra steps are required
        HowPublicAnswer::OwnNode => {
            obb.add_steps(&[SelectLocalHeritageProvider, ProvideLocalWalletBackup])
        }
    };

    // This is common to all
    let obb = obb.add_step(ClickCreateHeirWalletButton);

    let obb = match how_public {
        HowPublicAnswer::HeritageService => obb,
        // If local heritage, ask to sync
        HowPublicAnswer::OwnNode => obb.add_step(SynchronizeLocalHeritage),
    };

    // This is common to all
    let obb = obb.add_steps(&[
        ModalExplainInheritancesList,
        ClickInheritanceSpendButton,
        ModalExplainInheritanceSpend,
        InputInheritanceSpendAddress,
        ClickInheritanceCreateTransaction,
        ClickInheritanceSignTransaction,
        ModalInheritanceVerifyTransaction,
        HoverTransactionRecipientAddress,
        ClickInheritanceBroadcastTransaction,
        ModalFinishClaimingFirstInheritance,
    ]);

    obb.build(crate::Route::HeirWalletListView {})
}

fn owner_onboarding(how_public: HowPublicAnswer, how_private: HowPrivateAnswer) -> Onboarding {
    use crate::onboarding::OnboardingStep::*;

    let obb = Onboarding::builder().add_steps(match how_public {
        HowPublicAnswer::HeritageService => &[ModalCreateAccountOnTheService, ClickConnectService],
        HowPublicAnswer::OwnNode => &[
            ModalInstallBlockchainProviderNode,
            ConfigureBlockchainProvider,
        ],
    });

    // This is common to all
    let obb = obb.add_steps(&[ClickCreateWalletCard, ModalExplainWalletSplit, InputName]);

    let obb = match how_private {
        HowPrivateAnswer::LedgerDevice => obb.add_step(EnsureLedgerIsConnected),
        HowPrivateAnswer::LocalStorage => {
            obb.add_steps(&[SelectLocalKeyStorage, InputTheSeedPassword])
        }
        HowPrivateAnswer::RestoreSeed => obb.add_steps(&[
            SelectLocalKeyStorage,
            SelectRestoreSeed,
            RestoreKeyProviderSeed,
        ]),
    };

    let obb = match how_public {
        HowPublicAnswer::HeritageService => obb,
        // If local heritage, extra steps are required
        HowPublicAnswer::OwnNode => obb.add_step(SelectLocalOnlineWallet),
    };

    // This is common to all
    let obb = obb.add_steps(&[
        ClickCreateWalletButton,
        ModalExplainHeirs,
        ClickCreateHeirCard,
        InputBackupHeirName,
        ModalExplainHeirKeyProvider,
    ]);

    let obb = match how_public {
        // If Heritage Service, extra steps are required
        HowPublicAnswer::HeritageService => obb.add_steps(&[
            ClickExportHeirToService,
            InputEmailAddress,
            ModalExplainExportHeirToServiceOptions,
        ]),
        HowPublicAnswer::OwnNode => obb,
    };

    // This is common to all
    let obb = obb.add_steps(&[
        ClickCreateHeirButton,
        ClickHeirCard,
        ClickHeirShowMnemonic,
        ModalExplainStoreHeirMnemonic,
        CheckHeirRevealMnemonic,
        HoverHeirMnemonic,
        CloseHeirShowMnemonic,
        CheckConfirmStripHeirSeed,
        StripHeirSeed,
        ModalMoreHeirOrWallet,
        ClickWalletCardAfterHeirsCreation,
        OpenWalletConfiguration,
        ClickCreateHeritageConfigurationButton1,
        ModalExplainHeritageConfiguration,
        ClickCreateHeritageConfigurationButton2,
    ]);

    let obb = match how_private {
        HowPrivateAnswer::LedgerDevice => {
            obb.add_steps(&[ModalExplainLedgerPolicies, ClickRegisterLedgerPolicies])
        }
        HowPrivateAnswer::LocalStorage | HowPrivateAnswer::RestoreSeed => obb,
    };

    // This is common to all
    let obb = obb.add_step(ClickWalletBackFromConfig);

    let obb = match how_public {
        HowPublicAnswer::HeritageService => obb,
        // If local heritage, extra steps are required
        HowPublicAnswer::OwnNode => obb.add_steps(&[
            ModalExplainHeritageBackup,
            ClickBackupDescriptors,
            ClickSaveBackup,
        ]),
    };

    // This is common to all
    let obb = obb.add_steps(&[ClickWalletReceive, ModalFinishCreatingFirstWallet]);

    obb.build(crate::Route::WalletListView {})
}

fn start_onboarding(answers: OnboardingAnswers) {
    let OnboardingAnswers {
        what: Some(what),
        how_public: Some(how_public),
        how_private,
    } = answers
    else {
        panic!("start_onboarding called with incomplete OnboardingAnswers");
    };

    let onboarding = match what {
        WhatAnswer::CreateWallet => {
            let how_private =
                how_private.expect("start_onboarding called with incomplete OnboardingAnswers");
            owner_onboarding(how_public, how_private)
        }
        WhatAnswer::Inherit => heir_onboarding(how_public),
        WhatAnswer::NoOnboardingNeeded => {
            panic!("start_onboarding called with incomplete OnboardingAnswers")
        }
    };

    let next_route = onboarding.current_route();
    *state_management::ONBOARDING_STATUS.write() = OnboardingStatus::InProgress(onboarding);
    use_navigator().push(next_route);
}

#[component]
pub fn OnboardingLayout() -> Element {
    log::debug!("OnboardingLayout Rendered");

    // Central state for onboarding answers
    let onboarding_answers = use_signal(OnboardingAnswers::new);
    // Provide answers Signal to child components
    use_context_provider(|| onboarding_answers);

    use_drop(|| log::debug!("OnboardingLayout Dropped"));

    rsx! {
        div { class: "min-h-screen bg-base-200 py-8",
            h1 { class: "text-2xl font-black text-center text-primary/50 mb-8", "Onboarding" }
            Outlet::<crate::Route> {}
        }
    }
}

#[component]
pub fn OnboardingWhoView() -> Element {
    log::debug!("OnboardingWhoView Rendered");

    let mut onboarding_answers = use_context::<Signal<OnboardingAnswers>>();

    let mut handle_answer = move |answer: WhatAnswer| {
        onboarding_answers.write().what = Some(answer);

        match answer {
            WhatAnswer::NoOnboardingNeeded => {
                // Skip the rest of onboarding and go to main app
                *state_management::ONBOARDING_STATUS.write() = OnboardingStatus::Completed;
                use_navigator().push(crate::Route::WalletListView {});
            }
            _ => {
                // Continue to next question
                use_navigator().push(crate::Route::OnboardingHowPublicView {});
            }
        }
    };

    use_drop(|| log::debug!("OnboardingWhoView Dropped"));

    rsx! {
        OnboardingQuestionView { question: "What do you want to do?",
            OnboardingCard::<BankPlus> {
                title: "Setup an Heritage Wallet",
                subtitle: "I own bitoins and I want to set up a Heritage Wallet so I never lose them",
                onclick: move |_| handle_answer(WhatAnswer::CreateWallet),
            }
            OnboardingCard::<Seed> {
                title: "Inherit bitcoins",
                subtitle: "I'm heir of an Heritage Wallet user and I want to be able to retrieve inherited funds",
                onclick: move |_| handle_answer(WhatAnswer::Inherit),
            }
            OnboardingCard::<Cog> {
                title: "Explore by myself",
                subtitle: "I know what to do, I don't need an onboarding process",
                onclick: move |_| handle_answer(WhatAnswer::NoOnboardingNeeded),
            }
        }
    }
}

#[component]
pub fn OnboardingHowPublicView() -> Element {
    log::debug!("OnboardingHowPublicView Rendered");

    let mut onboarding_answers = use_context::<Signal<OnboardingAnswers>>();

    let mut handle_answer = move |answer: HowPublicAnswer| {
        onboarding_answers.write().how_public = Some(answer);
        match onboarding_answers().what {
            Some(WhatAnswer::CreateWallet) => {
                // Go on to next view
                use_navigator().push(crate::Route::OnboardingHowPrivateView {});
            }
            Some(WhatAnswer::Inherit) => {
                // Stop there and start inheritance onboarding
                log::debug!(
                    "Onboarding completed with answers: {:?}",
                    onboarding_answers()
                );
                start_onboarding(onboarding_answers());
            }
            _ => unreachable!("cannot be here in those cases"),
        };
    };

    let question = match onboarding_answers().what {
        Some(WhatAnswer::CreateWallet) => "How will you access the Bitcoin blockchain?",
        Some(WhatAnswer::Inherit) => "How is/was the inheritance managed by the original owner?",
        _ => unreachable!("cannot be here in those cases"),
    };

    let service_subtitle = match onboarding_answers().what {
        Some(WhatAnswer::CreateWallet) => {
            "Best option - managed infrastructure and features, easier and more reliable for heirs"
        }
        Some(WhatAnswer::Inherit) => "They told me / I received an email from the service",
        _ => unreachable!("cannot be here in those cases"),
    };

    let node_title = match onboarding_answers().what {
        Some(WhatAnswer::CreateWallet) => "Using my own node",
        Some(WhatAnswer::Inherit) => "Using their own node",
        _ => unreachable!("cannot be here in those cases"),
    };
    let node_subtitle = match onboarding_answers().what {
        Some(WhatAnswer::CreateWallet) => {
            "I will connect to a Bitcoin Core or Electrum node"
        }
        Some(WhatAnswer::Inherit) => "I have a backup of the original wallet and I will configure a Bitcoin Core or Electrum node",
        _ => unreachable!("cannot be here in those cases"),
    };

    use_drop(|| log::debug!("OnboardingHowPublicView Dropped"));

    rsx! {
        OnboardingQuestionView { question,
            OnboardingCard::<BankPlus> {
                title: "Using the Heritage Service",
                subtitle: service_subtitle,
                onclick: move |_| handle_answer(HowPublicAnswer::HeritageService),
            }
            OnboardingCard::<Cog> {
                title: node_title,
                subtitle: node_subtitle,
                onclick: move |_| handle_answer(HowPublicAnswer::OwnNode),
            }
        }
    }
}

#[component]
pub fn OnboardingHowPrivateView() -> Element {
    log::debug!("OnboardingHowPrivateView Rendered");

    let mut onboarding_answers = use_context::<Signal<OnboardingAnswers>>();

    let mut handle_answer = move |answer: HowPrivateAnswer| {
        onboarding_answers.write().how_private = Some(answer);

        // Onboarding complete, determine next step based on answers
        log::debug!(
            "Onboarding completed with answers: {:?}",
            onboarding_answers()
        );
        start_onboarding(onboarding_answers());
    };

    use_drop(|| log::debug!("OnboardingHowPrivateView Dropped"));

    rsx! {
        OnboardingQuestionView { question: "How will you manage your private keys?",
            OnboardingCard::<Cog> {
                title: "With a Ledger Hardware Device",
                subtitle: "Best security - hardware wallet protection",
                onclick: move |_| handle_answer(HowPrivateAnswer::LedgerDevice),
            }
            OnboardingCard::<Seed> {
                title: "Local storage with password",
                subtitle: "Software wallet with password protection",
                onclick: move |_| handle_answer(HowPrivateAnswer::LocalStorage),
            }
            OnboardingCard::<BankPlus> {
                title: "Restore an existing wallet",
                subtitle: "Import an existing seed mnemonic words",
                onclick: move |_| handle_answer(HowPrivateAnswer::RestoreSeed),
            }
        }
    }
}

/// Common component for onboarding question layout
#[component]
fn OnboardingQuestionView(question: &'static str, children: Element) -> Element {
    rsx! {
        // Question header
        div { class: "text-center mb-12",
            h1 { class: "text-5xl font-bold text-base-content mb-4", "{question}" }
        }

        // Answer cards grid
        div { class: "grid grid-cols-[repeat(auto-fit,var(--container-sm))] gap-8 place-content-center",
            {children}
        }
    }
}

#[doc = "Properties for the [`OnboardingCard`] component."]
#[allow(missing_docs)]
#[derive(Props, Clone, PartialEq)]
#[allow(non_camel_case_types)]
struct OnboardingCardProps {
    title: &'static str,
    subtitle: &'static str,
    onclick: EventHandler<MouseEvent>,
}
#[doc = " Individual answer card component"]
#[doc = "# Props\n*For details, see the [props struct definition](OnboardingCardProps).*"]
#[doc = "- [`title`](OnboardingCardProps::title) : `&'static str`"]
#[doc = "- [`subtitle`](OnboardingCardProps::subtitle) : `&'static str`"]
#[doc = "- [`onclick`](OnboardingCardProps::onclick) : `EventHandler<MouseEvent>`"]
#[allow(non_snake_case)]
fn OnboardingCard<S: DrawableSvg>(
    OnboardingCardProps {
        title,
        subtitle,
        onclick,
    }: OnboardingCardProps,
) -> Element {
    rsx! {
        div {
            class: "card bg-base-100 shadow-xl transition-shadow cursor-pointer border-2 border-transparent hover:border-primary hover:shadow-2xl",
            onclick: move |evt| onclick.call(evt),

            div { class: "p-4 grid grid-rows-3 gap-4 lg:gap-8 place-items-center text-center lg:h-[50vh] min-h-60 max-h-[500px]",
                // Icon
                div { class: "text-primary",
                    DrawSvg::<S> { size: Custom("size-20") }
                }

                // Title
                h2 { class: "card-title text-4xl font-black justify-center", "{title}" }

                // Subtitle
                p { class: "text-lg text-base-content/70", "{subtitle}" }
            }
        }
    }
}
