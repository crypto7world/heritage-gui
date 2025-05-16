use dioxus::prelude::*;

use btc_heritage_wallet::{
    online_wallet::WalletStatus, AnyKeyProvider, AnyOnlineWallet, BoundFingerprint, DatabaseItem,
    OnlineWallet, Wallet,
};

use crate::{
    components::loaded::badge::{ExternalDependencyStatus, KeyProviderType, OnlineWalletType},
    state_management,
    utils::wait_resource,
};

pub fn use_resource_wallet_status(
    wallet: Resource<Wallet>,
) -> Resource<Result<WalletStatus, String>> {
    use_resource(move || async move {
        log::debug!("use_resource_wallet_status - start");

        // Subscribe to the service connection
        let _ = *state_management::CONNECTED_USER.read();

        log::debug!("use_resource_wallet_status - waiting use_resource_wallet...");
        // Wait for wallet to finish
        wait_resource(wallet).await;
        log::debug!("use_resource_wallet_status - use_resource_wallet acquired");

        let wallet_status = if let Some(ref wallet) = *wallet.read() {
            wallet.get_wallet_status().await.map_err(|e| {
                log::error!(
                    "Error retrieving the wallet status of wallet {}: {e}",
                    wallet.name()
                );
                e.to_string()
            })
        } else {
            unreachable!("wait_resource barrier ensures we can't go there")
        };

        log::debug!("use_resource_wallet_status - loaded");

        // tokio::time::sleep(std::time::Duration::from_secs(5)).await;

        wallet_status
    })
}

pub fn use_memo_fingerprint(wallet: Resource<Wallet>) -> Memo<String> {
    use_memo(move || {
        log::debug!("use_memo_fingerprint - start compute");
        let fingerprint = if let Some(ref wallet) = *wallet.read() {
            wallet
                .fingerprint()
                .map(|fg| fg.to_string())
                .unwrap_or_else(|e| {
                    log::warn!("{e}");
                    "-".to_owned()
                })
        } else {
            "-".to_owned()
        };
        log::debug!("use_memo_fingerprint - finish compute");
        fingerprint
    })
}

pub fn use_memo_online_status(
    wallet: Resource<Wallet>,
    wallet_status: Resource<Result<WalletStatus, String>>,
) -> Memo<Option<(OnlineWalletType, ExternalDependencyStatus)>> {
    use_memo(move || {
        log::debug!("use_memo_online_status - start compute");
        let result = match (&*wallet.read(), &*wallet_status.read()) {
            (Some(wallet), Some(wallet_status)) => Some((
                match wallet.online_wallet() {
                    AnyOnlineWallet::None => OnlineWalletType::None,
                    AnyOnlineWallet::Service(_) => OnlineWalletType::Service,
                    AnyOnlineWallet::Local(_) => OnlineWalletType::Local,
                },
                match wallet_status {
                    Ok(_) => ExternalDependencyStatus::Available,
                    Err(_) => ExternalDependencyStatus::Unavailable,
                },
            )),
            _ => None,
        };
        log::debug!("use_memo_online_status - finish compute");
        result
    })
}
pub fn use_memo_keyprovider_status(
    wallet: Resource<Wallet>,
    wallet_status: Resource<Result<WalletStatus, String>>,
) -> Memo<Option<(KeyProviderType, ExternalDependencyStatus)>> {
    use_memo(move || {
        log::debug!("use_memo_keyprovider_status - start compute");
        let result = match (&*wallet.read(), &*wallet_status.read()) {
            (Some(wallet), Some(wallet_status)) => Some((
                match wallet.key_provider() {
                    AnyKeyProvider::None => KeyProviderType::None,
                    AnyKeyProvider::LocalKey(_) => KeyProviderType::LocalKey,
                    AnyKeyProvider::Ledger(_) => KeyProviderType::Ledger,
                },
                match wallet_status {
                    Ok(_) => ExternalDependencyStatus::Available,
                    Err(_) => ExternalDependencyStatus::Unavailable,
                },
            )),
            _ => None,
        };
        log::debug!("use_memo_keyprovider_status - finish compute");
        result
    })
}
