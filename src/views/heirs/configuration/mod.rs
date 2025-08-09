use crate::prelude::*;

mod delete;

use std::collections::BTreeSet;

use btc_heritage_wallet::{
    heritage_service_api_client::{Heir as ServiceHeir, HeirCreate, HeirUpdate, MainContact},
    Heir as DbHeir,
};

use crate::{
    components::{
        export_heir_to_service::{
            ExportToServiceConfig, ExportToServiceSectionForm, ExportToServiceSectionFormFlavor,
        },
        inputs::RenameDatabaseItem,
        misc::BackButton,
        svg::{DrawSvg, Edit, ExportVariant},
    },
    utils::{CCStr, CheapClone},
    Route,
};

#[component]
pub fn HeirConfigurationView(heir_index: usize) -> Element {
    log::debug!("HeirWalletConfigurationView Rendered");

    let service_client_service = state_management::use_service_client_service();

    let mut service_heirs = use_context::<FResource<Vec<CheapClone<ServiceHeir>>>>();
    let composite_heirs = use_context::<Memo<Vec<CompositeHeir>>>();

    let composite_heir = use_memo(move || composite_heirs.read()[heir_index].clone());

    let is_in_db = use_signal(move || composite_heir.read().db_heir.is_some());

    let composite_heir_fingerprint =
        use_memo(move || CCStr::from(composite_heir().heir_config.fingerprint().to_string()));

    let current_export_to_service_config_state = use_memo(move || {
        log::debug!("HeirWalletConfigurationView - Update current_export_to_service_config_state");
        match composite_heir.read().service_heir.clone() {
            Some(Some(sh)) => Ok(ExportToServiceConfig::Export {
                name: Some(sh.display_name.clone()),
                email: sh.main_contact.email.clone(),
                custom_message: sh.main_contact.custom_message.clone(),
                permissions: sh.permissions.clone(),
                additional_contacts: sh.additional_contacts.iter().cloned().collect(),
            }),
            Some(None) => Ok(ExportToServiceConfig::DoNotExport),
            _ => Err(()),
        }
    });
    let have_service_status = use_memo(move || {
        log::debug!("HeirWalletConfigurationView - Update have_service_status");
        current_export_to_service_config_state.read().is_ok()
    });

    let is_already_exported = use_memo(move || {
        log::debug!("HeirWalletConfigurationView - Update is_already_exported");
        current_export_to_service_config_state
            .read()
            .as_ref()
            .is_ok_and(|cetscs| matches!(cetscs, ExportToServiceConfig::Export { .. }))
    });

    let export_to_service_config_state = use_signal(|| Err(()));
    let mut processing = use_signal(|| false);

    // Combined validation
    let form_valid = use_memo(move || export_to_service_config_state.read().is_ok());
    let has_changes = use_memo(move || {
        *current_export_to_service_config_state.read() != *export_to_service_config_state.read()
    });
    let mut abort = move |message: &str| {
        *processing.write() = false;
        alert_error(message);
        log::error!("{message}");
    };
    // Handle form submission
    fn warn(message: impl AsRef<str>) {
        alert_warn(message.as_ref());
        log::warn!("{}", message.as_ref());
    }
    fn success(message: impl AsRef<str>) {
        alert_success(message.as_ref());
        log::info!("{}", message.as_ref());
    }
    let export_click = move |_| async move {
        log::debug!("HeirWalletConfigurationView: Export started");
        *processing.write() = true;

        let Ok(export_to_service_config) = export_to_service_config_state() else {
            return abort("Invalid Export To Service configuration");
        };
        log::debug!(
            "HeirCreateForm: Export to service config: {:?}",
            export_to_service_config
        );

        log::debug!("HeirWalletConfigurationView: Processing service export configuration");
        let mut refresh_service_heirs = false;
        if let ExportToServiceConfig::Export {
            name,
            email,
            custom_message,
            permissions,
            additional_contacts,
        } = export_to_service_config
        {
            log::debug!("HeirWalletConfigurationView: Exporting heir to service");
            refresh_service_heirs = true;
            let client = state_management::heritage_service_client(service_client_service).await;
            log::debug!("HeirWalletConfigurationView: Got heritage service client");

            let post_heirs_result = client
                .post_heirs(HeirCreate {
                    display_name: name.unwrap_or_else(|| composite_heir.read().name.to_string()),
                    heir_config: (*composite_heir.read().heir_config).clone(),
                    main_contact: MainContact {
                        email,
                        custom_message,
                    },
                    permissions,
                })
                .await;
            log::debug!("HeirWalletConfigurationView: Heir post request completed");

            let service_heir_id = match post_heirs_result {
                Ok(h) => {
                    log::debug!(
                        "HeirWalletConfigurationView: Heir exported to service successfully with ID: {}",
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
                        "HeirWalletConfigurationView: Adding additional contacts for heir ID: {}",
                        heir_id
                    );
                    // Add contacts
                    match client
                        .post_heir_contacts(&heir_id, additional_contacts)
                        .await
                    {
                        Ok(_) => {
                            log::debug!(
                            "HeirWalletConfigurationView: Additional contacts added successfully"
                        );
                        }
                        Err(e) => {
                            warn(format!("Could not add additional contacts to the exported Heir in the service: {e}"));
                        }
                    };
                }
            }
        } else {
            log::debug!("HeirWalletConfigurationView: Skipping service export");
        };

        // Need to refresh to "insert" the newly created heir
        if refresh_service_heirs {
            log::debug!("HeirWalletConfigurationView: Refreshing heir lists");
            log::debug!("service_heirs.restart()");
            service_heirs.restart();
        }
        log::debug!("HeirWalletConfigurationView: Form submission completed");
        *processing.write() = false;
    };

    let update_click = move |_| async move {
        log::debug!("HeirWalletConfigurationView: Update started");
        *processing.write() = true;

        let Ok(export_to_service_config) = export_to_service_config_state() else {
            return abort("Invalid Export To Service configuration");
        };
        let Ok(current_export_to_service_config_state) = current_export_to_service_config_state()
        else {
            return abort("Invalid Current Export To Service configuration");
        };
        let service_heir_id =
            if let Some(Some(ref service_heir)) = composite_heir.read().service_heir {
                service_heir.id.clone()
            } else {
                return abort("Invalid Current Export To Service configuration");
            };

        let (heir_update, contacts_to_add, contacts_to_delete) = match (
            current_export_to_service_config_state,
            export_to_service_config,
        ) {
            (
                ExportToServiceConfig::Export {
                    name: Some(current_name),
                    email: current_email,
                    custom_message: current_custom_message,
                    permissions: current_permissions,
                    additional_contacts: current_additional_contacts,
                },
                ExportToServiceConfig::Export {
                    name: Some(name),
                    email,
                    custom_message,
                    permissions,
                    additional_contacts,
                },
            ) => {
                let heir_update = HeirUpdate {
                    display_name: (current_name != name).then_some(name),
                    main_contact: (current_email != email
                        || current_custom_message != custom_message)
                        .then(|| MainContact {
                            email,
                            custom_message,
                        }),
                    permissions: (current_permissions != permissions).then_some(permissions),
                };
                let current_additional_contacts_index = current_additional_contacts
                    .into_iter()
                    .collect::<BTreeSet<_>>();
                let additional_contacts_index =
                    additional_contacts.into_iter().collect::<BTreeSet<_>>();
                let contacts_to_add = additional_contacts_index
                    .iter()
                    .filter(|c| !current_additional_contacts_index.contains(*c))
                    .cloned()
                    .collect::<Vec<_>>();
                let contacts_to_delete = current_additional_contacts_index
                    .iter()
                    .filter(|c| !additional_contacts_index.contains(*c))
                    .cloned()
                    .collect::<Vec<_>>();
                (
                    (heir_update.display_name.is_some()
                        || heir_update.main_contact.is_some()
                        || heir_update.permissions.is_some())
                    .then_some(heir_update),
                    (!contacts_to_add.is_empty()).then_some(contacts_to_add),
                    (!contacts_to_delete.is_empty()).then_some(contacts_to_delete),
                )
            }
            _ => (None, None, None),
        };

        let mut refresh_service_heirs = false;
        let client = state_management::heritage_service_client(service_client_service).await;
        if let Some(heir_update) = heir_update {
            log::debug!("HeirCreateForm: heir_update: {heir_update:?}");
            refresh_service_heirs = true;
            // Update heir
            match client.patch_heir(&service_heir_id, heir_update).await {
                Ok(_) => {
                    success("Successfully updated heir in the service");
                }
                Err(e) => {
                    warn(format!("Could not update the heir in the service: {e}"));
                }
            };
        }

        if let Some(contacts_to_delete) = contacts_to_delete {
            log::debug!("HeirCreateForm: contacts_to_delete: {contacts_to_delete:?}");
            refresh_service_heirs = true;
            // Remove contacts
            match client
                .delete_heir_contacts(&service_heir_id, contacts_to_delete)
                .await
            {
                Ok(_) => {
                    success("Removed additional contacts in the service");
                }
                Err(e) => {
                    warn(format!(
                        "Could not remove additional contacts in the service: {e}"
                    ));
                }
            };
        }
        if let Some(contacts_to_add) = contacts_to_add {
            log::debug!("HeirCreateForm: contacts_to_add: {contacts_to_add:?}");
            refresh_service_heirs = true;
            // Add contacts
            match client
                .post_heir_contacts(&service_heir_id, contacts_to_add)
                .await
            {
                Ok(_) => {
                    success("Added additional contacts in the service");
                }
                Err(e) => {
                    warn(format!(
                        "Could not add additional contacts in the service: {e}"
                    ));
                }
            };
        }

        // Need to refresh to "insert" the newly created heir
        if refresh_service_heirs {
            log::debug!("HeirWalletConfigurationView: Refreshing heir lists");
            log::debug!("service_heirs.restart()");
            service_heirs.restart();
        }
        log::debug!("HeirWalletConfigurationView: Form submission completed");
        *processing.write() = false;
    };

    use_drop(|| log::debug!("HeirWalletConfigurationView Dropped"));

    rsx! {
        super::super::TitledView {
            title: composite_heir.read().name.clone(),
            subtitle: composite_heir_fingerprint(),
            left: rsx! {
                BackButton { route: Route::HeirView { heir_index } }
            },
            delete::DeleteHeirSeedConfig {}

            if have_service_status() {
                if is_already_exported() {
                    ExportToServiceSectionForm {
                        export_to_service_config_state,
                        flavor: ExportToServiceSectionFormFlavor::Update(
                            current_export_to_service_config_state.into(),
                        ),
                        action_button: rsx! {
                            button {
                                class: "btn btn-primary btn-lg",
                                disabled: !form_valid() || !has_changes() || processing(),
                                onclick: update_click,
                                if processing() {
                                    span { class: "loading loading-spinner loading-sm mr-2" }
                                    "Updating..."
                                } else {
                                    DrawSvg::<Edit> {}
                                    "Update Heir"
                                }
                            }
                        },
                    }
                } else {
                    ExportToServiceSectionForm {
                        export_to_service_config_state,
                        flavor: ExportToServiceSectionFormFlavor::Export,
                        action_button: rsx! {
                            button {
                                class: "btn btn-primary btn-lg",
                                disabled: !form_valid() || processing(),
                                onclick: export_click,
                                if processing() {
                                    span { class: "loading loading-spinner loading-sm mr-2" }
                                    "Exporting..."
                                } else {
                                    DrawSvg::<ExportVariant> {}
                                    "Export Heir"
                                }
                            }
                        },
                    }
                }
            }

            if is_in_db() {
                RenameDatabaseItem::<DbHeir> {}
            }

            delete::DeleteHeirConfig {}
        }
    }
}
