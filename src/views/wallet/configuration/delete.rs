use crate::prelude::*;

use btc_heritage_wallet::Wallet;

use crate::{
    components::{
        badge::ExternalDependencyStatus,
        delete::{
            AlertDeleteAck, AlertDeleteAckStyle, AlertDeleteKeyProvider, AlertDeleteLocalWallet,
        },
        svg::{Delete, DrawSvg, InfoCircleOutline},
    },
    views::wallet::{KeyProviderType, OnlineWalletType},
    Route,
};

/// Component for deleting a wallet with appropriate warnings and confirmations
///
/// This component provides a dangerous operation interface for wallet deletion.
/// It shows different warnings based on the wallet's configuration and requires
/// explicit user acknowledgment before allowing deletion.
#[component]
pub(super) fn DeleteWalletConfig() -> Element {
    log::debug!("DeleteWalletConfig Rendered");

    let database_service = state_management::use_database_service();
    let wallet = use_context::<AsyncSignal<Wallet>>();

    let keyprovider_status =
        use_context::<Memo<Option<(KeyProviderType, ExternalDependencyStatus)>>>();
    let online_status = use_context::<Memo<Option<(OnlineWalletType, ExternalDependencyStatus)>>>();

    let mut in_operation = use_signal(|| false);
    let acknowledge_private_keys = use_signal(|| false);
    let acknowledge_descriptors = use_signal(|| false);
    let acknowledge_service_warning = use_signal(|| false);
    let acknowledge_ledger_warning = use_signal(|| false);

    // Check if wallet uses LocalKey key provider
    let uses_local_key = use_memo(move || {
        keyprovider_status()
            .map(|(key_type, _)| matches!(key_type, KeyProviderType::LocalKey))
            .unwrap_or(false)
    });

    // Check if wallet uses Ledger
    let uses_ledger_key = use_memo(move || {
        keyprovider_status()
            .map(|(key_type, _)| matches!(key_type, KeyProviderType::Ledger))
            .unwrap_or(false)
    });

    // Check if wallet uses Local online wallet
    let uses_local_online = use_memo(move || {
        online_status()
            .map(|(online_type, _)| matches!(online_type, OnlineWalletType::Local))
            .unwrap_or(false)
    });

    // Check if wallet uses Service online wallet
    let uses_service_online = use_memo(move || {
        online_status()
            .map(|(online_type, _)| matches!(online_type, OnlineWalletType::Service))
            .unwrap_or(false)
    });

    // Determine if all required acknowledgments are checked
    let all_acknowledged = use_memo(move || {
        let mut required_checks = true;

        if uses_local_key() {
            required_checks = required_checks && acknowledge_private_keys();
        }

        if uses_ledger_key() {
            required_checks = required_checks && acknowledge_ledger_warning();
        }

        if uses_local_online() {
            required_checks = required_checks && acknowledge_descriptors();
        }

        if uses_service_online() {
            required_checks = required_checks && acknowledge_service_warning();
        }

        required_checks
    });

    let delete_wallet = move |_| async move {
        *in_operation.write() = true;

        match wallet
            .with(async move |wallet| {
                state_management::delete_wallet(database_service, wallet).await
            })
            .await
        {
            Ok(()) => {
                log::info!("Wallet deletion completed successfully");
                alert_success("Wallet deletion completed successfully");
                navigator().push(Route::WalletListView {});
            }
            Err(e) => {
                log::info!("Wallet deletion failed: {e}");
                alert_error(format!("Wallet deletion failed: {e}"));
            }
        }

        *in_operation.write() = false;
    };

    use_drop(|| log::debug!("DeleteWalletConfig Dropped"));

    rsx! {
        details { class: "collapse collapse-arrow rounded-box border border-error shadow-md my-4 bg-error/5",
            summary { class: "collapse-title text-2xl font-bold text-error", "Delete Wallet" }
            div { class: "collapse-content p-4",

                // Backup recommendation alert
                div { class: "alert alert-info mb-6",
                    DrawSvg::<InfoCircleOutline> {}
                    div {
                        h3 { class: "font-bold", "Backup Recommendation" }
                        p {
                            "Deleting a wallet is a permanent and irreversible action. "
                            "Before deleting your wallet, we strongly recommend using the backup features available in the wallet main view. "
                            "These backups can help you recover your wallet in the future if needed."
                        }
                    }
                }

                // Warnings and acknowledgments
                div { class: "flex flex-col gap-4",

                    // Private key warning for LocalKey wallets
                    if uses_local_key() {
                        AlertDeleteKeyProvider { acknowledge_private_keys,
                            p {
                                "This wallet uses local key storage. Deleting the wallet will
                                permanently remove all private keys from this application.
                                If you have not backed up your mnemonic phrase,
                                your wallet will be unrecoverable forever."
                            }
                        }
                    }

                    // Descriptor warning for Local online wallets
                    if uses_local_online() {
                        AlertDeleteLocalWallet { acknowledge_descriptors,
                            p {
                                "This wallet uses local storage for Bitcoin descriptors.
                                Deleting the wallet will permanently remove these descriptors.
                                Heritage wallets use advanced Bitcoin scripts that require
                                these descriptors to properly sign transactions.
                                Without descriptor backups, your private keys "
                                span { class: "font-black uppercase", "are not" }
                                " sufficient to recover your funds."
                            }
                        }
                    }

                    // Ledger warning for Ledger key provider
                    if uses_ledger_key() {
                        AlertDeleteAck {
                            alert_style: AlertDeleteAckStyle::Warning,
                            acknowledge: acknowledge_ledger_warning,
                            acknowledgment: rsx! {
                                span { class: "font-black uppercase", "I understand" }
                                " that deleting this wallet will not affect my Ledger device
                                    and I can recreate the wallet using the same Ledger device."
                            },
                            h3 { class: "font-bold", "Ledger Device Notice" }
                            p {
                                "This wallet uses a Ledger hardware device for key management.
                                Deleting the wallet locally will NOT affect the Ledger device
                                or remove any keys from it.
                                Your Ledger device will continue to hold the private keys independently.
                                You can recreate this wallet in the future by connecting the same
                                Ledger device and importing the existing wallet configuration."
                            }
                        }
                    }

                    // Service warning for Service online wallets
                    if uses_service_online() {
                        AlertDeleteAck {
                            alert_style: AlertDeleteAckStyle::Warning,
                            acknowledge: acknowledge_service_warning,
                            acknowledgment: rsx! {
                                span { class: "font-black uppercase", "I understand" }
                                " that the online wallet copy on the Heritage Service will remain and I can bound it to a new wallet in the future."
                            },
                            h3 { class: "font-bold", "Service Wallet Notice" }
                            p {
                                "This wallet uses the Heritage Service for online operations. Deleting the wallet locally will NOT delete the wallet data on the service. "
                                "If you need to delete the service copy in the future, contact the service provider for assistance. "
                                "You can also recreate a local wallet and bind it to the existing service copy using the 'Bind Existing' option when creating a new wallet."
                            }
                        }
                    }
                }

                // Delete button
                div { class: "flex justify-center mt-8",
                    button {
                        class: "btn btn-error btn-lg",
                        disabled: !all_acknowledged() || in_operation(),
                        onclick: delete_wallet,
                        if in_operation() {
                            span { class: "loading loading-spinner loading-sm mr-2" }
                            "Deleting..."
                        } else {
                            DrawSvg::<Delete> {}
                            "Delete Wallet"
                        }
                    }
                }
            }
        }
    }
}
