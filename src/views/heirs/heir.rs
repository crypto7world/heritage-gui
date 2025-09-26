use crate::prelude::*;

use btc_heritage_wallet::{
    heritage_service_api_client::{
        Heir as ServiceHeir, HeirContact, HeirPermission, HeirPermissions,
    },
    AnyKeyProvider, Heir,
};

use crate::{
    components::{
        badge::UIHeirBadges,
        onboarding::MaybeOnPathHighlight,
        quick_actions::{ShowKeyProviderMnemonic, ShowKeyProviderMnemonicFlavor},
        svg::{CheckCircle, Close, Cog, DrawSvg, SvgSize::Full, SvgSize::Size5},
    },
    utils::{heir_config_type_to_string, CCStr},
    Route,
};

#[component]
pub fn HeirWrapperLayout(heir_index: usize) -> Element {
    log::debug!("HeirWrapperLayout Rendered");

    let composite_heirs = use_context::<Memo<Vec<CompositeHeir>>>();
    let composite_heir = use_memo(move || composite_heirs.read()[heir_index].clone());
    let composite_heir_fingerprint =
        use_memo(move || CCStr::from(composite_heir().heir_config.fingerprint().to_string()));

    // Provide the heir resources to all child that may want it
    use_context_provider(|| composite_heir);
    use_context_provider(|| composite_heir_fingerprint);

    // Use a Signal to ensure it is only computed ONCE!
    let is_in_db = use_signal(move || composite_heir.read().db_heir.is_some());

    // Should be to create a AsyncSignal in a conditional here because there is no way (I think!)
    // to re-load this component with a different index without dropping it first
    if is_in_db() {
        let db_heir = helper_hooks::use_async_heir(composite_heir.read().name.clone());
        use_context_provider(|| db_heir);
    }

    use_context_provider(move || {
        OnboardingContextItemId::HeirName.item(composite_heir.read().name.to_string())
    });

    use_drop(|| log::debug!("HeirWrapperLayout Dropped"));
    rsx! {
        Outlet::<crate::Route> {}
    }
}

#[component]
pub fn HeirView(heir_index: usize) -> Element {
    log::debug!("HeirView Rendered");

    let composite_heir = use_context::<Memo<CompositeHeir>>();
    let composite_heir_name = use_memo(move || composite_heir.read().name.clone());

    let display_heir = use_memo(move || {
        composite_heir
            .read()
            .service_heir
            .lmap(|o_sh| Display::from(o_sh.as_ref().map(|h| UIServiceHeir::from_ref(h))))
    });

    let opt_db_heir = try_consume_context::<AsyncSignal<Heir>>();
    // Use a Signal to ensure it is only computed ONCE!
    let can_show_signal = use_memo(move || {
        if let Some(opt_db_heir) = opt_db_heir {
            opt_db_heir
                .lmap(|db_heir| match db_heir.key_provider() {
                    AnyKeyProvider::LocalKey(_) => true,
                    AnyKeyProvider::None | AnyKeyProvider::Ledger(_) => false,
                })
                .unwrap_or(false)
        } else {
            false
        }
    });

    let composite_heir_fingerprint = use_context::<Memo<CCStr>>();

    let click_config = move |_| {
        navigator().push(Route::HeirConfigurationView { heir_index });
    };

    use_drop(|| log::debug!("HeirView Dropped"));

    rsx! {
        super::super::TitledView {
            title: composite_heir_name(),
            subtitle: composite_heir_fingerprint(),
            right: rsx! {
                div { class: "h-full content-center",
                    MaybeOnPathHighlight {
                        steps: &[OnboardingStep::CheckConfirmStripHeirSeed, OnboardingStep::StripHeirSeed],
                        context_filter: consume_onboarding_context(),
                        button {
                            class: "btn btn-circle btn-outline btn-primary btn-lg p-2",
                            onclick: click_config,
                            DrawSvg::<Cog> { size: Full }
                        }
                    }
                }
            },

            div { class: "flex flex-col gap-2 items-center",
                div { class: "flex flex-row flex-wrap justify-center gap-2",
                    LoadedComponent::<UIHeirBadges> { input: composite_heir.into() }
                }
                if can_show_signal() {
                    ShowKeyProviderMnemonic::<Heir> { flavor: ShowKeyProviderMnemonicFlavor::Heir }
                }
            }

            HeirConfigComponent {}

            LoadedComponent::<Display<UIServiceHeir>> { input: display_heir().into() }

            OnboardingInfoModal { step: OnboardingStep::ModalExplainStoreHeirMnemonic,
                div { class: "flex flex-col gap-4 max-w-xl text-base",
                    p {
                        "This first heir is your \"backup\" heir. If you loose access to your
                        wallet, this heir will act as a backdoor allowing you to eventually
                        recover your bitcoins."
                    }
                    p {
                        "You should now securely store both the "
                        span { class: "font-bold", "mnemonic phrase" }
                        " and the "
                        span { class: "font-bold", "key fingerprint" }
                        " in a sealed, offline environment."
                    }
                    div { class: "bg-base-200 p-4 rounded-lg",
                        div { class: "font-semibold mb-2", "Recommended secure storage:" }
                        ul { class: "list-disc list-inside space-y-1 text-sm",
                            li { "Write both the mnemonic and fingerprint on paper" }
                            li { "Place them in a sealed envelope or secure container" }
                            li {
                                "Store in a safe place where you can trivially verify it is still in place, still sealed"
                            }
                        }
                    }
                }
            }
        }
    }
}

#[component]
fn HeirConfigComponent() -> Element {
    log::debug!("HeirConfig Rendered");

    let composite_heir = use_context::<Memo<CompositeHeir>>();
    let composite_heir_fingerprint = use_context::<Memo<CCStr>>();

    let heir_config = use_memo(move || composite_heir.read().heir_config.clone());
    // Derive the heir configuration type and value
    let heir_config_type = use_memo(move || heir_config_type_to_string(&heir_config()));

    let heir_config_value = use_memo(move || CCStr::from(heir_config.read().to_string()));

    use_drop(|| log::debug!("HeirConfig Dropped"));

    rsx! {
        div { class: "rounded-box border border-base-content/5 shadow-md bg-base-100 my-4 max-w-7xl mx-auto",
            div { class: "p-6",
                // Title
                h2 { class: "text-2xl font-bold mb-4", "Heir Configuration" }

                // Explanatory text
                div { class: "prose prose-sm max-w-none mb-6",
                    p { class: "mb-3",
                        "The Heir Configuration is a cryptographic identifier that uniquely
                        defines this heir on the Bitcoin blockchain. It contains the
                        public key information necessary for the heir to eventually claim
                        inherited funds when the specified timelock conditions are met."
                    }
                    p {
                        "This configuration is embedded into the Heritage Wallet's spending
                        conditions on the blockchain and cannot be changed once set. It
                        ensures that only this specific heir, possessing the corresponding
                        private keys, can access the inherited funds once they are elligible."
                    }
                }

                // Configuration display
                div { class: "flex flex-col gap-4",
                    div { class: "flex flex-row gap-8",
                        div { class: "flex flex-col",
                            div { class: "font-light", "Type" }
                            div { class: "text-lg font-bold text-nowrap", {heir_config_type} }
                        }
                        div { class: "flex flex-col",
                            div { class: "font-light text-nowrap", "Key Fingerprint" }
                            div { class: "text-lg font-bold", {composite_heir_fingerprint()} }
                        }
                    }

                    fieldset { class: "fieldset",
                        legend { class: "fieldset-legend text-lg", "Heir Configuration Value" }
                        div { class: "fieldset-description",
                            "This value can be copied for use in an Heritage Wallet Configuration."
                        }
                        div { class: "input input-lg w-full font-mono focus:outline-none overflow-x-scroll",
                            "{heir_config_value()}"
                        }
                    }
                }
            }
        }
    }
}

/// UI component for displaying service heir information
#[derive(Debug, Clone, PartialEq)]
struct UIServiceHeir {
    /// Display name of the heir
    name: CCStr,
    /// Main contact email
    email: CCStr,
    /// Optional custom message
    custom_message: Option<CCStr>,
    /// Set of permissions granted to the heir
    permissions: HeirPermissions,
    /// Additional contacts for the heir
    additional_contacts: Vec<HeirContact>,
}

impl LoadedElement for UIServiceHeir {
    type Loader = SkeletonLoader;

    fn element<M: LoadedComponentInputMapper>(self, _m: M) -> Element {
        rsx! {
            div { class: "rounded-box border border-base-content/5 shadow-md bg-base-100 my-4 max-w-7xl mx-auto",
                div { class: "p-6",
                    // Title
                    h2 { class: "text-2xl font-bold mb-4", "Service Heir Information" }

                    // Explanatory text
                    div { class: "prose prose-sm max-w-none mb-6",
                        p { class: "mb-3",
                            "This heir is exported to the Heritage Service and can receive notifications about inheritances. "
                            "The service will contact the heir when inheritance conditions are met and provide access based on the configured permissions."
                        }
                        p {
                            "The heir can access their inheritance information through the Heritage Service portal using their registered email address."
                        }
                    }

                    // Service heir details
                    div { class: "flex flex-col gap-4",
                        div { class: "flex flex-row gap-8",
                            div { class: "flex flex-col",
                                div { class: "font-light", "Display Name" }
                                div { class: "text-lg font-bold", {self.name} }
                            }
                            div { class: "flex flex-col",
                                div { class: "font-light", "Main Email" }
                                div { class: "text-lg font-bold", {self.email} }
                            }
                        }

                        fieldset { class: "fieldset",
                            legend { class: "fieldset-legend", "Custom Message" }
                            div { class: "fieldset-description",
                                "Personal message from the wallet owner to the heir."
                            }
                            if let Some(ref message) = self.custom_message {
                                textarea {
                                    class: "textarea textarea-bordered font-mono text-xs w-full",
                                    readonly: true,
                                    rows: "10",
                                    value: "{message}",
                                }
                            } else {
                                "None"
                            }
                        }

                        // Permissions display
                        fieldset { class: "fieldset",
                            legend { class: "fieldset-legend", "Permissions" }
                            div { class: "fieldset-description",
                                "Information the heir can access about their inheritance through the service."
                            }
                            div { class: "grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-3 gap-3",
                                ServiceHeirPermissionDisplay {
                                    label: "Will inherit",
                                    description: "Can see inheritance info before maturity",
                                    has_permission: self.permissions.contains(&HeirPermission::IsHeir),
                                    starred: true,
                                }
                                ServiceHeirPermissionDisplay {
                                    label: "Amount",
                                    description: "Can see the inheritance amount",
                                    has_permission: self.permissions.contains(&HeirPermission::Amount),
                                    starred: true,
                                }
                                ServiceHeirPermissionDisplay {
                                    label: "Maturity",
                                    description: "Can see inheritance maturity and expiration dates",
                                    has_permission: self.permissions.contains(&HeirPermission::Maturity),
                                    starred: true,
                                }
                                ServiceHeirPermissionDisplay {
                                    label: "Owner Email",
                                    description: "Can see the inheritance owner's email (your email)",
                                    has_permission: self.permissions.contains(&HeirPermission::OwnerEmail),
                                    starred: false,
                                }
                                ServiceHeirPermissionDisplay {
                                    label: "Position",
                                    description: "Can see their position in the Heritage configuration",
                                    has_permission: self.permissions.contains(&HeirPermission::Position),
                                    starred: false,
                                }
                            }
                            div { class: "fieldset-label text-warning mt-2",
                                span { class: "font-black text-lg", "*" }
                                " This permission will always be available once an inheritance matures"
                            }
                        }

                        // Additional contacts
                        if !self.additional_contacts.is_empty() {
                            fieldset { class: "fieldset",
                                legend { class: "fieldset-legend", "Additional Contacts" }
                                div { class: "fieldset-description",
                                    "Alternative contact methods for the heir."
                                }
                                div { class: "flex flex-col gap-2",
                                    for contact in &self.additional_contacts {
                                        ServiceHeirContactDisplay { contact: contact.clone() }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    fn place_holder() -> Self {
        Self {
            name: CCStr::place_holder(),
            email: CCStr::place_holder(),
            custom_message: None,
            permissions: HeirPermissions::from([]),
            additional_contacts: Vec::new(),
        }
    }
}

impl LoadedSuccessConversionMarker for TypeCouple<ServiceHeir, UIServiceHeir> {}

impl FromRef<ServiceHeir> for UIServiceHeir {
    fn from_ref(service_heir: &ServiceHeir) -> Self {
        UIServiceHeir {
            name: CCStr::from(service_heir.display_name.as_str()),
            email: CCStr::from(service_heir.main_contact.email.as_str()),
            custom_message: service_heir
                .main_contact
                .custom_message
                .as_ref()
                .map(CCStr::from),
            permissions: service_heir.permissions.clone(),
            additional_contacts: service_heir.additional_contacts.iter().cloned().collect(),
        }
    }
}

/// Component for displaying a single permission status
#[component]
fn ServiceHeirPermissionDisplay(
    label: &'static str,
    description: &'static str,
    has_permission: bool,
    starred: bool,
) -> Element {
    rsx! {
        div { class: "flex items-center gap-3 p-3 rounded-lg border",

            div { class: "flex-shrink-0 mt-0.5",
                if has_permission {
                    div { class: "text-success",
                        DrawSvg::<CheckCircle> { size: Size5 }
                    }
                } else {
                    div { class: "text-error",
                        DrawSvg::<Close> { size: Size5 }
                    }
                }
            }

            div { class: "flex-1 min-w-0",
                div { class: "flex items-center gap-1",
                    div {
                        class: "font-semibold",
                        class: if has_permission { "text-base-content" } else { "text-base-content/60" },
                        {label}
                    }
                    if starred {
                        span { class: "text-warning font-black text-lg", "*" }
                    }
                }
                div { class: "text-sm opacity-70 mt-1", {description} }
            }
        }
    }
}

/// Component for displaying a contact method
#[component]
fn ServiceHeirContactDisplay(contact: HeirContact) -> Element {
    match contact {
        HeirContact::Email { email } => rsx! {
            div { class: "flex items-center gap-2 p-2 bg-base-200 rounded",
                span { class: "badge badge-secondary badge-sm", "Email" }
                span { class: "font-mono text-sm", {email.to_string()} }
            }
        },
    }
}
