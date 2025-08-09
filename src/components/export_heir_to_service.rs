use crate::prelude::*;

use std::collections::BTreeMap;

use btc_heritage_wallet::heritage_service_api_client::{
    EmailAddress, HeirContact, HeirPermission, HeirPermissions,
};

use crate::{
    components::{
        inputs::{
            use_future_error_feedback, use_future_error_feedback_from_parts, use_future_feedback,
        },
        svg::{Close, DrawSvg, SvgSize::Size3},
    },
    utils::{log_error_ccstr, CCStr},
};

#[derive(Debug, Clone, PartialEq)]
pub enum ExportToServiceConfig {
    DoNotExport,
    Export {
        name: Option<String>,
        email: EmailAddress,
        custom_message: Option<String>,
        permissions: HeirPermissions,
        additional_contacts: Vec<HeirContact>,
    },
}

#[derive(Debug, Clone, Copy, PartialEq)]
enum ContactType {
    Email,
    Phone,
}

impl ContactType {
    fn list() -> [Self; 2] {
        [Self::Email, Self::Phone]
    }

    fn display(self) -> &'static str {
        match self {
            ContactType::Email => "Email",
            ContactType::Phone => "Phone (coming soon)",
        }
    }

    fn disabled(self) -> bool {
        match self {
            ContactType::Phone => true,
            _ => false,
        }
    }
}

impl core::str::FromStr for ContactType {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "Email" => Ok(Self::Email),
            "Phone" => Ok(Self::Phone),
            _ => Err(()),
        }
    }
}

impl core::fmt::Display for ContactType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            Self::Email => "Email",
            Self::Phone => "Phone",
        })
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ExportToServiceSectionFormFlavor {
    Create,
    Export,
    Update(ReadOnlySignal<Result<ExportToServiceConfig, ()>>),
}

type ExportToServiceConfigState = Signal<Result<ExportToServiceConfig, ()>>;

/// Component for export to service configuration
#[component]
pub fn ExportToServiceSectionForm(
    export_to_service_config_state: ExportToServiceConfigState,
    flavor: ReadOnlySignal<ExportToServiceSectionFormFlavor>,
    action_button: Option<Element>,
) -> Element {
    log::debug!("ExportToServiceSection Rendered");
    use ExportToServiceSectionFormFlavor::*;

    // Internal state
    let mut export_enabled = use_signal(|| !matches!(flavor(), Create));
    let mut same_as_heir_name = use_signal(|| matches!(flavor(), Create | Export));
    let mut service_name = use_signal(String::new);
    let mut main_email_text = use_signal(String::new);
    let mut main_message = use_signal(String::new);
    let mut service_heir_permissions_state =
        use_signal(|| HeirPermissions::from([HeirPermission::IsHeir, HeirPermission::OwnerEmail]));
    let mut service_heir_contacts_state =
        use_signal(|| BTreeMap::<usize, Result<HeirContact, ()>>::new());
    let mut next_contact_key = use_signal(|| 0);

    use_effect(move || {
        if let Update(current_export_to_service_config_state) = flavor() {
            if let Ok(ExportToServiceConfig::Export {
                name,
                email,
                custom_message,
                permissions,
                additional_contacts,
            }) = current_export_to_service_config_state()
            {
                log::debug!(
                    "ExportToServiceSectionForm - Update from export_to_service_config_state"
                );
                *same_as_heir_name.write() = false;
                *service_name.write() = name.unwrap_or_default();
                *main_email_text.write() = email.to_string();
                *main_message.write() = custom_message.unwrap_or_default();
                *service_heir_permissions_state.write() = permissions;
                *service_heir_contacts_state.write() = additional_contacts
                    .into_iter()
                    .map(Ok)
                    .enumerate()
                    .collect();
                *next_contact_key.write() = service_heir_contacts_state.peek().len();
            }
        }
    });

    // Validation

    let service_name_provided = use_memo(move || {
        if !export_enabled() || same_as_heir_name() {
            true
        } else {
            !service_name.read().trim().is_empty()
        }
    });

    let main_email_provided = use_memo(move || !main_email_text.read().trim().is_empty());
    let main_email = use_memo(move || {
        main_email_text
            .read()
            .trim()
            .parse()
            .map_err(log_error_ccstr)
    });
    let main_email_error = use_memo(move || {
        if !export_enabled() {
            None
        } else if !main_email_provided() {
            Some(CCStr::from("Email is mandatory"))
        } else if let Err(ref e) = *main_email.read() {
            Some(e.clone())
        } else {
            None
        }
    });

    let service_connected = use_memo(move || {
        matches!(
            &*state_management::SERVICE_STATUS.read(),
            Some(ServiceStatus::Connected(_))
        )
    });

    // Update parent signal when internal state changes
    use_effect(move || {
        let result = if !export_enabled() {
            Ok(ExportToServiceConfig::DoNotExport)
        } else if !service_connected() {
            Err(())
        } else {
            let name = if same_as_heir_name() {
                Ok(None)
            } else {
                if service_name.read().trim().is_empty() {
                    Err(())
                } else {
                    Ok(Some(service_name()))
                }
            };

            let additional_contacts = service_heir_contacts_state
                .read()
                .values()
                .cloned()
                .collect::<Result<Vec<_>, _>>();

            let custom_message = (!main_message.read().trim().is_empty()).then(|| main_message());

            match (name, main_email(), additional_contacts) {
                (Ok(name), Ok(email), Ok(additional_contacts)) => {
                    Ok(ExportToServiceConfig::Export {
                        name,
                        email,
                        custom_message,
                        permissions: service_heir_permissions_state(),
                        additional_contacts,
                    })
                }
                _ => Err(()),
            }
        };
        export_to_service_config_state.set(result);
    });

    let (feed_back_active, mut main_email_signal_activity, main_email_onfocusout) =
        use_future_feedback();
    let main_email_error_display =
        use_future_error_feedback_from_parts(feed_back_active, main_email_error.into());

    let validate_onboarding_email =
        use_memo(move || main_email.read().is_ok() && feed_back_active());

    use_drop(|| log::debug!("ExportToServiceSection Dropped"));

    rsx! {

        div { class: "card [--cardtitle-fs:var(--text-2xl)] border border-base-content/5 shadow-md my-4",
            div { class: "card-body",
                h2 { class: "card-title",
                    match flavor() {
                        Create | Export => "Export to Service (optional)",
                        Update(_) => "Update Service Infos",
                    }
                }
                div { class: "card-subtitle mb-4",
                    "Declare the heir in the service so they automatically receive "
                    "notifications and support when their inheritance matures."
                }

                if matches!(flavor(), Create) {
                    // Export toggle
                    MaybeHighlight {
                        step: OnboardingStep::ClickExportHeirToService,
                        progress: MaybeHighlightProgressType::Signal(export_enabled.into()),
                        label { class: "label",
                            input {
                                r#type: "checkbox",
                                class: "toggle toggle-secondary",
                                checked: export_enabled(),
                                onchange: move |evt| export_enabled.set(evt.checked()),
                            }
                            span { class: "text-base ml-2",
                                if export_enabled() {
                                    "Export to Service"
                                } else {
                                    "Do Not Export to Service"
                                }
                            }
                        }
                    }
                }

                if export_enabled() {
                    if !service_connected() {
                        div { class: "fieldset-label text-error", "Heritage Service is not connected" }
                    }
                    // Name fieldset
                    fieldset { class: "fieldset",
                        legend { class: "fieldset-legend", "Name" }
                        div { class: "fieldset-description",
                            "The name that will be used for this heir in the service."
                        }
                        if matches!(flavor(), Create | Export) {
                            label { class: "label mb-2",
                                input {
                                    r#type: "checkbox",
                                    class: "toggle toggle-secondary",
                                    checked: same_as_heir_name(),
                                    onchange: move |evt| same_as_heir_name.set(evt.checked()),
                                }
                                span { class: "text-sm ml-2",
                                    if same_as_heir_name() {
                                        "Same as local"
                                    } else {
                                        "Custom"
                                    }
                                }
                            }
                        }
                        if !same_as_heir_name() {
                            input {
                                r#type: "text",
                                class: "input w-full",
                                class: if !service_name_provided() { "input-error" },
                                placeholder: "Heir name in service",
                                value: "{service_name.read()}",
                                oninput: move |evt| service_name.set(evt.value()),
                            }
                            div {
                                class: "fieldset-label",
                                class: if !service_name_provided() { "text-error" },
                                class: if service_name_provided() { "invisible" },
                                if !service_name_provided() {
                                    "Name is required"
                                } else {
                                    "ph"
                                }
                            }
                        }
                    }

                    // Main Contact fieldset
                    fieldset { class: "fieldset",
                        legend { class: "fieldset-legend", "Main Contact Email" }
                        div { class: "fieldset-description",
                            "Primary contact information for this heir."
                        }

                        MaybeHighlight {
                            step: OnboardingStep::InputEmailAddress,
                            progress: MaybeHighlightProgressType::Signal(validate_onboarding_email.into()),
                            input {
                                r#type: "email",
                                class: "input w-full",
                                class: if main_email_error_display().is_some() { "input-error" },
                                placeholder: "heir@example.com",
                                value: main_email_text,
                                oninput: move |evt| {
                                    main_email_signal_activity();
                                    main_email_text.set(evt.value());
                                },
                                onfocusout: main_email_onfocusout,
                            }
                        }
                        div {
                            class: "fieldset-label",
                            class: if main_email_error_display().is_some() { "text-error" },
                            class: if main_email_error_display().is_none() { "invisible" },
                            if let Some(e) = main_email_error_display() {
                                {e}
                            } else {
                                "ph"
                            }
                        }
                    }

                    // Custom Message
                    fieldset { class: "fieldset",
                        legend { class: "fieldset-legend", "Custom Message (Optional)" }
                        div { class: "fieldset-description",
                            "An optional message to include in messages sent to the heir."
                        }
                        textarea {
                            class: "textarea w-full",
                            placeholder: "Optional custom message for the heir",
                            rows: "3",
                            value: main_message,
                            oninput: move |evt| main_message.set(evt.value()),
                        }
                    }

                    // Permissions fieldset
                    ServiceHeirPermissions { service_heir_permissions_state }

                    // Additional Contacts
                    fieldset { class: "fieldset",
                        legend { class: "fieldset-legend", "Additional Contacts" }
                        div { class: "fieldset-description",
                            "Optional additional ways to contact this heir."
                        }

                        div { class: "flex flex-col gap-1",
                            for contact_key in service_heir_contacts_state.read().keys() {
                                ServiceHeirAdditionalContact {
                                    key: "{contact_key}",
                                    contact_key: *contact_key,
                                    service_heir_contacts_state,
                                }
                            }

                            button {
                                class: "btn btn-outline btn-sm self-start",
                                onclick: move |_| {
                                    let contact_key = next_contact_key();
                                    *next_contact_key.write() += 1;
                                    service_heir_contacts_state.write().insert(contact_key, Err(()));
                                },
                                "Add Contact"
                            }
                        }
                    }

                    if let Some(action_button) = action_button {
                        // Submit button
                        div { class: "flex justify-center mt-8", {action_button} }
                    }
                }
            }
        }
    }
}

#[component]
fn ServiceHeirPermissions(service_heir_permissions_state: Signal<HeirPermissions>) -> Element {
    log::debug!("ServiceHeirPermissions Rendered");

    use_drop(|| log::debug!("ServiceHeirPermissions Dropped"));

    rsx! {
        // Permissions fieldset
        fieldset { class: "fieldset",
            legend { class: "fieldset-legend", "Permissions" }
            div { class: "fieldset-description",
                "What information the heir can see about their inheritance."
            }

            div { class: "flex flex-row flex-wrap gap-8",
                ServiceHeirPermission {
                    value: HeirPermission::IsHeir,
                    label: "Will inherit",
                    description: "Can see the inheritance info before maturity",
                    service_heir_permissions_state,
                    stared: true,
                }
                ServiceHeirPermission {
                    value: HeirPermission::Amount,
                    label: "Amount",
                    description: "Can see the inheritance amount",
                    service_heir_permissions_state,
                    stared: true,
                }
                ServiceHeirPermission {
                    value: HeirPermission::Maturity,
                    label: "Maturity",
                    description: "Can see inheritance maturity and expiration dates",
                    service_heir_permissions_state,
                    stared: true,
                }
                ServiceHeirPermission {
                    value: HeirPermission::OwnerEmail,
                    label: "Owner Email",
                    description: "Can see the inheritance owner's email",
                    service_heir_permissions_state,
                }
                ServiceHeirPermission {
                    value: HeirPermission::Position,
                    label: "Position",
                    description: "Can see their position in the Heritage configuration",
                    service_heir_permissions_state,
                }
            }
            div { class: "fieldset-label text-warning",
                span { class: "font-black text-lg", "*" }
                " This permission will always be avaiable once an inheritance matures"
            }
        }
    }
}

#[component]
fn ServiceHeirPermission(
    value: HeirPermission,
    label: &'static str,
    description: &'static str,
    service_heir_permissions_state: Signal<HeirPermissions>,
    stared: Option<bool>,
) -> Element {
    rsx! {
        label { class: "label w-80",
            input {
                r#type: "checkbox",
                class: "checkbox checkbox-lg",
                checked: service_heir_permissions_state.read().contains(&value),
                onchange: move |evt| {
                    if evt.checked() {
                        service_heir_permissions_state.write().insert(value);
                    } else {
                        service_heir_permissions_state.write().remove(&value);
                    }
                },
            }
            div {
                div { class: "font-bold text-lg",
                    {label}
                    if stared.is_some_and(|b| b) {
                        span { class: "text-warning font-black", " *" }
                    }
                }
                div { class: "text-sm text-wrap", {description} }
            }
        }
    }
}

#[component]
fn ServiceHeirAdditionalContact(
    contact_key: usize,
    service_heir_contacts_state: Signal<BTreeMap<usize, Result<HeirContact, ()>>>,
) -> Element {
    // Internal state
    let mut contact_type = use_signal(|| {
        if let Ok(ref shcs) = service_heir_contacts_state.read()[&contact_key] {
            match shcs {
                HeirContact::Email { .. } => ContactType::Email,
            }
        } else {
            ContactType::Email
        }
    });
    let mut contact_text = use_signal(|| {
        if let Ok(ref shcs) = service_heir_contacts_state.read()[&contact_key] {
            match shcs {
                HeirContact::Email { email } => email.to_string(),
            }
        } else {
            String::new()
        }
    });

    // Validation
    let contact_provided = use_memo(move || !contact_text.read().trim().is_empty());
    let contact = use_memo(move || match contact_type() {
        ContactType::Email => contact_text
            .read()
            .trim()
            .parse()
            .map(|email| HeirContact::Email { email })
            .map_err(log_error_ccstr),
        ContactType::Phone => Err(CCStr::from("Not supported yet")),
    });
    let contact_error = use_memo(move || {
        if !contact_provided() {
            let type_name = match contact_type() {
                ContactType::Email => "Email",
                ContactType::Phone => "Phone number",
            };
            Some(CCStr::from(format!("{type_name} is required")))
        } else if let Err(ref e) = *contact.read() {
            Some(e.clone())
        } else {
            None
        }
    });

    // Update parent signal when internal state changes
    use_effect(move || {
        service_heir_contacts_state
            .write()
            .insert(contact_key, contact().map_err(|_| ()));
    });

    let (error_display, mut signal_activity, onfocusout) =
        use_future_error_feedback(contact_error.into());

    rsx! {
        div { class: "grid grid-cols-[128px_auto_24px] grid-rows-[auto_auto] gap-x-2",
            select {
                class: "select select-sm",
                value: "{contact_type()}",
                onchange: move |evt| {
                    if let Ok(new_type) = evt.parsed() {
                        contact_type.set(new_type);
                    }
                },
                for ct in ContactType::list() {
                    option {
                        value: "{ct}",
                        selected: contact_type() == ct,
                        disabled: ct.disabled(),
                        "{ct.display()}"
                    }
                }
            }
            input {
                r#type: "text",
                class: "input input-sm w-full",
                class: if error_display().is_some() { "input-error" },
                placeholder: match contact_type() {
                    ContactType::Email => "email@example.com",
                    ContactType::Phone => "+1234567890",
                },
                value: contact_text,
                oninput: move |evt| {
                    signal_activity();
                    *contact_text.write() = evt.value();
                },
                onfocusout,
            }


            button {
                class: "btn btn-circle btn-outline btn-primary btn-xs self-center",
                onclick: move |_| {
                    _ = service_heir_contacts_state.write().remove(&contact_key);
                },
                DrawSvg::<Close> { size: Size3 }
            }
            div {
                class: "fieldset-label col-start-2",
                class: if error_display().is_some() { "text-error" },
                class: if error_display().is_none() { "invisible" },
                if let Some(e) = error_display() {
                    {e}
                } else {
                    "ph"
                }
            }
        }
    }
}
