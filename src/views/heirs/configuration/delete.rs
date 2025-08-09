use crate::prelude::*;

use btc_heritage_wallet::{AnyKeyProvider, Heir};

use crate::{
    components::{
        delete::{AlertDeleteAck, AlertDeleteAckStyle, AlertDeleteKeyProvider},
        onboarding::MaybeOnPathHighlight,
        svg::{Delete, DrawSvg},
    },
    utils::CheapClone,
    Route,
};

/// Component for deleting an heir with appropriate warnings and confirmations
///
/// This component provides a dangerous operation interface for heir deletion.
/// It shows different warnings based on the heir wallet's configuration and requires
/// explicit user acknowledgment before allowing deletion.
#[component]
pub(super) fn DeleteHeirConfig() -> Element {
    log::debug!("DeleteHeirConfig Rendered");

    let database_service = state_management::use_database_service();
    let Some(heir) = try_use_context::<AsyncSignal<Heir>>() else {
        log::warn!("The heir is not in the database, nothing to delete");
        return rsx! {};
    };
    let mut database_heirs = use_context::<Resource<Vec<CheapClone<Heir>>>>();
    let composite_heir = use_context::<Memo<CompositeHeir>>();

    let mut in_operation = use_signal(|| false);
    let acknowledge_private_keys = use_signal(|| false);
    let acknowledge_service_warning = use_signal(|| false);

    // Check if heir uses LocalKey key provider
    let uses_local_key = use_memo(move || {
        heir.lmap(|heir| matches!(heir.key_provider(), AnyKeyProvider::LocalKey(_)))
            .unwrap_or(false)
    });

    // Check if heir uses Service
    let uses_service =
        use_memo(move || matches!(composite_heir.read().service_heir, Some(Some(_))));

    // Determine if all required acknowledgments are checked
    let all_acknowledged = use_memo(move || {
        let mut required_checks = true;

        if uses_local_key() {
            required_checks = required_checks && acknowledge_private_keys();
        }

        if uses_service() {
            required_checks = required_checks && acknowledge_service_warning();
        }

        required_checks
    });

    let delete_heir = move |_| async move {
        *in_operation.write() = true;

        match heir
            .with(async move |heir| state_management::delete_heir(database_service, heir).await)
            .await
        {
            Ok(()) => {
                log::info!("Heir deletion completed successfully");
                alert_success("Heir deletion completed successfully");
                log::debug!("HeirCreateForm: Refreshing database heirs list");
                database_heirs.restart();
                navigator().push(Route::HeirListView {});
            }
            Err(e) => {
                log::info!("Heir deletion failed: {e}");
                alert_error(format!("Heir deletion failed: {e}"));
            }
        }

        *in_operation.write() = false;
    };

    use_drop(|| log::debug!("DeleteHeirConfig Dropped"));

    rsx! {
        details { class: "collapse collapse-arrow rounded-box border border-error shadow-md my-4 bg-error/5",
            summary { class: "collapse-title text-2xl font-bold text-error", "Delete Heir" }
            div { class: "collapse-content p-4",

                // Warnings and acknowledgments
                div { class: "flex flex-col gap-4",

                    // Private key warning for LocalKey wallets
                    if uses_local_key() {
                        AlertDeleteKeyProvider { acknowledge_private_keys,
                            p {
                                "This heir uses local key storage. Deleting the heir will
                                permanently remove all private keys from this application.
                                If you have not backed up the mnemonic phrase,
                                this heir private keys will be unrecoverable forever."
                            }
                        }
                    }

                    // Service warning for Service heirs
                    if uses_service() {
                        AlertDeleteAck {
                            alert_style: AlertDeleteAckStyle::Warning,
                            acknowledge: acknowledge_service_warning,
                            acknowledgment: rsx! {
                                span { class: "font-black uppercase", "I understand" }
                                " that the online heir copy on the Heritage Service will remain."
                            },
                            h3 { class: "font-bold", "Service Wallet Notice" }
                            p {
                                "This heir is also present in the Heritage Service. Deleting the heir locally will NOT delete the heir data on the service. "
                                "If you need to delete the service copy, contact the service provider for assistance. "
                            }
                        }
                    }
                }

                // Delete button
                div { class: "flex justify-center mt-8",
                    button {
                        class: "btn btn-error btn-lg",
                        disabled: !all_acknowledged() || in_operation(),
                        onclick: delete_heir,
                        if in_operation() {
                            span { class: "loading loading-spinner loading-sm mr-2" }
                            "Deleting..."
                        } else {
                            DrawSvg::<Delete> {}
                            "Delete Heir"
                        }
                    }
                }
            }
        }
    }
}

/// Component for deleting an heir seed with appropriate warnings and confirmations
#[component]
pub(super) fn DeleteHeirSeedConfig() -> Element {
    log::debug!("DeleteHeirSeedConfig Rendered");

    let database_service = state_management::use_database_service();
    let Some(heir) = try_use_context::<AsyncSignal<Heir>>() else {
        log::warn!("The heir is not in the database, nothing to delete");
        return rsx! {};
    };
    let mut database_heirs = use_context::<Resource<Vec<CheapClone<Heir>>>>();

    let mut in_operation = use_signal(|| false);
    let acknowledge_private_keys = use_signal(|| false);
    let mut seed_stripped = use_signal(|| false);

    // Check if heir uses LocalKey key provider
    let uses_local_key = use_memo(move || {
        heir.lmap(|heir| matches!(heir.key_provider(), AnyKeyProvider::LocalKey(_)))
            .unwrap_or(false)
    });

    let delete_heir_seed = move |_| async move {
        *in_operation.write() = true;

        match heir
            .with(async move |heir| state_management::strip_heir_seed(database_service, heir).await)
            .await
        {
            Ok(()) => {
                seed_stripped.set(true);
                log::info!("Heir seed striping completed successfully");
                alert_success("Heir seed striping completed successfully");
                log::debug!("HeirCreateForm: Refreshing database heirs list");
                database_heirs.restart();
                navigator().push(Route::HeirListView {});
            }
            Err(e) => {
                log::info!("Heir seed striping failed: {e}");
                alert_error(format!("Heir seed striping failed: {e}"));
            }
        }

        *in_operation.write() = false;
    };

    use_drop(|| log::debug!("DeleteHeirSeedConfig Dropped"));

    rsx! {
        if uses_local_key() {
            details { class: "collapse collapse-arrow rounded-box border border-error shadow-md my-4 bg-error/5",

                summary { class: "collapse-title text-2xl font-bold text-error",
                    MaybeOnPathHighlight {
                        steps: &[OnboardingStep::CheckConfirmStripHeirSeed, OnboardingStep::StripHeirSeed],
                        context_filter: consume_onboarding_context(),
                        div { "Strip Heir Seed" }
                    }
                }
                div { class: "collapse-content p-4",

                    // Warnings and acknowledgments
                    div { class: "flex flex-col gap-4",
                        AlertDeleteKeyProvider { acknowledge_private_keys,
                            p {
                                "Deleting the seed will permanently remove all private keys from
                                this application. If you have not backed up the mnemonic phrase,
                                this heir private keys will be unrecoverable forever."
                            }
                        }
                    }

                    // Delete button
                    div { class: "flex justify-center mt-8",
                        MaybeHighlight {
                            step: OnboardingStep::StripHeirSeed,
                            context_filter: consume_onboarding_context(),
                            progress: MaybeHighlightProgressType::Signal(seed_stripped.into()),
                            button {
                                class: "btn btn-error btn-lg",
                                disabled: !acknowledge_private_keys() || in_operation(),
                                onclick: delete_heir_seed,
                                if in_operation() {
                                    span { class: "loading loading-spinner loading-sm mr-2" }
                                    "Deleting..."
                                } else {
                                    DrawSvg::<Delete> {}
                                    "Strip Heir Seed"
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}
