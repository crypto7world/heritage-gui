use crate::prelude::*;

use btc_heritage_wallet::{btc_heritage::HeritageWalletBackup, OnlineWallet, Wallet};

use crate::utils::log_error_ccstr;

/// Resource hook for retrieving wallet descriptor backups
pub fn use_resource_wallet_descriptor_backup(
    wallet: AsyncSignal<Wallet>,
) -> FResource<HeritageWalletBackup> {
    use_resource(move || async move {
        log::debug!("use_resource_wallet_descriptor_backup - start");

        let backup = wallet
            .with(async |wallet| wallet.backup_descriptors().await.map_err(log_error_ccstr))
            .await;
        log::debug!("use_resource_wallet_descriptor_backup - loaded");
        backup
    })
}
