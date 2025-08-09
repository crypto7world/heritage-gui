use crate::prelude::*;

use btc_heritage_wallet::{
    btc_heritage::utils::bitcoin_network, heritage_service_api_client::Fingerprint,
    BoundFingerprint, Language, LocalKey, Mnemonic,
};

use crate::{
    components::{
        app_config::LedgerServiceStatusWithDesc,
        inputs::{use_future_error_feedback, InputField, RadioChoice, RadioChoices},
        misc::Divider,
    },
    utils::{log_error_ccstr, CCStr, FutureFingerprints},
};

#[derive(Debug, Clone, Copy, PartialEq)]
enum KeyProviderType {
    None,
    Local,
    Ledger,
}

#[derive(Debug, Clone, Copy, PartialEq)]
enum LocalKeyCreation {
    New,
    Restore,
}

/// Configuration for key provider setup
#[derive(Debug, Clone, PartialEq)]
pub enum KeyProviderConfig {
    None,
    Ledger,
    Local(LocalKeyCreationConfig),
}
#[derive(Debug, Clone, PartialEq)]
pub enum LocalKeyCreationConfig {
    New {
        word_count: usize,
        password: Option<String>,
    },
    Restore {
        mnemo: Mnemonic,
        password: Option<String>,
    },
}

pub type KeyProviderConfigState = Signal<Result<KeyProviderConfig, ()>>;
type LocalKeyCreationConfigState = Signal<Result<LocalKeyCreationConfig, ()>>;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum KeyProviderSectionFlavor {
    Wallet,
    Heir,
    HeirWallet,
}

/// Key provider configuration section
#[component]
pub fn KeyProviderSection(
    key_provider_config_state: KeyProviderConfigState,
    wallet_component_error: ReadOnlySignal<Option<CCStr>>,
    flavor: KeyProviderSectionFlavor,
) -> Element {
    log::debug!("KeyProviderSection Rendered");

    use_context_provider(move || flavor);

    let key_provider_fingerprint = use_memo(move || match &*key_provider_config_state.read() {
        Ok(KeyProviderConfig::Ledger) => state_management::ledger_is_ready(),
        Ok(KeyProviderConfig::Local(LocalKeyCreationConfig::Restore { mnemo, password })) => Some(
            LocalKey::restore(mnemo.clone(), password.clone(), bitcoin_network::get())
                .fingerprint()
                .unwrap(),
        ),
        _ => None,
    });
    use_context_provider(move || key_provider_fingerprint);

    // Internal state - not exposed to parent
    let key_provider_type = use_signal(|| match flavor {
        KeyProviderSectionFlavor::Wallet => KeyProviderType::Ledger,
        KeyProviderSectionFlavor::Heir | KeyProviderSectionFlavor::HeirWallet => {
            KeyProviderType::Local
        }
    });

    let local_key_creation_config_state: LocalKeyCreationConfigState = use_signal(|| Err(()));

    // Internal validation

    // Update parent signal when internal state changes
    use_effect(move || {
        let result = match key_provider_type() {
            KeyProviderType::None => Ok(KeyProviderConfig::None),
            KeyProviderType::Local => {
                local_key_creation_config_state().map(KeyProviderConfig::Local)
            }
            KeyProviderType::Ledger => {
                if state_management::ledger_is_ready().is_some() {
                    Ok(KeyProviderConfig::Ledger)
                } else {
                    Err(())
                }
            }
        };
        key_provider_config_state.set(result);
    });

    let key_provider_is_local =
        use_memo(move || matches!(key_provider_type(), KeyProviderType::Local));

    use_drop(|| log::debug!("KeyProviderSection Dropped"));

    rsx! {
        div { class: "card [--cardtitle-fs:var(--text-2xl)] border border-base-content/5 shadow-md",
            div { class: "card-body",
                h2 { class: "card-title", "Key Provider" }
                div { class: "card-subtitle",
                    match flavor {
                        KeyProviderSectionFlavor::Wallet => {
                            "The key provider manages your private keys and handles transaction signing. Using a hardware wallet device is recommended for security."
                        }
                        KeyProviderSectionFlavor::Heir => {
                            "The key provider manages private keys and can generate the Heir Configuration."
                        }
                        KeyProviderSectionFlavor::HeirWallet => {
                            "The key provider manages your private keys and handles transaction signing."
                        }
                    }
                }

                Divider { "Key Provider Type" }

                RadioChoices { count: 3,
                    RadioChoice {
                        name: "key_provider",
                        state: key_provider_type,
                        value: KeyProviderType::None,
                        title: match flavor {
                            KeyProviderSectionFlavor::Wallet | KeyProviderSectionFlavor::HeirWallet => {
                                "None (Watch-Only)"
                            }
                            KeyProviderSectionFlavor::Heir => "None",
                        },
                        subtitle: match flavor {
                            KeyProviderSectionFlavor::Wallet | KeyProviderSectionFlavor::HeirWallet => {
                                "No signing capability; can only watch blockchain and create unsigned transactions"
                            }
                            KeyProviderSectionFlavor::Heir => {
                                "No generation capability; you provide the Heir Configuration yourself."
                            }
                        },
                    }

                    MaybeHighlight {
                        step: OnboardingStep::SelectLocalKeyStorage,
                        progress: MaybeHighlightProgressType::Signal(key_provider_is_local.into()),
                        RadioChoice {
                            name: "key_provider",
                            state: key_provider_type,
                            value: KeyProviderType::Local,
                            title: "Local Key Storage",
                            subtitle: match flavor {
                                KeyProviderSectionFlavor::Wallet => {
                                    "Store private keys locally (use with caution; password protection recommended)"
                                }
                                KeyProviderSectionFlavor::HeirWallet => "Store private keys locally",
                                KeyProviderSectionFlavor::Heir => "Generate the Heir Configuration locally",
                            },
                        }
                    }

                    RadioChoice {
                        name: "key_provider",
                        state: key_provider_type,
                        value: KeyProviderType::Ledger,
                        title: "Ledger Device",
                        subtitle: match flavor {
                            KeyProviderSectionFlavor::Wallet => {
                                "Use a Ledger hardware wallet device for secure key storage and signing (recommended)"
                            }
                            KeyProviderSectionFlavor::Heir | KeyProviderSectionFlavor::HeirWallet => {
                                "Not supported for heirs and heir wallets yet"
                            }
                        },
                        disabled: matches!(
                            flavor,
                            KeyProviderSectionFlavor::Heir | KeyProviderSectionFlavor::HeirWallet
                        ),
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

                // Key provider specific options
                match key_provider_type() {
                    KeyProviderType::Ledger => rsx! {
                        LedgerKeyOptions {}
                    },
                    KeyProviderType::Local => rsx! {
                        LocalKeyOptions { local_key_creation_config_state }
                    },
                    KeyProviderType::None => rsx! {},
                }
            }
        }
    }
}

/// Ledger key provider options
#[component]
fn LedgerKeyOptions() -> Element {
    log::debug!("LedgerKeyOptions Rendered");

    let ledger_error = use_memo(move || {
        if !state_management::ledger_is_ready().is_some() {
            Some("Connect your Ledger via USB with the Bitcoin application opened")
        } else {
            None
        }
    });
    let ledger_connected = use_memo(move || state_management::ledger_is_ready().is_some());

    use_drop(|| log::debug!("LedgerKeyOptions Dropped"));

    rsx! {
        Divider { "Ledger Status" }


        MaybeHighlight {
            step: OnboardingStep::EnsureLedgerIsConnected,
            progress: MaybeHighlightProgressType::Signal(ledger_connected.into()),
            LedgerServiceStatusWithDesc { class: "text-lg flex justify-center items-center gap-8" }
        }
        div {
            class: "fieldset-label text-error justify-center",
            class: if ledger_error().is_none() { "invisible" },
            if let Some(e) = ledger_error() {
                {e}
            } else {
                "ph"
            }
        }
    }
}

/// Local key provider options
#[component]
fn LocalKeyOptions(local_key_creation_config_state: LocalKeyCreationConfigState) -> Element {
    log::debug!("LocalKeyOptions Rendered");

    let flavor = consume_context::<KeyProviderSectionFlavor>();

    let password_disclamer = match flavor {
        KeyProviderSectionFlavor::Wallet => {
            "No copy of it will be stored by the Heritage wallet application or the service, \
            and it is strictly impossible to spend Bitcoin with your wallet without it. \
            If you forget your password, the only way to retrieve your Bitcoin will be to inherit \
            them. For this reason, it is advised that the first heir of your wallet is a backup \
            wallet that you own."
        }
        KeyProviderSectionFlavor::Heir => {
            "No copy of it will be stored by the Heritage wallet application or the service, \
            and it is strictly impossible to create an heir wallet and inherit Bitcoin without \
            it. If your heir forget their password, they will be unable to spend their \
            inheritance forever."
        }
        KeyProviderSectionFlavor::HeirWallet => {
            "No copy of it will be stored by the Heritage wallet application or the service, \
            and it is strictly impossible to inherit Bitcoin with your heir wallet without it. \
            If you forget your password, your inheritance will be lost forever."
        }
    };

    // Internal state - not exposed to parent
    let local_key_creation = use_signal(|| match flavor {
        KeyProviderSectionFlavor::Wallet | KeyProviderSectionFlavor::Heir => LocalKeyCreation::New,
        KeyProviderSectionFlavor::HeirWallet => LocalKeyCreation::Restore,
    });
    let mut word_count = use_signal(|| match flavor {
        KeyProviderSectionFlavor::Wallet => 24,
        KeyProviderSectionFlavor::Heir | KeyProviderSectionFlavor::HeirWallet => 12,
    });

    let mut use_password = use_signal(|| match flavor {
        KeyProviderSectionFlavor::Wallet => true,
        KeyProviderSectionFlavor::Heir | KeyProviderSectionFlavor::HeirWallet => false,
    });
    let password = use_signal(String::new);
    let password_confirm = use_signal(String::new);

    let mnemo_state: Signal<Result<Mnemonic, ()>> = use_signal(|| Err(()));

    // Internal validation
    let password_provided = use_memo(move || {
        if use_password() {
            !password().is_empty()
        } else {
            true
        }
    });
    let passwords_match = use_memo(move || {
        if use_password() && local_key_creation() == LocalKeyCreation::New {
            password_provided() && password() == password_confirm()
        } else {
            true
        }
    });
    let passwords_error = use_memo(move || {
        if !password_provided() {
            Some(CCStr::from("Provide a password"))
        } else if !passwords_match() {
            Some(CCStr::from("Passwords do not match"))
        } else {
            None
        }
    });

    // Update parent signal when internal state changes
    use_effect(move || {
        let result = match local_key_creation() {
            LocalKeyCreation::New => {
                if passwords_match() {
                    Ok(LocalKeyCreationConfig::New {
                        word_count: word_count(),
                        password: use_password().then(move || password()),
                    })
                } else {
                    Err(())
                }
            }
            LocalKeyCreation::Restore => {
                if password_provided() {
                    mnemo_state().map(|mnemo| LocalKeyCreationConfig::Restore {
                        mnemo,
                        password: use_password().then(move || password()),
                    })
                } else {
                    Err(())
                }
            }
        };
        local_key_creation_config_state.set(result);
    });

    let ob_passwords_provided = use_memo(move || {
        use_password() && password_provided() && *password.read() == *password_confirm.read()
    });

    let local_key_creation_is_restore =
        use_memo(move || matches!(local_key_creation(), LocalKeyCreation::Restore));

    use_drop(|| log::debug!("LocalKeyOptions Dropped"));

    rsx! {
        Divider { "Local Key Provider Creation Options" }

        div { class: "flex flex-col gap-4",
            // Mode selection
            RadioChoices { count: 2,
                RadioChoice {
                    name: "local_key_mode",
                    state: local_key_creation,
                    value: LocalKeyCreation::New,
                    title: "New",
                    subtitle: "Generate new seed mnemonic words",
                }

                MaybeHighlight {
                    step: OnboardingStep::SelectRestoreSeed,
                    progress: MaybeHighlightProgressType::Signal(local_key_creation_is_restore.into()),
                    RadioChoice {
                        name: "local_key_mode",
                        state: local_key_creation,
                        value: LocalKeyCreation::Restore,
                        title: "Restore",
                        subtitle: "Enter existing seed mnemonic words",
                    }
                }
            }


            fieldset { class: "fieldset",
                legend { class: "fieldset-legend", "Word Count" }

                div { class: "join",
                    select {
                        class: "select select-sm w-20 join-item",
                        value: "{word_count()}",
                        onchange: move |evt| {
                            if let Ok(count) = evt.parsed::<usize>() {
                                word_count.set(count);
                            }
                        },
                        option { value: "12", "12" }
                        option { value: "18", "18" }
                        option { value: "24", "24" }
                    }
                    span { class: "fieldset-label join-item border border-base-content/20 px-1",
                        "words"
                    }
                }
            }

            // Mode-specific options
            match local_key_creation() {
                LocalKeyCreation::New => rsx! {},
                LocalKeyCreation::Restore => rsx! {
                    LocalKeyModeRestore { mnemo_state, word_count }
                },
            }

            // Password options
            label { class: "label",
                input {
                    r#type: "checkbox",
                    class: "toggle toggle-secondary",
                    checked: use_password(),
                    onchange: move |evt| use_password.set(evt.checked()),
                }
                span { class: "text-base ml-2",
                    if use_password() {
                        match (local_key_creation(), flavor) {
                            (LocalKeyCreation::New, KeyProviderSectionFlavor::Wallet)
                            | (LocalKeyCreation::New, KeyProviderSectionFlavor::HeirWallet) => {
                                "Use password protection (recommended)"
                            }
                            (LocalKeyCreation::New, KeyProviderSectionFlavor::Heir) => {
                                "Use password protection"
                            }
                            (LocalKeyCreation::Restore, _) => "My seed is password protected",
                        }
                    } else {
                        match (local_key_creation(), flavor) {
                            (LocalKeyCreation::New, KeyProviderSectionFlavor::Wallet)
                            | (LocalKeyCreation::New, KeyProviderSectionFlavor::HeirWallet) => {
                                "Do not use password protection (unsafe)"
                            }
                            (LocalKeyCreation::New, KeyProviderSectionFlavor::Heir) => {
                                "Do not use password protection (recommended)"
                            }
                            (LocalKeyCreation::Restore, _) => "My seed is not password protected",
                        }
                    }
                }
            }

            // Password input
            if use_password() {
                MaybeHighlight {
                    step: OnboardingStep::InputTheSeedPassword,
                    progress: MaybeHighlightProgressType::Signal(ob_passwords_provided.into()),
                    div { class: "grid grid-cols-1 sm:grid-cols-2 gap-x-4",
                        if local_key_creation() == LocalKeyCreation::New {
                            match flavor {
                                KeyProviderSectionFlavor::Wallet | KeyProviderSectionFlavor::HeirWallet => {
                                    rsx! {
                                        div { class: "col-span-full text-warning",
                                            "Make sure you "
                                            span { class: "font-black", "do not forget" }
                                            " your password."
                                        }
                                    }
                                }
                                KeyProviderSectionFlavor::Heir => rsx! {
                                    div { class: "col-span-full text-warning",
                                        "Make sure your heir "
                                        span { class: "font-black", "do not forget" }
                                        " their password."
                                    }
                                },
                            }
                            div { class: "col-span-full", {password_disclamer} }
                        }

                        InputField {
                            title: "Password",
                            value: password,
                            r#type: "password",
                            placeholder: "Enter password for seed protection",
                            value_error: passwords_error,
                        }
                        if local_key_creation() == LocalKeyCreation::New {
                            // New seed - require password confirmation
                            InputField {
                                title: "Confirm Password",
                                value: password_confirm,
                                r#type: "password",
                                placeholder: "Re-enter password",
                                value_error: passwords_error,
                            }
                        }
                    }
                }
            }
        }
    }
}

/// Local key mode options for seed restoration
#[component]
fn LocalKeyModeRestore(
    mnemo_state: Signal<Result<Mnemonic, ()>>,
    word_count: Signal<usize>,
) -> Element {
    log::debug!("LocalKeyModeRestore Rendered");

    let future_fingerprints = try_use_context::<Memo<FutureFingerprints>>();
    let key_provider_fingerprint = use_context::<Memo<Option<Fingerprint>>>();

    // Internal state - not exposed to parent
    let mut seed_words = use_signal(|| vec![String::new(); word_count()]);
    use_effect(move || {
        let word_count = word_count();
        if seed_words.peek().len() != word_count {
            seed_words.write().resize_with(word_count, String::new);
        }
    });

    // Internal validation

    let restore_seed_provided = use_memo(move || seed_words().iter().all(|w| !w.trim().is_empty()));
    let restored_mnemonic = use_memo(move || {
        if restore_seed_provided() {
            Some(
                Mnemonic::parse_in(Language::English, seed_words.read().join(" "))
                    .map_err(log_error_ccstr),
            )
        } else {
            None
        }
    });

    let seed_error = use_memo(move || {
        if restored_mnemonic.read().is_none() {
            Some(CCStr::from("Provide all the seed mnemonic words"))
        } else if let Some(Err(ref e)) = *restored_mnemonic.read() {
            Some(e.clone())
        } else if future_fingerprints
            .is_some_and(|future_fingerprints| !future_fingerprints().coherents())
        {
            Some(CCStr::from(
                "The resulting seed is not compatible with the online wallet.",
            ))
        } else {
            None
        }
    });

    // Update parent signal when internal state changes
    use_effect(move || {
        let result = if let Some(Ok(mnemo)) = restored_mnemonic() {
            Ok(mnemo)
        } else {
            Err(())
        };
        mnemo_state.set(result);
    });

    let (error_display, mut signal_activity, onfocusout) =
        use_future_error_feedback(seed_error.into());

    let mnemo_state_ok = use_memo(move || mnemo_state.read().is_ok());

    use_drop(|| log::debug!("LocalKeyModeRestore Dropped"));

    rsx! {
        fieldset { class: "fieldset",
            legend { class: "fieldset-legend", "Seed Mnemonic Words" }
            div { class: "fieldset-description",
                "Enter your existing seed mnemonic words to restore the wallet."
            }

            MaybeHighlight {
                step: OnboardingStep::RestoreKeyProviderSeed,
                progress: MaybeHighlightProgressType::Signal(mnemo_state_ok.into()),
                div { class: "grid grid-cols-2 sm:grid-cols-3 md:grid-cols-4 lg:grid-cols-6 gap-2",
                    for (i , word) in seed_words().iter().enumerate().take(word_count() as usize) {
                        input {
                            r#type: "text",
                            class: "input input-sm text-center",
                            class: if key_provider_fingerprint().is_some() { "input-success" },
                            placeholder: "{i + 1}",
                            value: "{word}",
                            oninput: move |evt| {
                                signal_activity();
                                let words = &mut seed_words.write();
                                let new_words = evt
                                    .value()
                                    .split(" ")
                                    .filter_map(|w| (!w.is_empty()).then(|| w.to_owned()))
                                    .collect::<Vec<_>>();
                                if new_words.is_empty() {
                                    words[i] = String::new();
                                } else {
                                    let new_words_count = new_words.len();
                                    let words_count = words.len();
                                    let (base_index, max_new_words) = if i + new_words_count > words_count {
                                        if new_words_count <= words_count {
                                            (words_count - new_words_count, new_words_count)
                                        } else {
                                            (0, words_count)
                                        }
                                    } else {
                                        (i, new_words_count)
                                    };
                                    for (j, new_word) in new_words.into_iter().take(max_new_words).enumerate() {
                                        words[base_index + j] = new_word;
                                    }
                                }
                            },
                            onfocusout,
                        }
                    }
                }
            }
            div {
                class: "fieldset-label",
                class: if error_display().is_none() && key_provider_fingerprint().is_none() { "invisible" },
                class: if error_display().is_some() { "text-error" },
                class: if key_provider_fingerprint().is_some() { "text-success" },
                if let Some(fg) = key_provider_fingerprint() {
                    "Valid mnemonic words for a wallet seed with fingerprint "
                    span { class: "font-bold", "{fg}" }
                } else if let Some(e) = error_display() {
                    {e}
                } else {
                    "ph"
                }
            }
        }
    }
}
