use crate::prelude::*;

use std::collections::BTreeMap;

use btc_heritage_wallet::{
    btc_heritage::AccountXPubId, online_wallet::WalletStatus, AnyOnlineWallet, BoundFingerprint,
    DatabaseItem, LedgerPolicy, OnlineWallet, Wallet,
};

use crate::{
    components::badge::{ExternalDependencyStatus, KeyProviderType, OnlineWalletType},
    state_management::prelude::*,
    utils::CCStr,
};

pub fn use_resource_wallet_status(wallet: AsyncSignal<Wallet>) -> FResource<WalletStatus> {
    use_resource(move || async move {
        log::debug!("use_resource_wallet_status - start");

        let wallet_status = wallet
            .with(async |wallet| {
                wallet.get_wallet_status().await.map_err(|e| {
                    log::error!(
                        "Error retrieving the wallet status of wallet {}: {e}",
                        wallet.name()
                    );
                    CCStr::from(e.to_string())
                })
            })
            .await;

        log::debug!("use_resource_wallet_status - loaded");

        wallet_status
    })
}

pub fn use_memo_fingerprint(wallet: AsyncSignal<Wallet>) -> Memo<CCStr> {
    use_memo(move || {
        log::debug!("use_memo_fingerprint - start compute");
        let fingerprint = wallet
            .lmap(|wallet| {
                wallet
                    .fingerprint()
                    .map(|fg| fg.to_string().into())
                    .unwrap_or_else(|e| {
                        log::warn!("{e}");
                        "-".into()
                    })
            })
            .unwrap_or_else(|| CCStr::from("Loading..."));
        log::debug!("use_memo_fingerprint - finish compute");
        fingerprint
    })
}

pub fn use_memo_wallet_online_status(
    wallet: AsyncSignal<Wallet>,
) -> Memo<Option<(OnlineWalletType, ExternalDependencyStatus)>> {
    use_memo(move || {
        log::debug!("use_memo_wallet_online_status - start compute");
        let result = wallet.lmap(|wallet| match wallet.online_wallet() {
            AnyOnlineWallet::None => (OnlineWalletType::None, ExternalDependencyStatus::None),
            AnyOnlineWallet::Service(sb) => (
                OnlineWalletType::Service,
                match state_management::SERVICE_STATUS.read().as_ref() {
                    Some(ss) if ss.can_serve_wallet(sb) => ExternalDependencyStatus::Available,
                    _ => ExternalDependencyStatus::Unavailable,
                },
            ),
            AnyOnlineWallet::Local(_) => (
                OnlineWalletType::Local,
                match state_management::BLOCKCHAIN_PROVIDER_STATUS() {
                    Some(BlockchainProviderStatus::Connected(_)) => {
                        ExternalDependencyStatus::Available
                    }
                    _ => ExternalDependencyStatus::Unavailable,
                },
            ),
        });
        log::debug!("use_memo_wallet_online_status - finish compute");
        result
    })
}
pub fn use_memo_wallet_keyprovider_status(
    wallet: AsyncSignal<Wallet>,
    ledger_unregistered_policies: Option<
        Memo<Option<Result<BTreeMap<AccountXPubId, LedgerPolicy>, CCStr>>>,
    >,
) -> Memo<Option<(KeyProviderType, ExternalDependencyStatus)>> {
    use_memo(move || {
        let ledger_has_unregistered_policies = match ledger_unregistered_policies {
            Some(ledger_unregistered_policies) => {
                Some(match ledger_unregistered_policies.read().as_ref() {
                    Some(Ok(lup)) => !lup.is_empty(),
                    _ => false,
                })
            }
            None => None,
        };
        log::debug!("use_memo_wallet_keyprovider_status - start compute");
        let result = wallet.lmap(|wallet| {
            super::super::utils::keyprovider_status(
                wallet.key_provider(),
                ledger_has_unregistered_policies,
            )
        });
        log::debug!("use_memo_wallet_keyprovider_status - finish compute");
        result
    })
}
