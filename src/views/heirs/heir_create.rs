use crate::prelude::*;

use std::collections::HashSet;

use btc_heritage_wallet::{
    btc_heritage::{utils::bitcoin_network, HeirConfig},
    heritage_service_api_client::{Heir as ServiceHeir, HeirCreate, MainContact},
    AnyKeyProvider, DatabaseItem, Heir as DbHeir, KeyProvider, LocalKey,
};

use crate::{
    components::{
        create_key_provider::{
            KeyProviderConfig, KeyProviderConfigState, KeyProviderSection,
            KeyProviderSectionFlavor, LocalKeyCreationConfig,
        },
        export_heir_to_service::{
            ExportToServiceConfig, ExportToServiceSectionForm, ExportToServiceSectionFormFlavor,
        },
        inputs::{use_future_error_feedback, InputField},
        misc::BackButton,
        svg::{AccountMultiplePlus, DrawSvg},
    },
    utils::{log_error_ccstr, CCStr, CheapClone},
    Route,
};

#[derive(Debug, Clone, Copy, PartialEq)]
enum HeirConfigType {
    SingleHeirPubkey,
    HeirXPubkey,
}
impl HeirConfigType {
    fn list() -> [Self; 2] {
        [Self::HeirXPubkey, Self::SingleHeirPubkey]
    }
    fn display(self) -> &'static str {
        match self {
            HeirConfigType::SingleHeirPubkey => "Public Key (deprecated)",
            HeirConfigType::HeirXPubkey => "Extended Public Key",
        }
    }
    fn disabled(self) -> bool {
        match self {
            HeirConfigType::SingleHeirPubkey => true,
            _ => false,
        }
    }
}
impl core::str::FromStr for HeirConfigType {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "SingleHeirPubkey" => Ok(Self::SingleHeirPubkey),
            "HeirXPubkey" => Ok(Self::HeirXPubkey),
            _ => Err(()),
        }
    }
}
impl core::fmt::Display for HeirConfigType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            HeirConfigType::SingleHeirPubkey => "SingleHeirPubkey",
            HeirConfigType::HeirXPubkey => "HeirXPubkey",
        })
    }
}

#[derive(Debug, Clone, PartialEq)]
enum HeirConfigConfig {
    Generate(HeirConfigType),
    Provided(HeirConfig),
}

/// Configuration state types
type HeirNameState = Signal<Result<String, ()>>;
type HeirConfigConfigState = Signal<Result<HeirConfigConfig, ()>>;

/// Component for creating new heirs
#[component]
pub fn HeirCreateView() -> Element {
    log::debug!("HeirCreateView Rendered");

    use_drop(|| log::debug!("HeirCreateView Dropped"));

    rsx! {
        super::super::TitledView {
            title: CCStr::from("Create Heir"),
            subtitle: CCStr::from("Define a new heir for your inheritance configurations."),
            left: rsx! {
                BackButton { route: Route::HeirListView {} }
            },
            HeirCreateForm {}

            OnboardingInfoModal { step: OnboardingStep::ModalExplainHeirKeyProvider,
                div { class: "flex flex-col gap-4 max-w-xl text-base",
                    div { class: "text-lg font-semibold", "About the Key Provider" }
                    p {
                        "When creating an heir, you may safely use a simple 12-word mnemonic
                        without a passphrase. Here's why it is secure and practical:"
                    }

                    div { class: "flex flex-col gap-3",
                        div { class: "flex gap-3",
                            div { class: "badge badge-primary badge-outline font-bold min-w-fit",
                                "1"
                            }
                            div {
                                strong { "No immediate fund access: " }
                                "The heir keys cannot access any funds until inheritances become mature. This provides a built-in security buffer."
                            }
                        }
                        div { class: "flex gap-3",
                            div { class: "badge badge-primary badge-outline font-bold min-w-fit",
                                "2"
                            }
                            div {
                                strong { "Tamper detection: " }
                                "When stored in a sealed envelope or container, it's easy to verify that no one has accessed the mnemonic.
                                If tampering is detected, you can update your Heritage Configuration to invalidate the compromised heir keys."
                            }
                        }
                        div { class: "flex gap-3",
                            div { class: "badge badge-primary badge-outline font-bold min-w-fit",
                                "3"
                            }
                            div {
                                strong { "Long-term accessibility: " }
                                "Your heirs may need to use their seed years or decades from now. Adding a passphrase increases the risk that
                                it will be forgotten (or written down alongside the mnemonic, which defeats the security purpose)."
                            }
                        }
                    }

                    p {
                        "Of course, in the end it is for you to decide. And remember you can always change your Heritage Configuration
                        later if security circumstances change or if you suspect the heir mnemonic has been compromised."
                    }
                }
            }

            OnboardingInfoModal { step: OnboardingStep::ModalExplainExportHeirToServiceOptions,
                div { class: "flex flex-col gap-4 max-w-2xl text-base",
                    p {
                        "When exporting an heir to the Heritage Service, you have several optional
                        settings to customize how your heir will be contacted and
                        what information they can access:"
                    }

                    div { class: "flex flex-col gap-3",
                        div { class: "flex gap-3",
                            div { class: "badge badge-primary badge-outline font-bold min-w-fit",
                                "1"
                            }
                            div {
                                strong { "Custom Message (recommended): " }
                                "A personal message that will be included in all
                                communications sent to your heir from the Heritage Service.
                                This is useful for providing authentication details (like
                                family code words) or final instructions on how to claim
                                their inheritance."
                            }
                        }
                        div { class: "flex gap-3",
                            div { class: "badge badge-primary badge-outline font-bold min-w-fit",
                                "2"
                            }
                            div {
                                strong { "Permissions: " }
                                "Control if your heir can see their inheritance before maturity
                                and what they can see about it. Note that once an inheritance
                                becomes mature for an heir, they can always see it with the amount
                                and maturity dates."
                            }
                        }
                        div { class: "flex gap-3",
                            div { class: "badge badge-primary badge-outline font-bold min-w-fit",
                                "3"
                            }
                            div {
                                strong { "Additional Contacts: " }
                                "Provide alternative contact methods for your heir.
                                Having multiple contact methods increases the chances your heir
                                can be reached when inheritance becomes available."
                            }
                        }
                    }

                    div { class: "alert alert-info",
                        div { class: "flex items-start gap-2",
                            "💡"
                            div { "All of these settings are optional and can be updated later." }
                        }
                    }
                }
            }
        }
    }
}

/// Main form component for creating heirs
#[component]
fn HeirCreateForm() -> Element {
    log::debug!("HeirCreateForm Rendered");

    let database_service = state_management::use_database_service();
    let service_client_service = state_management::use_service_client_service();

    let mut database_heirs = use_context::<Resource<Vec<CheapClone<DbHeir>>>>();
    let mut service_heirs = use_context::<FResource<Vec<CheapClone<ServiceHeir>>>>();

    // Form state signals - coordination pattern for better performance
    let heir_name_state: HeirNameState = use_signal(|| Err(()));
    let key_provider_config_state: KeyProviderConfigState = use_signal(|| Err(()));
    let heir_config_config_state: HeirConfigConfigState = use_signal(|| Err(()));
    let export_to_service_config_state = use_signal(|| Ok(ExportToServiceConfig::DoNotExport));

    let mut creating = use_signal(|| false);

    // Validation logic

    let expect_key_provider_heirconfig =
        use_memo(move || match &*key_provider_config_state.read() {
            Ok(KeyProviderConfig::None) => false,
            _ => true,
        });

    // Combined validation
    let form_valid = use_memo(move || {
        heir_name_state.read().is_ok()
            && key_provider_config_state.read().is_ok()
            && heir_config_config_state.read().is_ok()
            && export_to_service_config_state.read().is_ok()
    });

    // Handle form submission
    let submit_form = move |_| async move {
        log::debug!("HeirCreateForm: Form submission started");
        *creating.write() = true;

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

        let Ok(db_name) = heir_name_state() else {
            return abort("No name was provided");
        };
        log::debug!("HeirCreateForm: Heir name: {}", db_name);

        let Ok(key_provider_config) = key_provider_config_state() else {
            return abort("Invalid Key Provider configuration");
        };
        log::debug!(
            "HeirCreateForm: Key provider config: {:?}",
            key_provider_config
        );

        let Ok(heir_config_config) = heir_config_config_state() else {
            return abort("Invalid Heir Configuration configuration");
        };
        log::debug!(
            "HeirCreateForm: Heir config config: {:?}",
            heir_config_config
        );

        let Ok(export_to_service_config) = export_to_service_config_state() else {
            return abort("Invalid Export To Service configuration");
        };
        log::debug!(
            "HeirCreateForm: Export to service config: {:?}",
            export_to_service_config
        );

        let network = bitcoin_network::get();
        log::debug!("HeirCreateForm: Using network: {:?}", network);

        log::debug!("HeirCreateForm: Creating key provider");
        let key_provider = match key_provider_config {
            KeyProviderConfig::Local(local_key_creation_config) => {
                log::debug!("HeirCreateForm: Creating local key provider");
                let local_key = match local_key_creation_config {
                    LocalKeyCreationConfig::New {
                        word_count,
                        password,
                    } => {
                        log::debug!(
                            "HeirCreateForm: Generating new local key with {} words",
                            word_count
                        );
                        LocalKey::generate(word_count, password, network)
                    }
                    LocalKeyCreationConfig::Restore { mnemo, password } => {
                        log::debug!("HeirCreateForm: Restoring local key from mnemonic");
                        LocalKey::restore(mnemo, password, network)
                    }
                };
                AnyKeyProvider::LocalKey(local_key)
            }
            KeyProviderConfig::Ledger => {
                return abort("Ledger is not supported for heir creation yet");
            }
            KeyProviderConfig::None => {
                return abort("Key Provider is required");
            }
        };
        log::debug!("HeirCreateForm: Key provider created successfully");

        log::debug!("HeirCreateForm: Generating or parsing heir config");
        let heir_config = match heir_config_config {
            HeirConfigConfig::Generate(heir_config_type) => {
                log::debug!(
                    "HeirCreateForm: Generating heir config of type: {:?}",
                    heir_config_type
                );
                let hct = match heir_config_type {
                    HeirConfigType::SingleHeirPubkey => {
                        return abort("Single Public Key is deprecated")
                    }
                    HeirConfigType::HeirXPubkey => btc_heritage_wallet::HeirConfigType::HeirXPubkey,
                };
                match &key_provider {
                    AnyKeyProvider::None => {
                        return abort("Cannot generate a Heir Config without a Key Provider")
                    }
                    AnyKeyProvider::LocalKey(local_key) => {
                        log::debug!("HeirCreateForm: Deriving heir config from local key");
                        local_key.derive_heir_config(hct).await
                    }
                    AnyKeyProvider::Ledger(ledger_key) => {
                        log::debug!("HeirCreateForm: Deriving heir config from ledger key");
                        ledger_key.derive_heir_config(hct).await
                    }
                }
            }
            HeirConfigConfig::Provided(heir_config) => {
                log::debug!("HeirCreateForm: Using provided heir config");
                Ok(heir_config)
            }
        };
        let heir_config = match heir_config {
            Ok(hc) => {
                log::debug!("HeirCreateForm: Heir config generated/parsed successfully");
                hc
            }
            Err(e) => {
                return abort(&format!("Cannot generate a Heir Config: {e}"));
            }
        };

        log::debug!("HeirCreateForm: Creating heir database entry");
        let heir = DbHeir::new(db_name.clone(), heir_config.clone(), key_provider);
        match state_management::blocking_db_service_operation(database_service, move |mut db| {
            heir.create(&mut db)
        })
        .await
        {
            Ok(()) => {
                success(format!("Heir '{db_name}' created successfully"));
            }
            Err(e) => {
                return abort(&format!("Could not save the heir wallet to database: {e}"));
            }
        }

        log::debug!("HeirCreateForm: Processing service export configuration");
        let mut refresh_service_heirs = false;
        if let ExportToServiceConfig::Export {
            name,
            email,
            custom_message,
            permissions,
            additional_contacts,
        } = export_to_service_config
        {
            log::debug!("HeirCreateForm: Exporting heir to service");
            refresh_service_heirs = true;
            let client = state_management::heritage_service_client(service_client_service).await;
            log::debug!("HeirCreateForm: Got heritage service client");

            let post_heirs_result = client
                .post_heirs(HeirCreate {
                    display_name: name.unwrap_or_else(|| db_name.clone()),
                    heir_config,
                    main_contact: MainContact {
                        email,
                        custom_message,
                    },
                    permissions,
                })
                .await;
            log::debug!("HeirCreateForm: Heir post request completed");

            let service_heir_id = match post_heirs_result {
                Ok(h) => {
                    log::debug!(
                        "HeirCreateForm: Heir exported to service successfully with ID: {}",
                        h.id
                    );
                    success("Exported the in the service successfully");
                    Some(h.id)
                }
                Err(e) => {
                    warn(format!("Could not export Heir to service: {e}"));
                    None
                }
            };
            if !additional_contacts.is_empty() {
                if let Some(heir_id) = service_heir_id {
                    log::debug!(
                        "HeirCreateForm: Adding additional contacts for heir ID: {}",
                        heir_id
                    );
                    // Add contacts
                    match client
                        .post_heir_contacts(&heir_id, additional_contacts)
                        .await
                    {
                        Ok(_) => {
                            log::debug!("HeirCreateForm: Additional contacts added successfully");
                        }
                        Err(e) => {
                            warn(format!("Could not add additional contacts to the exported Heir in the service: {e}"));
                        }
                    };
                }
            }
        } else {
            log::debug!("HeirCreateForm: Skipping service export");
        };

        // Need to refresh to "insert" the newly created heir
        log::debug!("HeirCreateForm: Refreshing heir lists");
        database_heirs.restart();
        if refresh_service_heirs {
            log::debug!("service_heirs.restart()");
            service_heirs.restart();
        }

        // Add context if onboarding is in progress
        if let OnboardingStatus::InProgress(ref mut onboarding) =
            *state_management::ONBOARDING_STATUS.write()
        {
            onboarding.add_context(
                OnboardingContextItemId::HeirName.item(db_name.clone()),
                false,
            );
        }

        log::debug!("Navigating to heir list");
        navigator().push(Route::HeirListView {});

        log::debug!("HeirCreateForm: Form submission completed");
        *creating.write() = false;
    };

    use_drop(|| log::debug!("HeirCreateForm Dropped"));

    rsx! {
        div { class: "px-6 py-8 flex flex-col gap-6",

            // Form sections
            HeirNameSection { heir_name_state }

            KeyProviderSection {
                key_provider_config_state,
                wallet_component_error: None,
                flavor: KeyProviderSectionFlavor::Heir,
            }

            HeirConfigSection {
                heir_config_config_state,
                expect_key_provider_heirconfig,
            }


            ExportToServiceSectionForm {
                export_to_service_config_state,
                flavor: ExportToServiceSectionFormFlavor::Create,
            }

            // Submit button
            div { class: "flex justify-center mt-8",
                MaybeHighlight {
                    step: OnboardingStep::ClickCreateHeirButton,
                    progress: MaybeHighlightProgressType::ContextAdded(OnboardingContextItemId::HeirName),
                    button {
                        class: "btn btn-primary btn-lg",
                        disabled: !form_valid() || creating(),
                        onclick: submit_form,
                        if creating() {
                            span { class: "loading loading-spinner loading-sm mr-2" }
                            "Creating..."
                        } else {
                            DrawSvg::<AccountMultiplePlus> {}
                            "Create Heir"
                        }
                    }
                }
            }
        

        }
    }
}

/// Component for heir name input
#[component]
fn HeirNameSection(heir_name_state: HeirNameState) -> Element {
    log::debug!("HeirNameSection Rendered");

    let heir_names = use_context::<Resource<Vec<CheapClone<DbHeir>>>>();
    let heir_names_set = use_memo(move || {
        heir_names
            .lmap(|database_heirs| {
                database_heirs
                    .iter()
                    .map(|h| CCStr::from(h.name.as_str()))
                    .collect::<HashSet<_>>()
            })
            .unwrap_or_default()
    });

    // Internal state - not exposed to parent
    let heir_name = use_signal(String::new);

    // Internal validation
    let heir_name_present = use_memo(move || !heir_name.read().trim().is_empty());
    let heir_name_available = use_memo(move || {
        let name_ref = heir_name.read();
        let name = name_ref.trim();
        if name.is_empty() {
            true // Don't show error for empty name
        } else {
            !heir_names_set.read().contains(name)
        }
    });
    let heir_name_forbidden = use_memo(move || heir_name.read().trim() == "create");
    let heir_name_error = use_memo(move || {
        if !heir_name_present() {
            Some(CCStr::from("Heir name is required"))
        } else if heir_name_forbidden() {
            Some(CCStr::from("\"create\" cannot be used as an heir name"))
        } else if !heir_name_available() {
            Some(CCStr::from("This heir name is already in use"))
        } else {
            None
        }
    });

    // Update parent signal when internal state changes
    use_effect(move || {
        let result = if heir_name_error.read().is_none() {
            Ok(heir_name.read().trim().to_owned())
        } else {
            Err(())
        };
        heir_name_state.set(result);
    });

    let heir_name_state_ok = use_memo(move || {
        heir_name_state
            .read()
            .as_ref()
            .is_ok_and(|name| name.to_lowercase() == "backup")
    });

    use_drop(|| log::debug!("HeirNameSection Dropped"));

    rsx! {
        div { class: "card [--cardtitle-fs:var(--text-2xl)] border border-base-content/5 shadow-md",
            div { class: "card-body",
                h2 { class: "card-title", "Heir Name" }
                div { class: "card-subtitle mb-4",
                    "The heir name will be used to identify this heir in heritage configurations."
                }
                MaybeHighlight {
                    step: OnboardingStep::InputBackupHeirName,
                    progress: MaybeHighlightProgressType::Signal(heir_name_state_ok.into()),
                    div { class: "w-80",
                        InputField {
                            value: heir_name,
                            placeholder: "Unique heir name",
                            value_error: heir_name_error,
                        }
                    }
                }
            }
        }
    }
}
/// Component for heir configuration input
#[component]
fn HeirConfigSection(
    heir_config_config_state: HeirConfigConfigState,
    expect_key_provider_heirconfig: ReadOnlySignal<bool>,
) -> Element {
    log::debug!("HeirConfigSection Rendered");

    // Internal state - not exposed to parent
    let mut heir_config_type = use_signal(|| HeirConfigType::HeirXPubkey);
    let mut heir_config_value = use_signal(String::new);

    // Internal validation
    let heir_config_provided = use_memo(move || !heir_config_value.read().trim().is_empty());
    let heir_config_parsed = use_memo(move || match heir_config_type() {
        HeirConfigType::SingleHeirPubkey => heir_config_value
            .read()
            .trim()
            .parse()
            .map(HeirConfig::SingleHeirPubkey)
            .map_err(log_error_ccstr),
        HeirConfigType::HeirXPubkey => heir_config_value
            .read()
            .trim()
            .parse()
            .map(HeirConfig::HeirXPubkey)
            .map_err(log_error_ccstr),
    });

    let heir_config_error = use_memo(move || {
        if !expect_key_provider_heirconfig() {
            if !heir_config_provided() {
                Some(CCStr::from("Heir configuration value is required"))
            } else if let Err(ref e) = *heir_config_parsed.read() {
                Some(e.clone())
            } else {
                None
            }
        } else {
            None
        }
    });

    // Update parent signal when internal state changes
    use_effect(move || {
        let result = if !expect_key_provider_heirconfig() {
            heir_config_parsed()
                .map(HeirConfigConfig::Provided)
                .map_err(|_| ())
        } else {
            Ok(HeirConfigConfig::Generate(heir_config_type()))
        };
        heir_config_config_state.set(result);
    });

    let (error_display, mut signal_activity, onfocusout) =
        use_future_error_feedback(heir_config_error.into());

    use_drop(|| log::debug!("HeirConfigSection Dropped"));

    rsx! {
        div { class: "card [--cardtitle-fs:var(--text-2xl)] border border-base-content/5 shadow-md",
            div { class: "card-body",
                h2 { class: "card-title", "Heir Configuration" }
                div { class: "card-subtitle mb-4",
                    "The real data representing the heir and going into the Bitcoin blockchain."
                }


                label { class: "select",
                    span { class: "label", "Heir Type" }
                    select {
                        value: "{heir_config_type()}",
                        onchange: move |evt| {
                            if let Ok(v) = evt.parsed() {
                                heir_config_type.set(v);
                            }
                        },
                        for hct in HeirConfigType::list() {
                            option {
                                value: "{hct}",
                                selected: heir_config_type() == hct,
                                disabled: hct.disabled(),
                                "{hct.display()}"
                            }
                        }
                    }
                }

                if !expect_key_provider_heirconfig() {
                    fieldset { class: "fieldset",
                        legend { class: "fieldset-legend", "Heir Key" }
                        input {
                            r#type: "text",
                            class: "input w-full font-mono text-sm",
                            class: if error_display().is_some() { "input-error" },
                            class: if heir_config_parsed().is_ok() { "input-success" },
                            placeholder: match heir_config_type() {
                                HeirConfigType::HeirXPubkey => {
                                    "[73c5da0a/86'/0'/1751476594']xpubDDfvzhdVV4unsoKt5aE6dcsNsfeWbTgmLZPi8LQDYU2xixrYemMfWJ3BaVneH3u7DBQePdTwhpybaKRU95pi6PMUtLPBJLVQRpzEnjfjZzX/*"
                                }
                                HeirConfigType::SingleHeirPubkey => {
                                    "[99ccb69a/86'/0'/1751476594'/0/0]02ee39732e7f49cf4c9bd9b3faec01ed6f62a668fef33fbec0f2708e4cebf5bc9b"
                                }
                            },
                            value: "{heir_config_value.read()}",
                            oninput: move |evt| {
                                signal_activity();
                                heir_config_value.set(evt.value());
                            },
                            onfocusout,
                        }
                        div {
                            class: "fieldset-label",
                            class: if error_display().is_some() { "text-error" },
                            if let Some(e) = error_display() {
                                {e}
                            } else {
                                match heir_config_type() {
                                    HeirConfigType::HeirXPubkey => {
                                        "Descriptor Extended public key starting with 'xpub'"
                                    }
                                    HeirConfigType::SingleHeirPubkey => {
                                        "Descriptor compressed public key (66 hex characters)"
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
