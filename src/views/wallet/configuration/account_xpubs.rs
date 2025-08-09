use crate::prelude::*;

use btc_heritage_wallet::{
    btc_heritage::AccountXPub, heritage_service_api_client::AccountXPubWithStatus,
    miniscript::DescriptorPublicKey, AnyOnlineWallet, BoundFingerprint, KeyProvider, OnlineWallet,
    Wallet,
};

use crate::{
    components::{
        badge::{
            ExternalDependencyStatus, KeyProviderType, OnlineWalletType, UIBadge, UIBadgeStyle,
        },
        modal::{ConfigModal, InfoModal},
        svg::{Cancel, DrawSvg},
    },
    utils::{CCStr, CheapClone},
};

#[derive(Debug, Clone, Copy, PartialEq)]
struct UIXPubStatusBadge(UIBadge);
impl LoadedElement for UIXPubStatusBadge {
    type Loader = SkeletonLoader;
    #[inline(always)]
    fn element<M: LoadedComponentInputMapper>(self, m: M) -> Element {
        self.0.element(m)
    }

    fn place_holder() -> Self {
        Self(UIBadge::place_holder())
    }
}
impl FromRef<AccountXPubWithStatus> for UIXPubStatusBadge {
    fn from_ref(status: &AccountXPubWithStatus) -> Self {
        Self(match status {
            AccountXPubWithStatus::Used(_) => UIBadge {
                text: "Used",
                badge_style: UIBadgeStyle::Custom("badge-success"),
                tooltip: "This XPub has been consumed",
                status: ExternalDependencyStatus::None,
            },

            AccountXPubWithStatus::Unused(_) => UIBadge {
                text: "Unused",
                badge_style: UIBadgeStyle::Custom("badge-warning"),
                tooltip: "This XPub is available for future use",
                status: ExternalDependencyStatus::None,
            },
        })
    }
}

#[derive(Debug, Clone, PartialEq)]
struct UIXPubRow {
    badge: UIXPubStatusBadge,
    descriptor: CCStr,
}
impl LoadedElement for UIXPubRow {
    type Loader = TransparentLoader;
    #[inline(always)]
    fn element<M: LoadedComponentInputMapper>(self, m: M) -> Element {
        rsx! {
            tr { key: "{self.descriptor}",
                td {
                    LoadedComponent { input: m.map(self.badge) }
                }
                td {
                    div { class: "font-mono text-sm",
                        LoadedComponent { input: m.map(self.descriptor.clone()) }
                    }
                }
            
            }
        }
    }
    fn place_holder() -> Self {
        Self {
            badge: UIXPubStatusBadge::place_holder(),
            descriptor: CCStr::place_holder(),
        }
    }
}
impl LoadedSuccessConversionMarker for TypeCouple<AccountXPubWithStatus, UIXPubRow> {}
impl FromRef<AccountXPubWithStatus> for UIXPubRow {
    fn from_ref(status: &AccountXPubWithStatus) -> Self {
        let descriptor = match status {
            AccountXPubWithStatus::Used(account_xpub)
            | AccountXPubWithStatus::Unused(account_xpub) => CCStr::from(account_xpub.to_string()),
        };
        let badge = UIXPubStatusBadge::from_ref(status);
        Self { badge, descriptor }
    }
}

/// Component to configure and manage Account eXtended Public Keys.
///
/// Displays a table with the status and value of each account extended public key,
/// and provides buttons to add and auto-feed more keys.
///
/// # Examples
///
/// ```
/// rsx! {
///     AccountXPubConfig {}
/// }
/// ```
#[component]
pub(super) fn AccountXPubConfig() -> Element {
    log::debug!("AccountXPubConfig Rendered");

    let service_client_service = state_management::use_service_client_service();

    let mut wallet = use_context::<AsyncSignal<Wallet>>();
    let keyprovider_status =
        use_context::<Memo<Option<(KeyProviderType, ExternalDependencyStatus)>>>();
    let online_status = use_context::<Memo<Option<(OnlineWalletType, ExternalDependencyStatus)>>>();

    let account_xpubs = helper_hooks::use_resource_wallet_account_xpubs(wallet);

    let could_generate = use_memo(move || match keyprovider_status() {
        Some((KeyProviderType::None, _)) => false,
        None => false,
        _ => true,
    });
    let could_feed = use_memo(move || match online_status() {
        Some((OnlineWalletType::None, _)) => false,
        None => false,
        _ => true,
    });
    let could_auto_feed = use_memo(move || could_generate() && could_feed());
    let can_generate = use_memo(move || match keyprovider_status() {
        Some((KeyProviderType::LocalKey, ExternalDependencyStatus::Available)) => true,
        Some((KeyProviderType::Ledger, ExternalDependencyStatus::Available)) => true,
        Some((KeyProviderType::Ledger, ExternalDependencyStatus::NeedUserAction)) => true,
        _ => false,
    });
    let can_feed = use_memo(move || match online_status() {
        Some((OnlineWalletType::Service, ExternalDependencyStatus::Available)) => true,
        Some((OnlineWalletType::Local, _)) => true,
        _ => false,
    });
    let can_auto_feed = use_memo(move || can_generate() && can_feed());

    let mut in_operation = use_signal(|| false);
    let mut operation_progress = use_signal(String::new);
    let mut add_xpubs_modal = use_signal(|| false);
    let mut new_xpubs_raw_text = use_signal(String::new);
    let new_xpubs = use_memo(move || {
        new_xpubs_raw_text
            .read()
            .split(['\n', ' '])
            .filter(|xpub_txt| !xpub_txt.is_empty())
            .map(|xpub_txt| {
                Ok(AccountXPub::try_from(
                    xpub_txt
                        .parse::<DescriptorPublicKey>()
                        .map_err(|e| e.to_string())?,
                )
                .map_err(|e| e.to_string())?)
            })
            .collect::<Result<Vec<_>, String>>()
    });
    let new_xpubs_count = use_memo(move || {
        if let Ok(ref v) = new_xpubs() {
            v.len()
        } else {
            0
        }
    });

    let last_seen_index = use_memo(move || {
        account_xpubs
            .lrmap_ok(|account_xpubs| {
                account_xpubs.iter().fold(None, |lsi, axpub| match axpub {
                    AccountXPubWithStatus::Used(axpub) => {
                        core::cmp::max(lsi, Some(axpub.descriptor_id()))
                    }
                    AccountXPubWithStatus::Unused(axpub) => {
                        core::cmp::max(lsi, Some(axpub.descriptor_id()))
                    }
                })
            })
            .flatten()
    });

    // Configuration state
    let mut config_expanded = use_signal(|| false);

    let mut auto_feed_count = use_signal(|| 20);
    let mut start_index = use_signal(|| 0);
    let end_index = use_memo(move || (start_index() + auto_feed_count()) as u32);
    use_effect(move || {
        *start_index.write() = last_seen_index().map(|lsi| lsi + 1).unwrap_or_default();
    });

    // Generate XPubs modal state
    let mut generate_xpubs_modal = use_signal(|| false);
    let mut generating = use_signal(|| false);
    let mut generated_xpubs = use_signal(|| String::new());

    let mut feed_xpubs = async move |account_xpubs: Vec<AccountXPub>| {
        *operation_progress.write() = "Feeding Account XPubs to wallet...".to_owned();

        let wallet_online_no_fingerprint = wallet
            .with_peek(async |wallet| wallet.online_wallet().fingerprint().is_err())
            .await;
        match wallet
            .with_mut(async |wallet| wallet.feed_account_xpubs(account_xpubs).await)
            .await
        {
            Ok(_) => {
                log::info!("Successfully auto-fed new Account XPubs to the wallet");
                alert_success("New Account XPubs auto-fed");
                if wallet_online_no_fingerprint {
                    *operation_progress.write() = "Updating service wallet binding...".to_owned();
                    wallet
                        .with_peek(async move |wallet| {
                            if let AnyOnlineWallet::Service(service_binding) =
                                wallet.online_wallet()
                            {
                                state_management::inject_serviceable_wallet(
                                    service_client_service,
                                    service_binding.wallet_id().to_owned(),
                                    service_binding.fingerprint().ok(),
                                )
                            }
                        })
                        .await;
                }

                // Close the modal
                *add_xpubs_modal.write() = false;
            }
            Err(e) => {
                log::error!("Failed to feed new Account XPubs to the wallet: {e}");
                alert_error(format!(
                    "Failed to feed new Account XPubs to the wallet: {e}"
                ));
            }
        };
    };
    // Function to handle form submission
    let submit_xpubs = move |_| async move {
        log::debug!("Manual XPub submission started");
        // Set creating state to true to show loading UI
        *in_operation.write() = true;
        *operation_progress.write() = "Preparing Account XPubs...".to_owned();

        let xpubs = new_xpubs().expect("can only be here if Result::Ok");
        log::debug!("Submitting {} manually entered XPubs", xpubs.len());

        feed_xpubs(xpubs).await;

        *in_operation.write() = false;
        *operation_progress.write() = String::new();
        log::debug!("Manual XPub submission completed");
    };

    let mut generate_xpubs = async move || {
        log::debug!("XPub generation started");
        *generating.write() = true;
        *generated_xpubs.write() = String::new();

        let start = start_index() as u32;
        let end = end_index();
        let count = end - start;

        log::info!(
            "Generating {} XPubs (indices {} to {})",
            count,
            start,
            end - 1
        );

        let account_xpubs = match wallet
            .with(async |wallet| wallet.derive_accounts_xpubs(start..end).await)
            .await
        {
            Ok(account_xpubs) => account_xpubs,
            Err(e) => {
                log::error!("Failed to generate Account XPubs: {e}");
                alert_error(format!("Failed to generate Account XPubs: {e}"));
                *generating.write() = false;
                return;
            }
        };

        let xpub_text = account_xpubs
            .iter()
            .map(|xpub| xpub.to_string())
            .collect::<Vec<_>>()
            .join("\n");

        *generated_xpubs.write() = xpub_text;
        *generating.write() = false;
        log::info!(
            "Successfully generated {} Account XPubs",
            account_xpubs.len()
        );
    };

    let auto_feed_xpubs = move |_| async move {
        log::debug!("Auto-feed XPubs operation started");
        *in_operation.write() = true;
        *operation_progress.write() = "Starting auto-feed operation...".to_owned();

        log::info!("Auto-feed XPubs clicked");

        let start = start_index() as u32;
        let end = end_index();
        let count = end - start;

        *operation_progress.write() = format!(
            "Generating {} Account XPubs (indices {} to {})...",
            count,
            start,
            end - 1
        );

        let account_xpubs = match wallet
            .with(async |wallet| wallet.derive_accounts_xpubs(start..end).await)
            .await
        {
            Ok(account_xpubs) => account_xpubs,
            Err(e) => {
                log::error!("Failed to generate new Account XPubs: {e}");
                alert_error(format!("Failed to generate new Account XPubs: {e}"));
                *in_operation.write() = false;
                *operation_progress.write() = String::new();
                log::debug!("Auto-feed XPubs operation completed");
                return;
            }
        };

        feed_xpubs(account_xpubs).await;

        *in_operation.write() = false;
        *operation_progress.write() = String::new();
        log::debug!("Auto-feed XPubs operation completed");
    };

    use_drop(|| log::debug!("AccountXPubConfig Dropped"));

    rsx! {
        div { class: "rounded-box border border-base-content/5 shadow-md p-4 my-4",
            h2 { class: "text-2xl font-bold mb-4", "Account Extended Public Keys" }

            div { class: "text-sm font-light mb-6",
                "Account Extended Public Keys (XPubs) are the foundation of your Heritage wallet's security and functionality. "
                "Each XPub allows the wallet to generate a virtually unlimited number of unique Bitcoin addresses without exposing your private keys. "
                "When you create or update a Heritage Configuration, the wallet consumes one XPub to generate the specific Bitcoin descriptors needed for that configuration. "
                "Once an XPub is \"Used\" for a Heritage Configuration, it cannot be reused—this ensures each configuration has its own unique set of addresses. "

                if could_generate() && could_feed() {
                    "It's recommended to maintain a healthy supply of \"Unused\" XPubs so you can easily update your inheritance settings without interruption. "
                    "The \"Auto-feed\" feature can automatically generate and feed more XPubs. "
                    "You can also generate XPubs offline for manual feeding later."
                } else if could_generate() {
                    "You can generate XPubs using your Key Provider, then manually feed them to the Online Wallet on another device."
                } else if could_feed() {
                    "You can manually add XPubs to the wallet. Generate them using your Key Provider on another device."
                } else {
                    "Both a Key Provider and an Online Wallet are needed to manage XPubs effectively."
                }
            }

            if could_feed() {
                // Table of XPubs
                div { class: "h-96 overflow-x-auto bg-base-100 rounded-lg shadow mb-6",
                    table { class: "table table-pin-rows w-full",
                        thead {
                            tr {
                                th { class: "w-1/6", "Status" }
                                th { "Descriptor" }
                            }
                        }
                        tbody {
                            LoadedComponent::<CheapClone<[UIXPubRow]>> { input: account_xpubs.into() }
                        }
                    }
                }
            }

            // Action buttons
            div { class: "flex gap-4 mb-6",

                if could_feed() {
                    button {
                        class: "btn btn-primary",
                        disabled: !can_feed() || in_operation(),
                        onclick: move |_| {
                            *add_xpubs_modal.write() = true;
                            *new_xpubs_raw_text.write() = String::new();
                        },
                        "Add XPubs"
                    }
                }

                if could_auto_feed() {
                    button {
                        class: "btn btn-secondary",
                        disabled: !can_auto_feed() || in_operation(),
                        onclick: auto_feed_xpubs,
                        if in_operation() {
                            span { class: "loading loading-spinner loading-sm mr-2" }
                            "Auto-feeding..."
                        } else {
                            "Auto-feed {start_index()} to {end_index() - 1}"
                        }
                    }
                } else if could_generate() {
                    button {
                        class: "btn btn-primary",
                        disabled: !can_generate() || in_operation(),
                        onclick: move |_| async move {
                            *generate_xpubs_modal.write() = true;
                            *generated_xpubs.write() = String::new();
                            generate_xpubs().await;
                        },
                        "Generate XPubs {start_index()} to {end_index() - 1}"
                    }
                }
            }

            // Configuration section
            if could_generate() {
                div { class: "collapse collapse-arrow border border-base-content/10 rounded-box",
                    input {
                        r#type: "checkbox",
                        checked: config_expanded(),
                        disabled: in_operation(),
                        onchange: move |evt| *config_expanded.write() = evt.checked(),
                    }
                    div { class: "collapse-title text-base font-medium", "XPubs Generation Config" }
                    div { class: "collapse-content",
                        div { class: "grid grid-cols-1 md:grid-cols-2 gap-4",

                            if !could_auto_feed() {
                                fieldset { class: "fieldset",
                                    legend { class: "fieldset-legend text-sm", "Start index" }
                                    div { class: "fieldset-description",
                                        "Starting index for generation."
                                    }
                                    label { class: "input input-sm w-60",
                                        input {
                                            r#type: "number",
                                            min: "0",
                                            max: "100",
                                            value: start_index(),
                                            disabled: in_operation(),
                                            oninput: move |evt| {
                                                if let Ok(value) = evt.value().parse::<u32>() {
                                                    start_index.set(value.clamp(0, 100));
                                                } else {
                                                    start_index.set(0);
                                                }
                                            },
                                        }
                                    }
                                    div { class: "fieldset-label", "Min: 0, Max: 100, Default: 0" }
                                }
                            }
                            fieldset { class: "fieldset",
                                legend { class: "fieldset-legend text-sm", "Number of keys" }
                                div { class: "fieldset-description",
                                    "Number of account extended public keys to "
                                    if could_auto_feed() {
                                        "auto-feed."
                                    } else {
                                        "generate."
                                    }
                                }
                                label { class: "input input-sm w-60",
                                    input {
                                        r#type: "number",
                                        min: "1",
                                        max: "100",
                                        value: auto_feed_count(),
                                        disabled: in_operation(),
                                        oninput: move |evt| {
                                            if let Ok(value) = evt.value().parse::<u32>() {
                                                auto_feed_count.set(value.clamp(1, 100));
                                            } else {
                                                auto_feed_count.set(20);
                                            }
                                        },
                                    }
                                    span { class: "label", "keys" }
                                }
                                div { class: "fieldset-label", "Min: 1, Max: 100, Default: 20" }
                            }
                        }
                        // Generate XPubs modal
                        ConfigModal {
                            is_open: generate_xpubs_modal,
                            title: "Generate Account eXtended Public Keys",
                            div { class: "mt-4 w-full max-w-4xl",
                                if generating() {
                                    div { class: "text-center py-8",
                                        div { class: "loading loading-spinner loading-lg mb-4" }
                                        div { class: "text-lg font-medium",
                                            "Generating {auto_feed_count()} XPubs (indices {start_index()} to {end_index() - 1})..."
                                        }
                                        div { class: "text-sm text-base-content/70",
                                            "This may take a while, especially with hardware wallets. Please be patient and do not disconnect your device."
                                        }
                                    }
                                } else {
                                    div { class: "text-sm font-light mb-4",
                                        "Successfully generated {auto_feed_count()} Account eXtended Public Keys (indices {start_index()} to {end_index() - 1}). "
                                        "You can copy these XPubs and feed them to the wallet later using the \"Add XPubs\" button."
                                    }
                                    div {
                                        role: "fieldset",
                                        class: "fieldset w-full",
                                        legend { class: "fieldset-legend text-lg", "Generated XPubs" }
                                        textarea {
                                            class: "textarea w-full h-48 font-mono text-sm",
                                            readonly: true,
                                            value: generated_xpubs(),
                                        }
                                    }
                                    div { class: "flex justify-center mt-4",
                                        button {
                                            class: "btn btn-outline",
                                            onclick: move |_| *generate_xpubs_modal.write() = false,
                                            DrawSvg::<Cancel> {}
                                            "Close"
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }

            // Add XPubs modal
            ConfigModal {
                is_open: add_xpubs_modal,
                title: "Add Account eXtended Public Keys",
                div { class: "mt-4 w-full max-w-4xl",
                    div { class: "text-sm font-light",
                        "Account eXtended Public Keys are the fuel of the Heritage wallet. \
                           Each time you change or renew your Heritage Configuration, the service needs \
                           to generate new Bitcoin descriptors that are in turn used to create your \
                           Bitcoin addresses. It is recommended to provide multiple Account eXtended \
                           Public Keys so you don't have to add more frequently."
                    }
                    div { role: "fieldset", class: "fieldset w-full",
                        legend { class: "fieldset-legend text-lg",
                            "Add your Account eXtended Public Keys, separated by spaces or new lines:"
                        }
                        textarea {
                            class: "textarea w-full h-24",
                            class: if let Err(_) = new_xpubs() { "textarea-error" },
                            placeholder: "[73c5da0a/86'/0'/0']xpubDEKGYxthPqkm...HdbWQQKuAd/*\n[73c5da0a/86'/0'/1']xpubDEKGYxthPqkm...5iAYoDDScN/*",
                            disabled: in_operation(),
                            oninput: move |evt| { *new_xpubs_raw_text.write() = evt.value() },
                        }
                        match new_xpubs() {
                            Ok(_) => rsx! {
                                div { class: "label invisible", "Nothing to display" }
                            },
                            Err(e) => rsx! {
                                div { class: "label text-error", {e} }
                            },
                        }
                    }
                    // Action buttons
                    div { class: "flex gap-4 mt-4 justify-center",
                        button {
                            class: "btn btn-primary",
                            onclick: submit_xpubs,
                            disabled: new_xpubs_count() == 0 || in_operation(),
                            if in_operation() {
                                span { class: "loading loading-spinner loading-sm mr-2" }
                                "Adding XPubs..."
                            } else {
                                "Add {new_xpubs_count()} XPubs"
                            }
                        }
                        button {
                            class: "btn btn-outline btn-primary",
                            disabled: in_operation(),
                            onclick: move |_| {
                                if !in_operation() {
                                    *add_xpubs_modal.write() = false;
                                }
                            },
                            DrawSvg::<Cancel> {}
                            "Cancel"
                        }
                    }
                }
            }

            // Progress modal for operations
            InfoModal { is_open: in_operation, title: "Processing Account XPubs",
                div { class: "flex flex-col items-center gap-4 p-6",
                    div { class: "flex flex-row justify-center gap-4",
                        span { class: "loading loading-spinner loading-lg" }
                        div { class: "text-lg font-semibold", "{operation_progress}" }
                    }
                    div { class: "text-sm text-base-content/70 text-center max-w-md",
                        "This operation may take some time, especially when using hardware wallets. Please be patient and do not close this window."
                    }
                }
            }
        }
    }
}
