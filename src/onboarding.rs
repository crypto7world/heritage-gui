use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::{
    utils::{log_error, CCStr},
    Route,
};

pub type Exclusive = bool;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct OnboardingContext(HashMap<OnboardingContextItemId, Vec<String>>);
impl OnboardingContext {
    pub fn new() -> Self {
        Self(HashMap::new())
    }
    pub fn add_context(&mut self, context: OnboardingContextItem, exclusive: Exclusive) {
        let OnboardingContextItem(context_key, content) = context;
        if exclusive {
            self.0.get_mut(&context_key).map(|vec| vec.clear());
        }
        self.0
            .entry(context_key)
            .or_insert_with(Vec::new)
            .push(content);
    }
    pub fn get_first_context(&self, context_key: OnboardingContextItemId) -> Option<&str> {
        self.get(&context_key)
            .map(|hs| hs.iter().next())
            .flatten()
            .map(|s| s.as_str())
    }
}
impl core::ops::Deref for OnboardingContext {
    type Target = HashMap<OnboardingContextItemId, Vec<String>>;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
struct PrivateMarker;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum OnboardingContextItemId {
    WalletName,
    HeirName,
    HeirWalletName,
    HeritageId,
    DefaultRoute,
    KeyProviderCreationRoute,
    /// Private marker ensure the MatchNothing variant cannot be used directly
    #[allow(private_interfaces)]
    #[serde(skip)]
    MatchNothing(PrivateMarker),
}
impl OnboardingContextItemId {
    pub fn item(self, content: String) -> OnboardingContextItem {
        OnboardingContextItem(self, content)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct OnboardingContextItem(OnboardingContextItemId, String);

pub fn consume_onboarding_context() -> OnboardingContextItem {
    dioxus::prelude::try_consume_context().unwrap_or_else(|| {
        OnboardingContextItem(
            OnboardingContextItemId::MatchNothing(PrivateMarker),
            String::new(),
        )
    })
}

#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq, Serialize, Deserialize)]
pub enum OnboardingStep {
    // Before we start modals
    ModalCreateAccountOnTheService,
    ModalInstallBlockchainProviderNode,
    // Config
    ClickConnectService,
    ConfigureBlockchainProvider,
    // Wallet
    ClickCreateWalletCard,
    ModalExplainWalletSplit,
    EnsureLedgerIsConnected,
    SelectLocalOnlineWallet,
    ClickCreateWalletButton,
    ClickWalletCardAfterHeirsCreation,
    OpenWalletConfiguration,
    ClickCreateHeritageConfigurationButton1,
    ModalExplainHeritageConfiguration,
    ClickCreateHeritageConfigurationButton2,
    ModalExplainLedgerPolicies,
    ClickRegisterLedgerPolicies,
    ClickWalletBackFromConfig,
    ModalExplainHeritageBackup,
    ClickBackupDescriptors,
    ClickSaveBackup,
    ClickWalletReceive,
    ModalFinishCreatingFirstWallet,
    // Heir
    ModalExplainHeirs,
    ClickCreateHeirCard,
    InputBackupHeirName,
    ModalExplainHeirKeyProvider,
    ClickExportHeirToService,
    InputEmailAddress,
    ModalExplainExportHeirToServiceOptions,
    ClickCreateHeirButton,
    ClickHeirShowMnemonic,
    ModalExplainStoreHeirMnemonic,
    CheckHeirRevealMnemonic,
    HoverHeirMnemonic,
    CloseHeirShowMnemonic,
    CheckConfirmStripHeirSeed,
    StripHeirSeed,
    ModalMoreHeirOrWallet,
    ClickHeirCard,
    // HeirWallet
    ModalLocalInheritance,
    ClickCreateHeirWalletCard,
    SelectLocalHeritageProvider,
    ProvideLocalWalletBackup,
    ClickCreateHeirWalletButton,
    ClickHeirWalletCard,
    SynchronizeLocalHeritage,
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
    // Creation commons
    InputName,
    SelectLocalKeyStorage,
    InputTheSeedPassword,
    SelectRestoreSeed,
    RestoreKeyProviderSeed,
}
impl OnboardingStep {
    fn message(self, context: &OnboardingContext) -> Option<CCStr> {
        use OnboardingContextItemId::*;
        use OnboardingStep::*;
        match self {
            // Before we start modals
            ModalCreateAccountOnTheService => None,
            ModalInstallBlockchainProviderNode => None,
            // Config
            ClickConnectService => Some(CCStr::from(
                "Click on \"Status\" -> \"Connect\" to connect this application with the Heritage Service",
            )),
            ConfigureBlockchainProvider => {
                Some(CCStr::from("Click on \"Status\" -> \"Open Configuration\" and configure the blockchain provider settings"))
            }
            // Wallet
            ClickCreateWalletCard => Some(CCStr::from("Click on the \"Create Wallet\" card")),
            ModalExplainWalletSplit => None,
            EnsureLedgerIsConnected => Some(CCStr::from(
                "Connect your Ledger device and ensure it's ready",
            )),
            SelectLocalOnlineWallet => {
                Some(CCStr::from("Select the \"Local Node\" Online Wallet Type"))
            }
            ClickCreateWalletButton => {
                Some(CCStr::from("Click the \"Create Wallet\" button to finish"))
            }
            ClickWalletCardAfterHeirsCreation => {
                let wallet_name = context
                    .get_first_context(WalletName)
                    .expect("should not be there without WalletName context");
                Some(CCStr::from(format!(
                    "Once you have created all the heirs you need, \
                    click on the \"{wallet_name}\" wallet card"
                )))
            }
            OpenWalletConfiguration => {
                Some(CCStr::from("Go to the wallet Configuration view"))
            }
            ClickCreateHeritageConfigurationButton1 => {
                Some(CCStr::from("Click \"Create\" in the Heritage Configuration section"))
            }
            ModalExplainHeritageConfiguration => None,
            ClickCreateHeritageConfigurationButton2 => {
                Some(CCStr::from("Once you are satisfied with your configuration, click \"Create\" button"))
            }
            ModalExplainLedgerPolicies => None,
            ClickRegisterLedgerPolicies => {
                Some(CCStr::from("Ensure your Ledger is still connected and ready, \
                    then click the \"Register Ledger Policies\" button"))
            }
            ClickWalletBackFromConfig => {
                Some(CCStr::from("Go back to the wallet main view"))
            }
            ModalExplainHeritageBackup => None,
            ClickBackupDescriptors =>  Some(CCStr::from("Click the \"Backup Descriptors\" button")),
            ClickSaveBackup =>  Some(CCStr::from("Click the \"Save Backup\" button \
                after changing the save location if needed")),
            ClickWalletReceive => {
                Some(CCStr::from("Click the \"Receive\" button to generate your first Bitcoin address"))
            }
            ModalFinishCreatingFirstWallet => None,
            // Heir
            ModalExplainHeirs => Some(CCStr::from("Go to the \"Heirs\" list view")),
            ClickCreateHeirCard => Some(CCStr::from("Click on the \"Create Heir\" card")),
            InputBackupHeirName => Some(CCStr::from("Name this first heir \"Backup\"")),
            ModalExplainHeirKeyProvider => None,
            ClickExportHeirToService => {
                Some(CCStr::from("Toggle the \"Export to Service\" option"))
            }
            InputEmailAddress => Some(CCStr::from(
                "Enter your own email address for this \"Backup\" heir",
            )),
            ModalExplainExportHeirToServiceOptions => None,
            ClickCreateHeirButton => {
                Some(CCStr::from("Click the \"Create Heir\" button to finish"))
            }
            ClickHeirShowMnemonic => Some(CCStr::from("Click the \"Show Mnemonic\" red button")),
            ModalExplainStoreHeirMnemonic => None,
            CheckHeirRevealMnemonic => Some(CCStr::from(
                "Reveal the mnemonic and write it somewhere safe",
            )),
            HoverHeirMnemonic => Some(CCStr::from(
                "Copy the mnemonic (hover it for at least 1 second to validate this step)",
            )),
            CloseHeirShowMnemonic => Some(CCStr::from(
                "Now that you have copied the mnemonic, close this window",
            )),
            CheckConfirmStripHeirSeed => Some(CCStr::from(
                "Now that you have copied the mnemonic, confirm that you have backed up the mnemonic",
            )),
            StripHeirSeed => Some(CCStr::from(
                "Strip it from the app so it only exist offline",
            )),
            ModalMoreHeirOrWallet => None,
            ClickHeirCard => {
                let heir_name = context
                    .get_first_context(HeirName)
                    .expect("should not be there without HeirName context");
                Some(CCStr::from(format!("Click on the \"{heir_name}\" card")))
            }
            // HeirWallet
            ModalLocalInheritance => None,
            ClickCreateHeirWalletCard => {
                Some(CCStr::from("Click on the \"Create Heir Wallet\" card"))
            }
            SelectLocalHeritageProvider => {
                Some(CCStr::from("Select the \"Local\" Heritage Provider"))
            }
            ProvideLocalWalletBackup => Some(CCStr::from(
                "Provide the Online Wallet Descriptors backup of the inheritance's source Heritage Wallet",
            )),
            ClickCreateHeirWalletButton => {
                Some(CCStr::from("Click the \"Create Heir Wallet\" button"))
            }
            ClickHeirWalletCard => {
                let heirwallet_name = context
                    .get_first_context(HeirWalletName)
                    .expect("should not be there without HeirWalletName context");
                Some(CCStr::from(format!(
                    "Click on the \"{heirwallet_name}\" card"
                )))
            }
            SynchronizeLocalHeritage => Some(CCStr::from("Synchronize your local Heir Wallet")),
            ModalExplainInheritancesList => None,
            ClickInheritanceSpendButton => {
                Some(CCStr::from("Click the \"Spend Inheritance\" button"))
            }
            ModalExplainInheritanceSpend => None,
            InputInheritanceSpendAddress => Some(CCStr::from(
                "Enter the Bitcoin address to send the inheritance to",
            )),
            ClickInheritanceCreateTransaction => Some(CCStr::from(
                "Click \"Create Transaction\" to prepare the inheritance transaction",
            )),
            ClickInheritanceSignTransaction => Some(CCStr::from(
                "Click \"Sign Transaction\" to sign the transaction",
            )),
            ModalInheritanceVerifyTransaction => None,
            HoverTransactionRecipientAddress => Some(CCStr::from(
                "Verify the recipient of the transaction is the address you \
                inputted and that you own that address (hover it for 2 \
                seconds to validate the step)",
            )),
            ClickInheritanceBroadcastTransaction => Some(CCStr::from(
                "Click \"Broadcast Transaction\" to send it to the Bitcoin network",
            )),
            ModalFinishClaimingFirstInheritance => None,
            // Creation commons
            InputName => Some(CCStr::from("Enter a name")),
            SelectLocalKeyStorage => Some(CCStr::from(
                "Select the \"Local Key Storage\" Key Provider Type",
            )),
            InputTheSeedPassword => Some(CCStr::from("Choose a Password to protect your seed \
                (mandatory in onboarding mode)")),
            SelectRestoreSeed => Some(CCStr::from(
                "Select the \"Restore\" Local Key Provider Creation Option",
            )),
            RestoreKeyProviderSeed => Some(CCStr::from(
                "Enter the mnemonic seed phrase to restore the Key Provider. If you where provided \
                with a 8-characters fingerprint, verify it matches the one displayed after \
                you have entered your mnemonic phrase",
            )),
        }
    }

    /// When reloading an in_progress onboarding, can tell on which
    /// view the user is supposed to be for this step
    fn associated_route(self, context: &OnboardingContext) -> Option<Route> {
        use OnboardingContextItemId::*;
        use OnboardingStep::*;
        match self {
            ModalCreateAccountOnTheService | ModalInstallBlockchainProviderNode => None,
            ClickConnectService => None,
            ConfigureBlockchainProvider => Some(Route::AppConfigView {}),
            ClickCreateWalletCard => Some(Route::WalletListView {}),
            ModalExplainWalletSplit
            | EnsureLedgerIsConnected
            | SelectLocalOnlineWallet
            | ClickCreateWalletButton => Some(Route::WalletCreateView {}),
            ClickWalletCardAfterHeirsCreation => Some(Route::WalletListView {}),
            OpenWalletConfiguration
            | ModalExplainHeritageBackup
            | ClickBackupDescriptors
            | ClickSaveBackup
            | ClickWalletReceive
            | ModalFinishCreatingFirstWallet => {
                let wallet_name = CCStr::from(
                    context
                        .get_first_context(WalletName)
                        .expect("should not be there without WalletName context"),
                );
                Some(Route::WalletView { wallet_name })
            }
            ClickCreateHeritageConfigurationButton1
            | ModalExplainHeritageConfiguration
            | ClickCreateHeritageConfigurationButton2
            | ModalExplainLedgerPolicies
            | ClickRegisterLedgerPolicies
            | ClickWalletBackFromConfig => {
                let wallet_name = CCStr::from(
                    context
                        .get_first_context(WalletName)
                        .expect("should not be there without WalletName context"),
                );
                Some(Route::WalletConfigurationView { wallet_name })
            }
            ClickCreateHeirCard => Some(Route::HeirListView {}),
            ModalExplainHeirs
            | InputBackupHeirName
            | ModalExplainHeirKeyProvider
            | ClickExportHeirToService
            | InputEmailAddress
            | ModalExplainExportHeirToServiceOptions
            | ClickCreateHeirButton => Some(Route::HeirCreateView {}),
            ModalMoreHeirOrWallet
            | ClickHeirCard
            | ClickHeirShowMnemonic
            | ModalExplainStoreHeirMnemonic
            | CheckHeirRevealMnemonic
            | HoverHeirMnemonic
            | CloseHeirShowMnemonic
            | CheckConfirmStripHeirSeed
            | StripHeirSeed => Some(Route::HeirListView {}),
            ModalLocalInheritance => None,
            ClickCreateHeirWalletCard => Some(Route::HeirWalletListView {}),
            SelectLocalHeritageProvider
            | ProvideLocalWalletBackup
            | ClickCreateHeirWalletButton => Some(Route::HeirWalletCreateView {}),
            ClickHeirWalletCard => Some(Route::HeirWalletListView {}),
            SynchronizeLocalHeritage
            | ModalExplainInheritancesList
            | ClickInheritanceSpendButton => {
                let heirwallet_name = CCStr::from(
                    context
                        .get_first_context(HeirWalletName)
                        .expect("should not be there without HeirWalletName context"),
                );
                Some(Route::HeirWalletView { heirwallet_name })
            }
            ModalExplainInheritanceSpend
            | InputInheritanceSpendAddress
            | ClickInheritanceCreateTransaction
            | ClickInheritanceSignTransaction
            | ModalInheritanceVerifyTransaction
            | HoverTransactionRecipientAddress
            | ClickInheritanceBroadcastTransaction
            | ModalFinishClaimingFirstInheritance => {
                let heirwallet_name = CCStr::from(
                    context
                        .get_first_context(HeirWalletName)
                        .expect("should not be there without HeirWalletName context"),
                );
                let heritage_id = CCStr::from(
                    context
                        .get_first_context(HeritageId)
                        .expect("should not be there without HeritageId context"),
                );
                Some(Route::HeirWalletSpendView {
                    heirwallet_name,
                    heritage_id,
                })
            }
            InputName
            | SelectLocalKeyStorage
            | InputTheSeedPassword
            | SelectRestoreSeed
            | RestoreKeyProviderSeed => context
                .get_first_context(OnboardingContextItemId::KeyProviderCreationRoute)
                .expect("KeyProviderCreationRoute is always present")
                .parse()
                .map_err(log_error)
                .ok(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Onboarding {
    steps: Vec<OnboardingStep>,
    current_step: usize,
    is_paused: bool,
    context: OnboardingContext,
}

impl Onboarding {
    /// Creates a new [OnboardingBuilder]
    ///
    /// # Examples
    ///
    /// ```
    /// let onboarding = Onboarding::builder()
    ///     .add_step(OnboardingStep::OpenStatusMenu)
    ///     .add_step(OnboardingStep::ClickConnectService)
    ///     .build();
    /// ```
    pub fn builder() -> OnboardingBuilder {
        OnboardingBuilder::new()
    }

    /// Used to verify if the onboarding is currently at a given step.
    /// Return true if the current OnboardingStep has the OnboardingStepId
    /// If a context is provided, also verify it is in the context hashset and return `true` only if it is the case
    pub fn is_active(&self, id: OnboardingStep, context: Option<OnboardingContextItem>) -> bool {
        if !self.finished() && !self.is_paused {
            id == self.steps[self.current_step]
                && context.is_none_or(|ctx| {
                    self.context
                        .get(&ctx.0)
                        .is_some_and(|hs| hs.contains(&ctx.1))
                })
        } else {
            false
        }
    }

    pub fn finished(&self) -> bool {
        self.steps.len() == self.current_step
    }

    /// Advances the onboarding to the next step if the provided step matches the current or next skippable step.
    ///
    /// This method searches forward from the current step, skipping over any steps marked as skippable,
    /// until it finds a non-skippable step or the provided step. If the provided step matches what was
    /// found, the onboarding advances to the step after that match.
    pub fn progress(&mut self, step: OnboardingStep) {
        if self.current_step().is_some_and(|cs| cs == step) {
            // Move to the next step
            self.current_step += 1;
        }
    }

    pub fn pause(&mut self) {
        self.is_paused = true;
    }

    pub fn resume(&mut self) {
        self.is_paused = false;
    }

    pub fn is_paused(&self) -> bool {
        self.is_paused
    }

    pub fn current_step(&self) -> Option<OnboardingStep> {
        (!self.finished()).then(|| self.steps[self.current_step])
    }
    pub fn current_message(&self) -> Option<CCStr> {
        if !self.is_paused {
            self.current_step()
                .map(|step| step.message(&self.context))
                .flatten()
        } else {
            None
        }
    }
    pub fn current_route(&self) -> Route {
        let opt_route = self
            .current_step()
            .map(|step| step.associated_route(&self.context))
            .flatten();

        opt_route.unwrap_or_else(|| {
            self.context
                .get_first_context(OnboardingContextItemId::DefaultRoute)
                .expect("DefaultRoute is always present")
                .parse()
                .map_err(log_error)
                .unwrap_or(Route::WalletListView {})
        })
    }

    pub fn context(&self) -> &OnboardingContext {
        &self.context
    }

    /// exclusive: Make sure to remove any other context sharing the same enum variant before adding this one
    pub fn add_context(&mut self, context: OnboardingContextItem, exclusive: Exclusive) {
        self.context.add_context(context, exclusive);
    }
}

/// Builder for creating [Onboarding] instances
#[derive(Debug, Clone)]
pub struct OnboardingBuilder {
    steps: Vec<OnboardingStep>,
}

impl OnboardingBuilder {
    /// Creates a new [OnboardingBuilder]
    ///
    /// # Examples
    ///
    /// ```
    /// let builder = OnboardingBuilder::new();
    /// ```
    pub fn new() -> Self {
        Self { steps: Vec::new() }
    }

    /// Adds a single [OnboardingStep] to the builder
    ///
    /// # Examples
    ///
    /// ```
    /// let onboarding = OnboardingBuilder::new()
    ///     .add_step(OnboardingStep::OpenStatusMenu)
    ///     .add_step(OnboardingStep::ClickConnectService)
    ///     .build();
    /// ```
    pub fn add_step(mut self, step: OnboardingStep) -> Self {
        self.steps.push(step);
        self
    }

    /// Adds multiple [OnboardingStep]s from a slice to the builder
    ///
    /// # Examples
    ///
    /// ```
    /// let steps = &[
    ///     OnboardingStep::OpenStatusMenu,
    ///     OnboardingStep::ClickConnectService,
    ///     OnboardingStep::GoToConfiguration,
    /// ];
    /// let onboarding = OnboardingBuilder::new()
    ///     .add_steps(steps)
    ///     .build();
    /// ```
    pub fn add_steps(mut self, steps: &[OnboardingStep]) -> Self {
        self.steps.extend_from_slice(steps);
        self
    }

    /// Builds the [Onboarding] instance
    ///
    /// The built onboarding will start with:
    /// - current_step = 0
    /// - Empty context
    ///
    /// # Examples
    ///
    /// ```
    /// let onboarding = OnboardingBuilder::new()
    ///     .add_step(OnboardingStep::OpenStatusMenu)
    ///     .build();
    ///
    /// assert_eq!(onboarding.current_step(), Some(OnboardingStep::OpenStatusMenu));
    /// ```
    pub fn build(self, default_route: Route) -> Onboarding {
        let mut ob = Onboarding {
            steps: self.steps,
            current_step: 0,
            is_paused: false,
            context: OnboardingContext::new(),
        };
        ob.add_context(
            OnboardingContextItemId::DefaultRoute.item(default_route.to_string()),
            true,
        );
        ob
    }
}

impl Default for OnboardingBuilder {
    fn default() -> Self {
        Self::new()
    }
}
