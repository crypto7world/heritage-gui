use crate::prelude::*;

use std::collections::HashSet;

use btc_heritage_wallet::{
    btc_heritage::{utils::bitcoin_network, HeritageWalletBackup},
    heritage_provider::{
        AnyHeritageProvider, LocalWallet, ServiceBinding as HeritageServiceBinding,
    },
    heritage_service_api_client::Fingerprint,
    AnyKeyProvider, BoundFingerprint, DatabaseItem, HeirWallet, LocalKey,
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
        svg::{DrawSvg, Gold},
    },
    utils::{log_error_ccstr, CCStr},
    Route,
};

/// Component for creating new heir wallets
#[component]
pub fn HeirWalletCreateView() -> Element {
    log::debug!("HeirWalletCreateView Rendered");

    use_drop(|| log::debug!("HeirWalletCreateView Dropped"));

    rsx! {
        super::TitledView {
            title: CCStr::from("Create Heir Wallet"),
            subtitle: CCStr::from("Create a new heir wallet to manage and spend your inheritances."),
            left: rsx! {
                BackButton { route: Route::HeirWalletListView {} }
            },
            HeirWalletCreateForm {}
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Default)]
enum HeritageProviderType {
    None,
    #[default]
    Service,
    Local,
}

/// Configuration for heritage provider setup
#[derive(Debug, Clone, PartialEq)]
enum HeritageProviderConfig {
    None,
    Local {
        fingerprint: Option<Fingerprint>,
        backup: HeritageWalletBackup,
    },
    Service {
        fingerprint: Option<Fingerprint>,
    },
}

/// Type aliases for component state signals
type HeirWalletNameState = Signal<Result<String, ()>>;
type HeritageProviderConfigState = Signal<Result<HeritageProviderConfig, ()>>;

/// Main heir wallet creation form component
#[component]
fn HeirWalletCreateForm() -> Element {
    log::debug!("HeirWalletCreateForm Rendered");

    let database_service = state_management::use_database_service();
    let service_client_service = state_management::use_service_client_service();

    let service_heritages = helper_hooks::use_resource_service_heritages();
    use_context_provider(move || service_heritages);

    // Form state signals - split into individual signals for better performance
    let heirwallet_name_state: HeirWalletNameState = use_signal(|| Err(()));
    let heritage_provider_config_state: HeritageProviderConfigState = use_signal(|| Err(()));
    let key_provider_config_state: KeyProviderConfigState = use_signal(|| Err(()));

    let mut creating = use_signal(|| false);

    // Individual validation memos

    let key_provider_fingerprint = use_memo(move || match &*key_provider_config_state.read() {
        Ok(KeyProviderConfig::Ledger) => Some(state_management::ledger_is_ready()),
        Ok(KeyProviderConfig::Local(lkcc)) => match lkcc {
            LocalKeyCreationConfig::New { .. } => Some(None),
            LocalKeyCreationConfig::Restore { mnemo, password } => Some(
                LocalKey::restore(mnemo.clone(), password.clone(), bitcoin_network::get())
                    .fingerprint()
                    .ok(),
            ),
        },
        Ok(KeyProviderConfig::None) => None,
        _ => Some(None),
    });
    use_context_provider(move || key_provider_fingerprint);

    let not_both_none = use_memo(move || {
        !(matches!(
            &*heritage_provider_config_state.read(),
            Ok(HeritageProviderConfig::None)
        ) && matches!(
            &*key_provider_config_state.read(),
            Ok(KeyProviderConfig::None)
        ))
    });

    let wallet_component_error = use_memo(move || {
        if !not_both_none() {
            Some(CCStr::from("The heir wallet cannot be both Watch-Only and Sign-Only. Please choose a Heritage Provider and/or a Key Provider."))
        } else {
            None
        }
    });

    // Combined validation
    let form_valid = use_memo(move || {
        heirwallet_name_state.read().is_ok()
            && heritage_provider_config_state.read().is_ok()
            && key_provider_config_state.read().is_ok()
            && not_both_none()
    });

    // Handle form submission
    let submit_form = move |_| async move {
        *creating.write() = true;

        let mut abort = |message: &str| {
            *creating.write() = false;
            alert_error(message);
            log::error!("{message}");
        };
        fn success(message: impl AsRef<str>) {
            alert_success(message.as_ref());
            log::info!("{}", message.as_ref());
        }

        let name = heirwallet_name_state().unwrap();
        let network = bitcoin_network::get();
        let Ok(kp_config) = key_provider_config_state() else {
            return abort("Invalid Key Provider configuration");
        };
        let kp = match kp_config {
            KeyProviderConfig::None => AnyKeyProvider::None,
            KeyProviderConfig::Local(local_key_creation_config) => {
                let local_key = match local_key_creation_config {
                    LocalKeyCreationConfig::New {
                        word_count,
                        password,
                    } => LocalKey::generate(word_count, password, network),
                    LocalKeyCreationConfig::Restore { mnemo, password } => {
                        LocalKey::restore(mnemo, password, network)
                    }
                };
                AnyKeyProvider::LocalKey(local_key)
            }
            KeyProviderConfig::Ledger => {
                let ledger = match btc_heritage_wallet::LedgerKey::new(network).await {
                    Ok(ledger) => ledger,
                    Err(e) => {
                        return abort(&format!("Could not create the Ledger key: {e}"));
                    }
                };
                AnyKeyProvider::Ledger(ledger)
            }
        };

        let Ok(hp_config) = heritage_provider_config_state() else {
            return abort("Invalid Heritage Provider configuration");
        };

        // Will be none if the kp is None
        // Else it cannot fail and it will be Some
        let kp_fingerprint = kp.fingerprint().ok();
        let fingerprint = match hp_config {
            HeritageProviderConfig::None => None,
            HeritageProviderConfig::Service { fingerprint }
            | HeritageProviderConfig::Local { fingerprint, .. } => {
                let fingerprint = if let Some(kp_fingerprint) = kp_fingerprint {
                    kp_fingerprint
                } else {
                    if let Some(fingerprint) = fingerprint {
                        fingerprint
                    } else {
                        return abort(
                            "Cannot retrieve the Heir fingerprint for the Heritage Provider",
                        );
                    }
                };
                Some(fingerprint)
            }
        };
        let hp = match hp_config {
            HeritageProviderConfig::None => AnyHeritageProvider::None,
            HeritageProviderConfig::Service { .. } => {
                let service_client =
                    state_management::heritage_service_client(service_client_service).await;

                let service_binding =
                    HeritageServiceBinding::new(fingerprint.unwrap(), service_client);
                AnyHeritageProvider::Service(service_binding)
            }
            HeritageProviderConfig::Local { backup, .. } => {
                let local_wallet =
                    state_management::blocking_db_service_operation(database_service, move |db| {
                        LocalWallet::create(fingerprint.unwrap(), &db, backup)
                    })
                    .await;

                let local_wallet = match local_wallet {
                    Ok(local_wallet) => local_wallet,
                    Err(e) => {
                        return abort(&format!(
                            "Could not create the Local Heritage Provider: {e}"
                        ));
                    }
                };
                AnyHeritageProvider::LocalWallet(local_wallet)
            }
        };

        let heirwallet = match HeirWallet::new(name.clone(), kp, hp) {
            Ok(heirwallet) => heirwallet,
            Err(e) => {
                return abort(&format!("Could not create the Heir Wallet: {e}"));
            }
        };

        log::debug!("Created heir wallet: {heirwallet:?}");

        match state_management::blocking_db_service_operation(database_service, move |mut db| {
            heirwallet.create(&mut db)
        })
        .await
        {
            Ok(()) => {
                *creating.write() = false;
                success(format!("Heir wallet '{name}' created successfully"));

                // Add context if onboarding is in progress
                if let OnboardingStatus::InProgress(ref mut onboarding) =
                    *state_management::ONBOARDING_STATUS.write()
                {
                    onboarding.add_context(
                        OnboardingContextItemId::HeirWalletName.item(name.clone()),
                        true,
                    );
                }

                navigator().push(Route::HeirWalletView {
                    heirwallet_name: CCStr::from(name),
                });
            }
            Err(e) => {
                return abort(&format!("Could not save the heir wallet to database: {e}"));
            }
        }
    };

    use_drop(|| log::debug!("HeirWalletCreateForm Dropped"));

    rsx! {
        div { class: "px-6 py-8 flex flex-col gap-6",

            // Form sections
            HeirWalletNameSection { heirwallet_name_state }

            KeyProviderSection {
                key_provider_config_state,
                wallet_component_error,
                flavor: KeyProviderSectionFlavor::HeirWallet,
            }

            HeritageProviderSection {
                heritage_provider_config_state,
                wallet_component_error,
            }

            // Submit Button
            div { class: "flex justify-center mt-8",
                MaybeHighlight {
                    step: OnboardingStep::ClickCreateHeirWalletButton,
                    progress: MaybeHighlightProgressType::ContextAdded(OnboardingContextItemId::HeirWalletName),
                    button {
                        class: "btn btn-primary btn-lg",
                        disabled: !form_valid() || creating(),
                        onclick: submit_form,
                        if creating() {
                            span { class: "loading loading-spinner loading-sm mr-2" }
                            "Creating Heir Wallet..."
                        } else {
                            DrawSvg::<Gold> {}
                            "Create Heir Wallet"
                        }
                    }
                }
            }
        }
    }
}

/// Heir wallet name input section
#[component]
fn HeirWalletNameSection(heirwallet_name_state: HeirWalletNameState) -> Element {
    log::debug!("HeirWalletNameSection Rendered");

    let heirwallet_names = helper_hooks::use_resource_heirwallet_names();
    let heirwallet_names_set = use_memo(move || {
        heirwallet_names
            .lmap(|heirwallet_names| heirwallet_names.iter().cloned().collect::<HashSet<_>>())
            .unwrap_or_default()
    });

    // Internal state - not exposed to parent
    let heirwallet_name = use_signal(String::new);

    // Internal validation
    let heirwallet_name_present = use_memo(move || !heirwallet_name.read().trim().is_empty());
    let heirwallet_name_available = use_memo(move || {
        let name_ref = heirwallet_name.read();
        let name = name_ref.trim();
        if name.is_empty() {
            true // Don't show error for empty name
        } else {
            !heirwallet_names_set.read().contains(name)
        }
    });
    let heirwallet_name_forbidden = use_memo(move || heirwallet_name.read().trim() == "create");
    let heirwallet_name_error = use_memo(move || {
        if !heirwallet_name_present() {
            Some(CCStr::from("Heir wallet name is required"))
        } else if heirwallet_name_forbidden() {
            Some(CCStr::from(
                "\"create\" cannot be used as a Heir Wallet name",
            ))
        } else if !heirwallet_name_available() {
            Some(CCStr::from("This heir wallet name is already in use"))
        } else {
            None
        }
    });

    // Update parent signal when internal state changes
    use_effect(move || {
        let result = if heirwallet_name_error.read().is_none() {
            Ok(heirwallet_name.read().trim().to_owned())
        } else {
            Err(())
        };
        heirwallet_name_state.set(result);
    });

    let heirwallet_name_state_ok = use_memo(move || heirwallet_name_state.read().is_ok());

    use_drop(|| log::debug!("HeirWalletNameSection Dropped"));

    rsx! {
        div { class: "card [--cardtitle-fs:var(--text-2xl)] border border-base-content/5 shadow-md",
            div { class: "card-body",
                h2 { class: "card-title", "Heir Wallet Name" }
                div { class: "card-subtitle mb-4",
                    "The heir wallet name will be used in the application and must be unique."
                }
                MaybeHighlight {
                    step: OnboardingStep::InputName,
                    progress: MaybeHighlightProgressType::Signal(heirwallet_name_state_ok.into()),
                    div { class: "w-80",
                        InputField {
                            value: heirwallet_name,
                            placeholder: "Unique heir wallet name",
                            value_error: heirwallet_name_error,
                        }
                    }
                }
            }
        }
    }
}

/// Heritage provider configuration section
#[component]
fn HeritageProviderSection(
    heritage_provider_config_state: HeritageProviderConfigState,
    wallet_component_error: ReadOnlySignal<Option<CCStr>>,
) -> Element {
    log::debug!("HeritageProviderSection Rendered");

    // Internal state - not exposed to parent
    let heritage_provider_type = use_signal(|| HeritageProviderType::Service);

    let heritage_wallet_backup_state = use_signal(|| Err(CCStr::from("No backup provided")));

    let heritage_provider_fingerprint_state = use_signal(|| Err(()));
    // Internal validation

    // Update parent signal when internal state changes
    use_effect(move || {
        let result = match heritage_provider_type() {
            HeritageProviderType::None => Ok(HeritageProviderConfig::None),
            HeritageProviderType::Service => heritage_provider_fingerprint_state()
                .map(|fingerprint| HeritageProviderConfig::Service { fingerprint }),
            HeritageProviderType::Local => {
                match (
                    heritage_provider_fingerprint_state(),
                    heritage_wallet_backup_state(),
                ) {
                    (Ok(fingerprint), Ok(backup)) => Ok(HeritageProviderConfig::Local {
                        fingerprint,
                        backup,
                    }),
                    _ => Err(()),
                }
            }
        };
        heritage_provider_config_state.set(result);
    });

    let heritage_provider_is_local =
        use_memo(move || matches!(heritage_provider_type(), HeritageProviderType::Local));
    let heritage_backup_is_ok = use_memo(move || heritage_wallet_backup_state.read().is_ok());

    use_drop(|| log::debug!("HeritageProviderSection Dropped"));

    rsx! {
        div { class: "card [--cardtitle-fs:var(--text-2xl)] border border-base-content/5 shadow-md",
            div { class: "card-body",
                h2 { class: "card-title", "Heritage Provider" }
                div { class: "card-subtitle mb-4",
                    "The heritage provider manages access to inheritances you are eligible to claim."
                }

                RadioChoices { count: 3,
                    RadioChoice {
                        name: "heritage_provider",
                        state: heritage_provider_type,
                        value: HeritageProviderType::None,
                        title: "None (Sign-Only)",
                        subtitle: "No blockchain access; cannot find inheritances",
                    }

                    MaybeHighlight {
                        step: OnboardingStep::SelectLocalHeritageProvider,
                        progress: MaybeHighlightProgressType::Signal(heritage_provider_is_local.into()),
                        RadioChoice {
                            name: "heritage_provider",
                            state: heritage_provider_type,
                            value: HeritageProviderType::Local,
                            title: "Local",
                            subtitle: "Find inheritances using a Heritage Wallet backup and your own Electrum or Bitcoin Core node",
                        }
                    }

                    RadioChoice {
                        name: "heritage_provider",
                        state: heritage_provider_type,
                        value: HeritageProviderType::Service,
                        title: "Service",
                        subtitle: "Use the Heritage Service to find inheritances",
                    }
                }

                match heritage_provider_type() {
                    HeritageProviderType::None => rsx! {},
                    HeritageProviderType::Local => rsx! {
                        HeritageProviderHeirFingerprint { heritage_provider_fingerprint_state }
                        
                        Divider { "Local Heritage Provider Creation" }
                        div { class: "card-subtitle mb-4",
                            "You need a backup of the original owner's Heritage Wallet to create this Heritage Provider."
                        }
                        MaybeHighlight {
                            step: OnboardingStep::ProvideLocalWalletBackup,
                            progress: MaybeHighlightProgressType::Signal(heritage_backup_is_ok.into()),
                            BackupRestoreSection { heritage_wallet_backup_state, expected_fingerprint: None }
                        }
                    },
                    HeritageProviderType::Service => rsx! {
                        HeritageProviderHeirFingerprint { heritage_provider_fingerprint_state }
                    },
                }
            }
        }
    }
}
#[component]
fn HeritageProviderHeirFingerprint(
    heritage_provider_fingerprint_state: Signal<Result<Option<Fingerprint>, ()>>,
) -> Element {
    log::debug!("HeritageProviderHeirFingerprint Rendered");

    let key_provider_fingerprint = use_context::<Memo<Option<Option<Fingerprint>>>>();

    // Internal state - not exposed to parent
    let mut fingerprint_str = use_signal(|| match heritage_provider_fingerprint_state() {
        Ok(Some(fp)) => fp.to_string(),
        _ => String::new(),
    });

    // Internal validation
    let fingerprint_provided = use_memo(move || fingerprint_str().len() == 8);
    let fingerprint_monomode = use_memo(move || !fingerprint_str().is_empty());
    let fingerprint = use_memo(move || {
        fingerprint_str
            .read()
            .as_str()
            .parse::<Fingerprint>()
            .map_err(log_error_ccstr)
    });
    let fingerprint_error = use_memo(move || {
        if !fingerprint_provided() {
            Some(CCStr::from("Provide an 8-character fingerprint"))
        } else if let Err(ref e) = *fingerprint.read() {
            Some(e.clone())
        } else {
            None
        }
    });

    // Update parent signal when internal state changes
    use_effect(move || {
        let result = match key_provider_fingerprint() {
            Some(_) => Ok(None),
            None => {
                if fingerprint_error.read().is_none() {
                    fingerprint().map(Some).map_err(|_| ())
                } else {
                    Err(())
                }
            }
        };
        heritage_provider_fingerprint_state.set(result);
    });

    let (error_display, mut signal_activity, onfocusout) =
        use_future_error_feedback(fingerprint_error.into());

    use_drop(|| log::debug!("HeritageProviderHeirFingerprint Dropped"));

    rsx! {
        Divider { "Heir Fingerprint" }

        fieldset { class: "fieldset",
            legend { class: "fieldset-legend", "Heir Fingerprint" }

            div { class: "fieldset-description",
                "The fingerprint of your Key Provider, which will be used to filter inheritances and find the ones you are eligible for."
            }
            if let Some(ofp) = key_provider_fingerprint() {
                input {
                    r#type: "text",
                    class: "input w-80",
                    class: if ofp.is_some() { "font-mono" },
                    value: ofp.map(|fp| fp.to_string()).unwrap_or_default(),
                    placeholder: "Will be retrieved from the Key Provider",
                    disabled: true,
                }
                div {
                    class: "fieldset-label",
                    class: if ofp.is_none() { "invisible" },
                    "Retrieved from the Key Provider"
                }
            } else {
                input {
                    r#type: "text",
                    class: "input w-80",
                    class: if fingerprint_monomode() { "font-mono" },
                    class: if error_display().is_some() { "input-error" },
                    class: if fingerprint.read().is_ok() { "input-success" },
                    placeholder: "Provide an 8-character fingerprint",
                    value: "{fingerprint_str.read()}",
                    maxlength: 8,
                    oninput: move |evt| {
                        signal_activity();
                        let mut val = evt.value().to_lowercase();
                        val.retain(|c| "1234567890abcdef".contains(c));
                        fingerprint_str.set(val);
                    },
                    onfocusout,
                }
                div {
                    class: "fieldset-label",
                    class: if error_display().is_some() { "text-error" },
                    if let Some(e) = error_display() {
                        {e}
                    } else {
                        "8 characters: digits and letters 'a' to 'f' only"
                    }
                }
            }
        }
    }
}
