use crate::prelude::*;

use std::collections::HashSet;

use btc_heritage_wallet::{
    btc_heritage::{
        errors::ParseBlockInclusionObjectiveError, utils::bitcoin_network, BlockInclusionObjective,
        HeritageWalletBackup,
    },
    heritage_service_api_client::HeritageWalletMeta,
    online_wallet::{LocalHeritageWallet, ServiceBinding as WalletServiceBinding},
    AnyKeyProvider, AnyOnlineWallet, BoundFingerprint, KeyProvider, LedgerKey, LocalKey,
    OnlineWallet, Wallet,
};

use crate::{
    components::{
        create_key_provider::{
            KeyProviderConfig, KeyProviderConfigState, KeyProviderSection,
            KeyProviderSectionFlavor, LocalKeyCreationConfig,
        },
        inputs::{
            use_future_error_feedback, BackupRestoreSection, InputField, RadioChoice, RadioChoices,
        },
        misc::{BackButton, Divider},
        modal::InfoModal,
        svg::{DrawSvg, WalletPlus},
    },
    utils::{CCStr, CheapClone, FutureFingerprints},
    Route,
};

/// Component for creating new wallets
#[component]
pub fn WalletCreateView() -> Element {
    log::debug!("WalletCreateView Rendered");

    use_drop(|| log::debug!("WalletCreateView Dropped"));

    rsx! {
        super::TitledView {
            title: CCStr::from("Create Wallet"),
            subtitle: CCStr::from("Create a new Heritage wallet to securely hold your bitcoins."),
            left: rsx! {
                BackButton { route: Route::WalletListView {} }
            },
            WalletCreateForm {}

            OnboardingInfoModal { step: OnboardingStep::ModalExplainWalletSplit,
                div { class: "flex flex-col gap-4 max-w-xl text-base",
                    p {
                        "Heritage Wallets use a split architecture that separates your wallet into
                        two distinct components by design for enhanced security and flexibility:"
                    }

                    div { class: "flex flex-col gap-3",
                        div { class: "flex gap-3",
                            div { class: "badge badge-primary badge-outline font-bold min-w-fit",
                                "1"
                            }
                            div {
                                strong { "Key Provider (Offline): " }
                                "Manages your private keys and handles transaction signing.
                                This can be a hardware wallet like Ledger for maximum security,
                                or a software wallet for convenience. The key provider never
                                needs internet access and can be used on an air-gapped computer."
                            }
                        }
                        div { class: "flex gap-3",
                            div { class: "badge badge-primary badge-outline font-bold min-w-fit",
                                "2"
                            }
                            div {
                                strong { "Online Wallet: " }
                                "Handles blockchain operations like checking balances, creating
                                addresses and transactions, and broadcasting to the network. This
                                component needs to connects to either your own Bitcoin node or the Heritage Service."
                            }
                        }
                    }

                    div { class: "alert alert-info",
                        div { class: "flex items-start gap-2",
                            "💡"
                            div {
                                "This split design is what enables the Heritage Service to provide
                                convenient Bitcoin inheritance features while maintaining the
                                highest security standards for your private keys."
                            }
                        }
                    }
                }
            }
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Default)]
enum OnlineWalletType {
    None,
    #[default]
    Service,
    Local,
}

#[derive(Debug, Clone, Copy, PartialEq, Default)]
enum OnlineWalletCreation {
    #[default]
    New,
    Restore,
    BindExisting,
}

/// Configuration for online wallet setup
#[derive(Debug, Clone, PartialEq)]
enum OnlineWalletConfig {
    None,
    Service(OnlineWalletCreationConfig),
    Local(OnlineWalletCreationConfig),
}
#[derive(Debug, Clone, PartialEq)]
enum OnlineWalletCreationConfig {
    New,
    Restore(HeritageWalletBackup),
    BindExisting(String),
}

/// Configuration for miscellaneous wallet options
#[derive(Debug, Clone, PartialEq)]
struct MiscConfig {
    block_inclusion_objective: BlockInclusionObjective,
    // None => no auto_feed
    // Some(count) => auto_feed <count> key
    auto_feed: Option<u32>,
}

/// Type aliases for component state signals
type WalletNameState = Signal<Result<String, ()>>;
type OnlineWalletConfigState = Signal<Result<OnlineWalletConfig, ()>>;
type OnlineWalletCreationConfigState = Signal<Result<OnlineWalletCreationConfig, ()>>;
type MiscConfigState = Signal<Result<MiscConfig, ()>>;

/// Main wallet creation form component
#[component]
fn WalletCreateForm() -> Element {
    log::debug!("WalletCreateForm Rendered");

    let database_service = state_management::use_database_service();
    let service_client_service = state_management::use_service_client_service();

    let service_wallets = helper_hooks::use_resource_service_wallets();
    use_context_provider(move || service_wallets);

    // Form state signals - split into individual signals for better performance
    let wallet_name_state: WalletNameState = use_signal(|| Err(()));
    let online_wallet_config_state: OnlineWalletConfigState = use_signal(|| Err(()));
    let key_provider_config_state: KeyProviderConfigState = use_signal(|| Err(()));
    let misc_config_state: MiscConfigState = use_signal(|| Err(()));

    let mut creating = use_signal(|| false);
    let mut creation_progress = use_signal(String::new);

    // Individual validation memos

    let online_wallet_fingerprint = use_memo(move || match &*online_wallet_config_state.read() {
        Ok(OnlineWalletConfig::Service(OnlineWalletCreationConfig::BindExisting(wallet_id))) => {
            service_wallets
                .lmap(|service_wallets| {
                    service_wallets
                        .iter()
                        .find(|&w| w.id == *wallet_id)
                        .map(|wm| wm.fingerprint)
                        .flatten()
                })
                .flatten()
        }
        Ok(OnlineWalletConfig::Local(OnlineWalletCreationConfig::Restore(bkp)))
        | Ok(OnlineWalletConfig::Service(OnlineWalletCreationConfig::Restore(bkp))) => {
            bkp.fingerprint().ok().flatten()
        }
        _ => None,
    });
    let key_provider_fingerprint = use_memo(move || match &*key_provider_config_state.read() {
        Ok(KeyProviderConfig::Ledger) => state_management::ledger_is_ready(),
        Ok(KeyProviderConfig::Local(LocalKeyCreationConfig::Restore { mnemo, password })) => Some(
            LocalKey::restore(mnemo.clone(), password.clone(), bitcoin_network::get())
                .fingerprint()
                .unwrap(),
        ),
        _ => None,
    });
    let future_fingerprints = use_memo(move || FutureFingerprints {
        key_provider: key_provider_fingerprint(),
        online_wallet: online_wallet_fingerprint(),
    });
    use_context_provider(move || future_fingerprints);

    let not_both_none = use_memo(move || {
        !(matches!(
            &*online_wallet_config_state.read(),
            Ok(OnlineWalletConfig::None)
        ) && matches!(
            &*key_provider_config_state.read(),
            Ok(KeyProviderConfig::None)
        ))
    });

    let coherent_fingerprint_error = use_memo(move || {
        let FutureFingerprints {
            key_provider,
            online_wallet,
        } = future_fingerprints();
        match (key_provider, online_wallet) {
            // If both user choice impose a fingerprint, it must be the same
            (Some(kp_fp), Some(ow_fp)) => if kp_fp != ow_fp {Some(CCStr::from(format!(
                "Incoherent fingerprint: Key provider has {kp_fp} but Online Wallet has {ow_fp}"
            )))} else {None},
            // If the key provider is a new LocalKey, and the online wallet has a fingerprint, that's an error
            (None, Some(ow_fg)) if matches!(
                &*key_provider_config_state.read(),
                Ok(KeyProviderConfig::Local(LocalKeyCreationConfig::New{..}))) => Some(CCStr::from(format!(
                "Online Wallet has fingerprint {ow_fg} but the newly generated Key Provider will have another one"
            ))),
            // Other cases are ok
            _ => None,
        }
    });
    let coherent_fingerprint = use_memo(move || coherent_fingerprint_error.read().is_none());
    let wallet_component_error = use_memo(move || {
        if !not_both_none() {
            Some(CCStr::from("The Wallet cannot be both Watch-Only and Sign-Only, please choose an Online Wallet and/or a Key Provider"))
        } else if !coherent_fingerprint() {
            coherent_fingerprint_error()
        } else {
            None
        }
    });

    // Miscellaneous options display memos
    let show_block_inclusion = use_memo(move || {
        !matches!(
            &*online_wallet_config_state.read(),
            Ok(OnlineWalletConfig::None)
        )
    });

    let show_auto_feed = use_memo(move || {
        !(matches!(
            &*online_wallet_config_state.read(),
            Ok(OnlineWalletConfig::None)
        ) || matches!(
            &*key_provider_config_state.read(),
            Ok(KeyProviderConfig::None)
        ))
    });

    let show_misc_options = use_memo(move || show_block_inclusion() || show_auto_feed());

    // Combined validation
    let form_valid = use_memo(move || {
        wallet_name_state.read().is_ok()
            && online_wallet_config_state.read().is_ok()
            && key_provider_config_state.read().is_ok()
            && misc_config_state.read().is_ok()
            && not_both_none()
            && coherent_fingerprint()
    });

    // Handle form submission
    let submit_form = move |_| async move {
        *creating.write() = true;
        *creation_progress.write() = "Starting wallet creation...".to_owned();

        let mut abort = |message: &str| {
            *creating.write() = false;
            alert_error(message);
            log::error!("{message}");
        };
        fn warn(message: impl AsRef<str>) {
            alert_warn(message.as_ref());
            log::warn!("{}", message.as_ref());
        }
        fn success(message: impl AsRef<str>) {
            alert_success(message.as_ref());
            log::info!("{}", message.as_ref());
        }

        let name = wallet_name_state().unwrap();
        let network = bitcoin_network::get();
        let Ok(kp_config) = key_provider_config_state() else {
            return abort("Invalid Key Provider configuration");
        };
        let kp_name = match kp_config {
            KeyProviderConfig::Local(_) => "Local",
            KeyProviderConfig::Ledger => "Ledger",
            KeyProviderConfig::None => "None",
        };
        let kp = match kp_config {
            KeyProviderConfig::None => AnyKeyProvider::None,
            KeyProviderConfig::Local(local_key_creation_config) => {
                let local_key = match local_key_creation_config {
                    LocalKeyCreationConfig::New {
                        word_count,
                        password,
                    } => {
                        *creation_progress.write() = "Generating Local Key Provider".to_owned();
                        LocalKey::generate(word_count, password, network)
                    }
                    LocalKeyCreationConfig::Restore { mnemo, password } => {
                        *creation_progress.write() = "Restoring Local Key Provider".to_owned();
                        LocalKey::restore(mnemo, password, network)
                    }
                };
                AnyKeyProvider::LocalKey(local_key)
            }
            KeyProviderConfig::Ledger => {
                *creation_progress.write() = "Connecting to Ledger Key Provider".to_owned();
                let ledger = match LedgerKey::new(network).await {
                    Ok(ledger) => ledger,
                    Err(e) => {
                        return abort(&format!("Could not create the Ledger key: {e}"));
                    }
                };
                AnyKeyProvider::Ledger(ledger)
            }
        };

        let Ok(MiscConfig {
            block_inclusion_objective,
            auto_feed,
        }) = misc_config_state()
        else {
            return abort("Invalid miscelaneous configurations");
        };

        let Ok(ow_config) = online_wallet_config_state() else {
            return abort("Invalid Online Wallet configuration");
        };
        let need_to_insert_in_state = matches!(
            ow_config,
            OnlineWalletConfig::Service(OnlineWalletCreationConfig::New)
                | OnlineWalletConfig::Service(OnlineWalletCreationConfig::Restore(_))
        );

        let ow_name = match ow_config {
            OnlineWalletConfig::Local(_) => "Local",
            OnlineWalletConfig::Service(_) => "Service",
            OnlineWalletConfig::None => "None",
        };
        let ow = match ow_config {
            OnlineWalletConfig::None => AnyOnlineWallet::None,
            OnlineWalletConfig::Service(owcc) => {
                let service_client =
                    state_management::heritage_service_client(service_client_service).await;

                let service_binding = match owcc {
                    OnlineWalletCreationConfig::New => {
                        *creation_progress.write() = "Generating Service Online Wallet".to_owned();
                        WalletServiceBinding::create(
                            &name,
                            None,
                            block_inclusion_objective,
                            service_client,
                            network,
                        )
                        .await
                    }
                    OnlineWalletCreationConfig::Restore(backup) => {
                        *creation_progress.write() = "Restoring Service Online Wallet".to_owned();
                        WalletServiceBinding::create(
                            &name,
                            Some(backup),
                            block_inclusion_objective,
                            service_client,
                            network,
                        )
                        .await
                    }
                    OnlineWalletCreationConfig::BindExisting(selected_service_wallet_id) => {
                        *creation_progress.write() = "Binding to Service Online Wallet".to_owned();
                        WalletServiceBinding::bind_by_id(
                            &selected_service_wallet_id,
                            service_client,
                            network,
                        )
                        .await
                    }
                };
                let service_binding = match service_binding {
                    Ok(service_binding) => service_binding,
                    Err(e) => {
                        return abort(&format!("Could not create the Service Binding: {e}"));
                    }
                };
                AnyOnlineWallet::Service(service_binding)
            }
            OnlineWalletConfig::Local(owcc) => {
                let backup = match owcc {
                    OnlineWalletCreationConfig::New => {
                        *creation_progress.write() = "Generating Local Online Wallet".to_owned();
                        None
                    }
                    OnlineWalletCreationConfig::Restore(bkp) => {
                        *creation_progress.write() = "Restoring Local Online Wallet".to_owned();
                        Some(bkp)
                    }
                    OnlineWalletCreationConfig::BindExisting(_) => {
                        unreachable!("Inexistent option for Local Wallet")
                    }
                };
                let local_wallet =
                    state_management::blocking_db_service_operation(database_service, move |db| {
                        LocalHeritageWallet::create(&db, backup, block_inclusion_objective)
                    })
                    .await;

                let local_wallet = match local_wallet {
                    Ok(local_wallet) => local_wallet,
                    Err(e) => {
                        return abort(&format!("Could not create the Local Wallet: {e}"));
                    }
                };
                AnyOnlineWallet::Local(local_wallet)
            }
        };

        let mut wallet = match Wallet::new(name.clone(), kp, ow) {
            Ok(wallet) => wallet,
            Err(e) => {
                return abort(&format!("Could not create the Wallet: {e}"));
            }
        };

        log::debug!("Created wallet: {wallet:?}");
        if show_auto_feed() {
            if let Some(count) = auto_feed {
                log::debug!("Will auto-feed {count} xpubs");
                *creation_progress.write() = format!(
                    "Generating {count} Account Extended Public Keys from {kp_name} Key Provider",
                );
                match wallet.derive_accounts_xpubs(0..count).await {
                    Ok(account_xpubs) => {
                        *creation_progress.write() = format!(
                            "Feeding {count} Account Extended Public Keys to {ow_name} Online Wallet",
                        );
                        match wallet.feed_account_xpubs(account_xpubs).await {
                            Ok(_) => {
                                log::debug!("Wallet after auto-feed: {wallet:?}");
                            }
                            Err(e) => warn(format!("Could not feed the account xpub: {e}")),
                        }
                    }
                    Err(e) => warn(format!("Could not generate the account xpub: {e}")),
                };
            }
        }

        if need_to_insert_in_state {
            let AnyOnlineWallet::Service(service_binding) = wallet.online_wallet() else {
                unreachable!();
            };
            state_management::inject_serviceable_wallet(
                service_client_service,
                service_binding.wallet_id().to_owned(),
                service_binding.fingerprint().ok(),
            )
        }

        *creation_progress.write() = "Saving new Wallet in database".to_owned();
        match state_management::create_wallet(database_service, wallet).await {
            Ok(_) => {
                success(format!("Wallet '{}' created successfully!", name));
                // Add context if onboarding is in progress
                if let OnboardingStatus::InProgress(ref mut onboarding) =
                    *state_management::ONBOARDING_STATUS.write()
                {
                    onboarding
                        .add_context(OnboardingContextItemId::WalletName.item(name.clone()), true);
                }

                navigator().push(Route::WalletView {
                    wallet_name: CCStr::from(name),
                });
            }
            Err(e) => {
                return abort(&format!("Failed to create wallet: {e}"));
            }
        }

        *creating.write() = false;
        *creation_progress.write() = String::new();
    };

    use_drop(|| log::debug!("WalletCreateForm Dropped"));

    rsx! {
        div { class: "px-6 py-8 flex flex-col gap-6",

            // Wallet Name Section
            WalletNameSection { wallet_name_state }

            // Key Provider Section
            KeyProviderSection {
                key_provider_config_state,
                wallet_component_error,
                flavor: KeyProviderSectionFlavor::Wallet,
            }

            // Online Wallet Section
            OnlineWalletSection { online_wallet_config_state, wallet_component_error }

            // Miscellaneous Options Section
            if show_misc_options() {
                MiscellaneousOptionsSection {
                    misc_config_state,
                    show_block_inclusion,
                    show_auto_feed,
                }
            }

            // Submit Button
            div { class: "flex justify-center mt-8",
                MaybeHighlight {
                    step: OnboardingStep::ClickCreateWalletButton,
                    progress: MaybeHighlightProgressType::ContextAdded(OnboardingContextItemId::WalletName),
                    button {
                        class: "btn btn-primary btn-lg",
                        disabled: !form_valid() || creating(),
                        onclick: submit_form,
                        if creating() {
                            span { class: "loading loading-spinner loading-sm mr-2" }
                            "Creating Wallet..."
                        } else {
                            DrawSvg::<WalletPlus> {}
                            "Create Wallet"
                        }
                    }
                }
            }

            InfoModal { is_open: creating, title: "Creating Wallet",
                div { class: "flex flex-col items-center gap-2 p-6",
                    div { class: "flex flex-row justify-center gap-4",
                        span { class: "loading loading-spinner loading-lg" }
                        div { class: "text-lg font-semibold", "{creation_progress}" }
                    }
                    div { class: "text-sm text-base-content/20",
                        "Please wait while your wallet is being created..."
                    }
                }
            }
        }
    }
}

/// Wallet name input section
#[component]
fn WalletNameSection(wallet_name_state: WalletNameState) -> Element {
    log::debug!("WalletNameSection Rendered");

    let wallet_names = helper_hooks::use_resource_wallet_names();
    let wallet_names_set = use_memo(move || {
        wallet_names
            .lmap(|wallet_names| wallet_names.iter().cloned().collect::<HashSet<_>>())
            .unwrap_or_default()
    });

    // Internal state - not exposed to parent
    let wallet_name = use_signal(String::new);

    // Internal validation
    let wallet_name_present = use_memo(move || !wallet_name.read().trim().is_empty());
    let wallet_name_available = use_memo(move || {
        let name_ref = wallet_name.read();
        let name = name_ref.trim();
        if name.is_empty() {
            true // Don't show error for empty name
        } else {
            !wallet_names_set.read().contains(name)
        }
    });
    let wallet_name_forbidden = use_memo(move || wallet_name.read().trim() == "create");
    let wallet_name_error = use_memo(move || {
        if !wallet_name_present() {
            Some(CCStr::from("Wallet name is required"))
        } else if wallet_name_forbidden() {
            Some(CCStr::from("\"create\" cannot be used as a Wallet name"))
        } else if !wallet_name_available() {
            Some(CCStr::from("This wallet name is already in use"))
        } else {
            None
        }
    });

    // Update parent signal when internal state changes
    use_effect(move || {
        let result = if wallet_name_error.read().is_none() {
            Ok(wallet_name.read().trim().to_owned())
        } else {
            Err(())
        };
        wallet_name_state.set(result);
    });

    let wallet_name_state_ok = use_memo(move || wallet_name_state.read().is_ok());

    use_drop(|| log::debug!("WalletNameSection Dropped"));

    rsx! {
        div { class: "card [--cardtitle-fs:var(--text-2xl)] border border-base-content/5 shadow-md",
            div { class: "card-body",
                h2 { class: "card-title", "Wallet Name" }
                div { class: "card-subtitle mb-4",
                    "The wallet name will be used in the app and must be unique."
                }
                MaybeHighlight {
                    step: OnboardingStep::InputName,
                    progress: MaybeHighlightProgressType::Signal(wallet_name_state_ok.into()),
                    div { class: "w-80",
                        InputField {
                            value: wallet_name,
                            placeholder: "Unique wallet name",
                            value_error: wallet_name_error,
                        }
                    }
                }
            }
        }
    }
}

/// Online wallet configuration section
#[component]
fn OnlineWalletSection(
    online_wallet_config_state: OnlineWalletConfigState,
    wallet_component_error: ReadOnlySignal<Option<CCStr>>,
) -> Element {
    log::debug!("OnlineWalletSection Rendered");
    // Internal state - not exposed to parent
    let online_wallet_type = use_signal(|| OnlineWalletType::Service);

    let online_wallet_creation_config_state: OnlineWalletCreationConfigState =
        use_signal(|| Err(()));

    // Internal validation

    // Update parent signal when internal state changes
    use_effect(move || {
        let result = match online_wallet_type() {
            OnlineWalletType::None => Ok(OnlineWalletConfig::None),
            OnlineWalletType::Service => {
                online_wallet_creation_config_state().map(OnlineWalletConfig::Service)
            }
            OnlineWalletType::Local => {
                online_wallet_creation_config_state().map(OnlineWalletConfig::Local)
            }
        };
        online_wallet_config_state.set(result);
    });

    let online_wallet_is_local =
        use_memo(move || matches!(online_wallet_type(), OnlineWalletType::Local));

    use_drop(|| log::debug!("OnlineWalletSection Dropped"));

    rsx! {
        div { class: "card [--cardtitle-fs:var(--text-2xl)] border border-base-content/5 shadow-md",
            div { class: "card-body",
                h2 { class: "card-title", "Online Wallet" }
                div { class: "card-subtitle",
                    "The online wallet handles blockchain synchronization, address generation, and transaction broadcasting. Using the Heritage Service is recommended for security and reliability"
                }

                Divider { "Online Wallet Type" }

                RadioChoices { count: 3,
                    RadioChoice {
                        name: "online_wallet",
                        state: online_wallet_type,
                        value: OnlineWalletType::None,
                        title: "None (Sign-Only)",
                        subtitle: "No blockchain access, cannot sync or generate addresses",
                    }

                    MaybeHighlight {
                        step: OnboardingStep::SelectLocalOnlineWallet,
                        progress: MaybeHighlightProgressType::Signal(online_wallet_is_local.into()),
                        RadioChoice {
                            name: "online_wallet",
                            state: online_wallet_type,
                            value: OnlineWalletType::Local,
                            title: "Local Node",
                            subtitle: "Use your own Electrum or Bitcoin Core node",
                        }
                    }

                    RadioChoice {
                        name: "online_wallet",
                        state: online_wallet_type,
                        value: OnlineWalletType::Service,
                        title: "Heritage Service",
                        subtitle: "Use the Heritage service for sync and Heritage configuration management (recommended)",
                    }
                }

                div {
                    class: "fieldset-label text-error",
                    class: if wallet_component_error.read().is_none() { "invisible" },
                    if let Some(e) = wallet_component_error() {
                        {e}
                    } else {
                        "ph"
                    }
                }

                // Online wallet specific options
                match online_wallet_type() {
                    OnlineWalletType::Service => rsx! {
                        ServiceWalletOptions { online_wallet_creation_config_state }
                    },
                    OnlineWalletType::Local => rsx! {
                        LocalWalletOptions { online_wallet_creation_config_state }
                    },
                    OnlineWalletType::None => rsx! {},
                }
            }
        }
    }
}

/// Service wallet specific options
#[component]
fn ServiceWalletOptions(
    online_wallet_creation_config_state: OnlineWalletCreationConfigState,
) -> Element {
    log::debug!("ServiceWalletOptions Rendered");

    let future_fingerprints = use_context::<Memo<FutureFingerprints>>();

    // Internal state - not exposed to parent
    let online_wallet_creation = use_signal(|| OnlineWalletCreation::New);

    let heritage_wallet_backup_state: Signal<Result<HeritageWalletBackup, CCStr>> =
        use_signal(|| Err(CCStr::default()));
    let service_wallet_id_state: Signal<Result<String, ()>> = use_signal(|| Err(()));

    // Internal validation
    let expected_fingerprint = future_fingerprints().key_provider;

    // Update parent signal when internal state changes
    use_effect(move || {
        let result = match online_wallet_creation() {
            OnlineWalletCreation::New => Ok(OnlineWalletCreationConfig::New),
            OnlineWalletCreation::Restore => heritage_wallet_backup_state()
                .map(OnlineWalletCreationConfig::Restore)
                .map_err(|_| ()),
            OnlineWalletCreation::BindExisting => {
                service_wallet_id_state().map(OnlineWalletCreationConfig::BindExisting)
            }
        };
        online_wallet_creation_config_state.set(result);
    });

    use_drop(|| log::debug!("ServiceWalletOptions Dropped"));

    rsx! {
        Divider { "Service Online Wallet Creation Options" }

        RadioChoices { count: 3,
            RadioChoice {
                name: "service_mode",
                state: online_wallet_creation,
                value: OnlineWalletCreation::New,
                title: "Create New",
                subtitle: "Create a new wallet on the Heritage service",
            }

            RadioChoice {
                name: "service_mode",
                state: online_wallet_creation,
                value: OnlineWalletCreation::Restore,
                title: "Restore from Backup",
                subtitle: "Restore wallet from descriptors backup",
            }

            RadioChoice {
                name: "service_mode",
                state: online_wallet_creation,
                value: OnlineWalletCreation::BindExisting,
                title: "Bind to Existing",
                subtitle: "Connect to an existing Heritage service wallet",
            }
        }

        // Mode-specific options
        match online_wallet_creation() {
            OnlineWalletCreation::Restore => rsx! {
                
                Divider { "Backup Restore" }
                
                BackupRestoreSection { heritage_wallet_backup_state, expected_fingerprint }
            },
            OnlineWalletCreation::BindExisting => rsx! {
                ServiceWalletSelection { service_wallet_id_state }
            },
            OnlineWalletCreation::New => rsx! {},
        }
    }
}

/// Service wallet selection component for binding to existing wallets
#[component]
fn ServiceWalletSelection(service_wallet_id_state: Signal<Result<String, ()>>) -> Element {
    log::debug!("ServiceWalletSelection Rendered");

    let service_wallets = use_context::<Resource<Vec<CheapClone<HeritageWalletMeta>>>>();
    let future_fingerprints = use_context::<Memo<FutureFingerprints>>();

    // Internal state - not exposed to parent
    let mut selected_service_wallet_id = use_signal(String::new);

    // Internal validation
    let service_wallet_selected = use_memo(move || !selected_service_wallet_id().trim().is_empty());
    let service_wallet_selected_error = use_memo(move || {
        if !service_wallet_selected() {
            Some(CCStr::from("Select the existing wallet to bind to"))
        } else if !future_fingerprints().coherents() {
            Some(CCStr::from(
                "Selected wallet is not compatible with the key provider",
            ))
        } else {
            None
        }
    });

    let (error_display, mut signal_activity, onfocusout) =
        use_future_error_feedback(service_wallet_selected_error.into());

    // Update parent signal when internal state changes
    use_effect(move || {
        let result = if service_wallet_selected() {
            Ok(selected_service_wallet_id())
        } else {
            Err(())
        };
        service_wallet_id_state.set(result);
    });

    use_drop(|| log::debug!("ServiceWalletSelection Dropped"));

    rsx! {
        div { class: "mt-4",
            fieldset { class: "fieldset w-full",
                legend { class: "fieldset-legend", "Existing Service Wallet" }
                select {
                    class: "select w-full",
                    class: if error_display().is_some() { "select-error" },
                    value: selected_service_wallet_id(),
                    onchange: move |evt| {
                        signal_activity();
                        selected_service_wallet_id.set(evt.value());
                    },
                    onfocusout,
                    option {
                        selected: selected_service_wallet_id.read().is_empty(),
                        disabled: true,
                        "-- Select a wallet --"
                    }
                    if let Some(wallets) = service_wallets.read().as_ref() {
                        for wallet in wallets.iter() {
                            option {
                                value: "{wallet.id}",
                                selected: wallet.id == selected_service_wallet_id.read().as_str(),
                                "{wallet.name}"
                                if let Some(fg) = wallet.fingerprint {
                                    " ({fg})"
                                } else {
                                    " (No Fingerprint)"
                                }
                            }
                        }
                    }
                }
                div {
                    class: "fieldset-label",
                    class: if error_display().is_none() { "invisible" } else { "text-error" },
                    if let Some(e) = error_display() {
                        {e}
                    } else {
                        "ph"
                    }
                }
            }
        }
    }
}

/// Local wallet specific options
#[component]
fn LocalWalletOptions(
    online_wallet_creation_config_state: OnlineWalletCreationConfigState,
) -> Element {
    log::debug!("LocalWalletOptions Rendered");

    let future_fingerprints = use_context::<Memo<FutureFingerprints>>();

    // Internal state - not exposed to parent
    let online_wallet_creation = use_signal(|| OnlineWalletCreation::New);

    let heritage_wallet_backup_state: Signal<Result<HeritageWalletBackup, CCStr>> =
        use_signal(|| Err(CCStr::default()));

    // Internal validation
    let expected_fingerprint = future_fingerprints().key_provider;

    // Update parent signal when internal state changes
    use_effect(move || {
        let result = match online_wallet_creation() {
            OnlineWalletCreation::New => Ok(OnlineWalletCreationConfig::New),
            OnlineWalletCreation::Restore => heritage_wallet_backup_state()
                .map(OnlineWalletCreationConfig::Restore)
                .map_err(|_| ()),
            OnlineWalletCreation::BindExisting => {
                unreachable!("Does not exist for local online wallets")
            }
        };
        online_wallet_creation_config_state.set(result);
    });

    use_drop(|| log::debug!("LocalWalletOptions Dropped"));

    rsx! {
        Divider { "Local Online Wallet Creation Options" }

        RadioChoices { count: 2,
            RadioChoice {
                name: "local_mode",
                state: online_wallet_creation,
                value: OnlineWalletCreation::New,
                title: "Create New",
                subtitle: "Create a new local wallet",
            }

            RadioChoice {
                name: "local_mode",
                state: online_wallet_creation,
                value: OnlineWalletCreation::Restore,
                title: "Restore from Backup",
                subtitle: "Restore wallet from descriptors backup",
            }
        }

        // Mode-specific options
        match online_wallet_creation() {
            OnlineWalletCreation::Restore => rsx! {
                
                Divider { "Backup Restore" }
                
                BackupRestoreSection { heritage_wallet_backup_state, expected_fingerprint }
            },
            OnlineWalletCreation::New | OnlineWalletCreation::BindExisting => {
                rsx! {}
            }
        }
    }
}

/// Miscellaneous options section
#[component]
fn MiscellaneousOptionsSection(
    misc_config_state: MiscConfigState,
    show_block_inclusion: ReadOnlySignal<bool>,
    show_auto_feed: ReadOnlySignal<bool>,
) -> Element {
    log::debug!("MiscellaneousOptionsSection Rendered");

    // Internal state - not exposed to parent
    let mut show_collapsed = use_signal(|| false);
    let mut block_inclusion_objective = use_signal(|| BlockInclusionObjective::default());

    let mut auto_feed_xpubs = use_signal(|| true);
    let mut auto_feed_keys_count = use_signal(|| 20);

    // Internal validation

    // Update parent signal when internal state changes
    use_effect(move || {
        let result = Ok(MiscConfig {
            block_inclusion_objective: block_inclusion_objective(),
            auto_feed: auto_feed_xpubs().then(|| auto_feed_keys_count()),
        });
        misc_config_state.set(result);
    });

    use_drop(|| log::debug!("MiscellaneousOptionsSection Dropped"));

    rsx! {
        div { class: "collapse collapse-arrow border border-base-content/5 shadow-md",
            input {
                r#type: "checkbox",
                checked: show_collapsed(),
                onchange: move |evt| show_collapsed.set(evt.checked()),
            }
            div { class: "collapse-title text-2xl font-semibold", "Miscellaneous Options" }
            div { class: "collapse-content",
                div { class: "flex flex-col gap-4",
                    if show_block_inclusion() {
                        fieldset { class: "fieldset",
                            legend { class: "fieldset-legend", "Block Inclusion Objective" }
                            div { class: "fieldset-description",
                                "Target number of blocks for transaction confirmation (affects fee calculation and can be changed later)"
                            }
                            label { class: "input",
                                input {
                                    r#type: "number",
                                    min: "{BlockInclusionObjective::MIN}",
                                    max: "{BlockInclusionObjective::MAX}",
                                    value: "{block_inclusion_objective()}",
                                    oninput: move |evt| {
                                        let new_bio = match evt.parsed() {
                                            Ok(new_bio) => new_bio,
                                            Err(ParseBlockInclusionObjectiveError::InvalidInt) => {
                                                BlockInclusionObjective::default()
                                            }
                                            Err(ParseBlockInclusionObjectiveError::ValueTooLow) => {
                                                BlockInclusionObjective::MIN
                                            }
                                            Err(ParseBlockInclusionObjectiveError::ValueTooHigh) => {
                                                BlockInclusionObjective::MAX
                                            }
                                        };
                                        block_inclusion_objective.set(new_bio);
                                    },
                                }
                                span { class: "label", "blocks" }
                            }
                            div { class: "fieldset-label",
                                "Min: {BlockInclusionObjective::MIN}, Max: {BlockInclusionObjective::MAX}, Default: {BlockInclusionObjective::default()}"
                            }
                        }
                    }

                    if show_auto_feed() {
                        fieldset { class: "fieldset",
                            legend { class: "fieldset-legend", "Auto-feed Account Extended Public Keys" }
                            div { class: "fieldset-description",
                                "If enabled, the "
                                span { class: "font-bold", "Online Wallet" }
                                " will automatically be fed "
                                span { class: "italic", "Account Extended Public Keys" }
                                " using the "
                                span { class: "font-bold", "Key Provider" }
                                " to generate them."
                            }
                            label { class: "label",
                                input {
                                    r#type: "checkbox",
                                    class: "toggle toggle-secondary",
                                    checked: auto_feed_xpubs(),
                                    onchange: move |evt| auto_feed_xpubs.set(evt.checked()),
                                }
                                span { class: "text-base ml-2",
                                    if auto_feed_xpubs() {
                                        "Enabled"
                                    } else {
                                        "Disabled"
                                    }
                                }
                            }
                            if auto_feed_xpubs() {
                                div { class: "fieldset-description",
                                    "Number of account extended public keys to generate and feed to the online wallet"
                                }
                                label { class: "input",
                                    input {
                                        r#type: "number",
                                        min: "1",
                                        max: "100",
                                        value: auto_feed_keys_count(),
                                        oninput: move |evt| {
                                            if let Ok(value) = evt.value().parse::<u32>() {
                                                auto_feed_keys_count.set(value.clamp(1, 100));
                                            } else {
                                                auto_feed_keys_count.set(20);
                                            }
                                        },
                                    }
                                    span { class: "label", "keys" }
                                }
                                div { class: "fieldset-label", "Min: 1, Max: 100, Default: 20" }
                            }
                        }
                    }
                }
            }
        }
    }
}
