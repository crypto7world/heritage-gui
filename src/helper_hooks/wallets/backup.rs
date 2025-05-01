use dioxus::prelude::*;

use btc_heritage_wallet::{
    btc_heritage::HeritageWalletBackup, key_provider::MnemonicBackup, DatabaseItem, KeyProvider,
    OnlineWallet, Wallet,
};

use crate::{state_management, utils::wait_resource};

/// Resource hook for retrieving wallet descriptor backups
pub fn use_resource_wallet_descriptor_backup(
    wallet: Resource<Wallet>,
) -> Resource<Option<HeritageWalletBackup>> {
    use_resource(move || async move {
        log::debug!("use_resource_wallet_descriptor_backup - start");

        // Subscribe to the service connection
        let _ = *state_management::CONNECTED_USER.read();

        log::debug!("use_resource_wallet_descriptor_backup - waiting use_resource_wallet...");
        // Wait for wallet to finish
        wait_resource(wallet).await;
        log::debug!("use_resource_wallet_descriptor_backup - use_resource_wallet acquired");

        let backup = if let Some(ref wallet) = *wallet.read() {
            let wallet_name = wallet.name().to_owned();
            wallet
                .backup_descriptors()
                .await
                .map_err(|e| {
                    log::error!(
                        "Error retrieving the wallet descriptors backup of wallet {}: {e}",
                        wallet_name
                    );
                    ()
                })
                .ok()
        } else {
            unreachable!("wait_resource barrier ensures we can't go there")
        };
        log::debug!("use_resource_wallet_descriptor_backup - loaded");
        backup
    })
}

/// Resource hook for retrieving wallet mnemonic backup
pub fn use_resource_wallet_mnemonic_backup(
    wallet: Resource<Wallet>,
) -> Resource<Option<MnemonicBackup>> {
    use_resource(move || async move {
        log::debug!("use_resource_wallet_mnemonic_backup - start");

        log::debug!("use_resource_wallet_mnemonic_backup - waiting use_resource_wallet...");
        // Wait for wallet to finish
        wait_resource(wallet).await;
        log::debug!("use_resource_wallet_mnemonic_backup - use_resource_wallet acquired");

        let backup = if let Some(ref wallet) = *wallet.read() {
            let wallet_name = wallet.name().to_owned();
            wallet
                .backup_mnemonic()
                .await
                .map_err(|e| {
                    log::error!(
                        "Error retrieving the wallet mnemonic backup of wallet {}: {e}",
                        wallet_name
                    );
                    ()
                })
                .ok()
        } else {
            unreachable!("wait_resource barrier ensures we can't go there")
        };
        log::debug!("use_resource_wallet_mnemonic_backup - loaded");
        backup
    })
}
