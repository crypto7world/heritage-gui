use crate::prelude::*;

use btc_heritage_wallet::HeirWallet;

use crate::{
    components::{
        badge::{ExternalDependencyStatus, HeritageProviderType, KeyProviderType},
        delete::{AlertDeleteKeyProvider, AlertDeleteLocalWallet},
        svg::{Delete, DrawSvg},
    },
    Route,
};

/// Component for deleting an heir wallet with appropriate warnings and confirmations
///
/// This component provides a dangerous operation interface for heir wallet deletion.
/// It shows different warnings based on the heir wallet's configuration and requires
/// explicit user acknowledgment before allowing deletion.
#[component]
pub(super) fn DeleteHeirWalletConfig() -> Element {
    log::debug!("DeleteHeirWalletConfig Rendered");

    let database_service = state_management::use_database_service();
    let heirwallet = use_context::<AsyncSignal<HeirWallet>>();

    let keyprovider_status =
        use_context::<Memo<Option<(KeyProviderType, ExternalDependencyStatus)>>>();
    let online_status =
        use_context::<Memo<Option<(HeritageProviderType, ExternalDependencyStatus)>>>();

    let mut in_operation = use_signal(|| false);
    let acknowledge_private_keys = use_signal(|| false);
    let acknowledge_descriptors = use_signal(|| false);

    // Check if wallet uses LocalKey key provider
    let uses_local_key = use_memo(move || {
        keyprovider_status()
            .map(|(key_type, _)| matches!(key_type, KeyProviderType::LocalKey))
            .unwrap_or(false)
    });

    // Check if wallet uses Local online wallet
    let uses_local_heritage_provider = use_memo(move || {
        online_status()
            .map(|(online_type, _)| matches!(online_type, HeritageProviderType::LocalWallet))
            .unwrap_or(false)
    });

    // Determine if all required acknowledgments are checked
    let all_acknowledged = use_memo(move || {
        let mut required_checks = true;

        if uses_local_key() {
            required_checks = required_checks && acknowledge_private_keys();
        }

        if uses_local_heritage_provider() {
            required_checks = required_checks && acknowledge_descriptors();
        }

        required_checks
    });

    let delete_heirwallet = move |_| async move {
        *in_operation.write() = true;

        match heirwallet
            .with(async move |heirwallet| {
                state_management::delete_heirwallet(database_service, heirwallet).await
            })
            .await
        {
            Ok(()) => {
                log::info!("Heir Wallet deletion completed successfully");
                alert_success("Heir Wallet deletion completed successfully");
                navigator().push(Route::HeirWalletListView {});
            }
            Err(e) => {
                log::info!("Heir Wallet deletion failed: {e}");
                alert_error(format!("Heir Wallet deletion failed: {e}"));
            }
        }

        *in_operation.write() = false;
    };

    use_drop(|| log::debug!("DeleteHeirWalletConfig Dropped"));

    rsx! {

        details { class: "collapse collapse-arrow rounded-box border border-error shadow-md my-4 bg-error/5",
            summary { class: "collapse-title text-2xl font-bold text-error", "Delete Heir Wallet" }
            div { class: "collapse-content p-4",

                // Warnings and acknowledgments
                div { class: "flex flex-col gap-4",

                    // Private key warning for LocalKey wallets
                    if uses_local_key() {
                        AlertDeleteKeyProvider { acknowledge_private_keys,
                            p {
                                "This heir wallet uses local key storage.
                                Deleting the wallet will permanently remove all
                                private keys from this application. If you have not
                                backed up your mnemonic phrase, this heir wallet will
                                be unrecoverable forever."
                            }
                        }
                    }

                    // Descriptor warning for Local online wallets
                    if uses_local_heritage_provider() {
                        AlertDeleteLocalWallet { acknowledge_descriptors,
                            p {
                                "This heir wallet uses local storage for Bitcoin descriptors.
                                Deleting the heir wallet will permanently remove these descriptors.
                                These descriptors are mandatory for finding the inheritances you
                                are eligible for and spending them."
                            }
                        }
                    }
                }

                // Delete button
                div { class: "flex justify-center mt-8",
                    button {
                        class: "btn btn-error btn-lg",
                        disabled: !all_acknowledged() || in_operation(),
                        onclick: delete_heirwallet,
                        if in_operation() {
                            span { class: "loading loading-spinner loading-sm mr-2" }
                            "Deleting..."
                        } else {
                            DrawSvg::<Delete> {}
                            "Delete Heir Wallet"
                        }
                    }
                }
            }
        }
    }
}
