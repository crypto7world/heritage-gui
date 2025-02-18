use std::rc::Rc;

use btc_heritage_wallet::{
    bitcoin::Amount,
    btc_heritage::HeritageConfig,
    heritage_service_api_client::{HeritageUtxo, TransactionSummary},
    online_wallet::WalletStatus,
    BoundFingerprint, DatabaseItem, OnlineWallet, Wallet,
};
use dioxus::prelude::*;

use crate::{
    state_management,
    utils::{amount_to_string, timestamp_to_string, wait_resource},
};

pub fn use_resource_wallet_names() -> dioxus::hooks::Resource<Vec<String>> {
    dioxus::hooks::use_resource(move || async move {
        log::debug!("use_resource_wallet_names - start");
        let wallet_names = state_management::list_wallet_names()
            .await
            .unwrap_or_default();
        log::debug!("use_resource_wallet_names - loaded");
        wallet_names
    })
}

pub fn use_resource_heir_names() -> dioxus::hooks::Resource<Vec<String>> {
    dioxus::hooks::use_resource(move || async move {
        log::debug!("use_resource_heir_names - start");
        let heir_names = state_management::list_heir_names()
            .await
            .unwrap_or_default();
        log::debug!("use_resource_heir_names - loaded");
        heir_names
    })
}

pub fn use_resource_heirwallet_names() -> dioxus::hooks::Resource<Vec<String>> {
    dioxus::hooks::use_resource(move || async move {
        log::debug!("use_resource_heir_names - start");
        let heir_wallet_names = state_management::list_heir_wallet_names()
            .await
            .unwrap_or_default();
        log::debug!("use_resource_heir_names - loaded");
        heir_wallet_names
    })
}

pub fn use_resource_wallet(name: String) -> dioxus::hooks::Resource<Wallet> {
    dioxus::hooks::use_resource(move || {
        let name = name.clone();
        async move {
            log::debug!("use_resource_wallet - start");
            let wallet = state_management::get_wallet(name.as_str()).await.expect(
                "wallet should exist and I have nothing smart to do with this error anyway",
            );
            log::debug!("use_resource_wallet - loaded");
            wallet
        }
    })
}

pub fn use_resource_wallet_status(
    wallet: dioxus::hooks::Resource<Wallet>,
) -> dioxus::hooks::Resource<Option<WalletStatus>> {
    dioxus::hooks::use_resource(move || async move {
        log::debug!("use_resource_wallet_status - start");

        // Subscribe to the service connection
        let _ = *state_management::CONNECTED_USER.read();

        log::debug!("use_resource_wallet_status - waiting use_resource_wallet...");
        // Wait for wallet to finish
        wait_resource(wallet).await;
        log::debug!("use_resource_wallet_status - use_resource_wallet acquired");

        let wallet_status = if let Some(ref wallet) = *wallet.read() {
            let wallet_name = wallet.name().to_owned();
            wallet
                .get_wallet_status()
                .await
                .map_err(|e| {
                    log::error!(
                        "Error retrieving the wallet status of wallet {}: {e}",
                        wallet_name
                    );
                    ()
                })
                .ok()
        } else {
            unreachable!("wait_resource barrier ensures we can't go there")
        };
        log::debug!("use_resource_wallet_status - loaded");
        wallet_status
    })
}

pub fn use_resource_wallet_transactions(
    wallet: dioxus::hooks::Resource<Wallet>,
) -> dioxus::hooks::Resource<Vec<TransactionSummary>> {
    dioxus::hooks::use_resource(move || async move {
        log::debug!("use_resource_wallet_transactions - start");

        // Subscribe to the service connection
        let _ = *state_management::CONNECTED_USER.read();

        log::debug!("use_resource_wallet_transactions - waiting use_resource_wallet...");
        // Wait for wallet to finish
        wait_resource(wallet).await;
        log::debug!("use_resource_wallet_transactions - use_resource_wallet acquired");

        let wallet_txs = if let Some(ref wallet) = *wallet.read() {
            let wallet_name = wallet.name().to_owned();
            wallet
                .list_transactions()
                .await
                .map_err(|e| {
                    log::error!(
                        "Error retrieving the wallet transactions of wallet {}: {e}",
                        wallet_name
                    );
                    ()
                })
                .unwrap_or_default()
        } else {
            unreachable!("wait_resource barrier ensures we can't go there")
        };
        log::debug!("use_resource_wallet_transactions - loaded");
        wallet_txs
    })
}

pub fn use_resource_wallet_utxos(
    wallet: dioxus::hooks::Resource<Wallet>,
) -> dioxus::hooks::Resource<Vec<HeritageUtxo>> {
    dioxus::hooks::use_resource(move || async move {
        log::debug!("use_resource_wallet_utxos - start");

        // Subscribe to the service connection
        let _ = *state_management::CONNECTED_USER.read();

        log::debug!("use_resource_wallet_utxos - waiting use_resource_wallet...");
        // Wait for wallet to finish
        wait_resource(wallet).await;
        log::debug!("use_resource_wallet_utxos - use_resource_wallet acquired");

        let wallet_utxos = if let Some(ref wallet) = *wallet.read() {
            let wallet_name = wallet.name().to_owned();
            wallet
                .list_heritage_utxos()
                .await
                .map_err(|e| {
                    log::error!(
                        "Error retrieving the wallet UTXOs of wallet {}: {e}",
                        wallet_name
                    );
                    ()
                })
                .unwrap_or_default()
        } else {
            unreachable!("wait_resource barrier ensures we can't go there")
        };
        log::debug!("use_resource_wallet_utxos - loaded");
        wallet_utxos
    })
}

pub fn use_resource_wallet_heritage_configs(
    wallet: dioxus::hooks::Resource<Wallet>,
) -> dioxus::hooks::Resource<Vec<HeritageConfig>> {
    dioxus::hooks::use_resource(move || async move {
        log::debug!("use_resource_wallet_heritage_configs - start");

        // Subscribe to the service connection
        let _ = *state_management::CONNECTED_USER.read();

        log::debug!("use_resource_wallet_heritage_configs - waiting use_resource_wallet...");
        // Wait for wallet to finish
        wait_resource(wallet).await;
        log::debug!("use_resource_wallet_heritage_configs - use_resource_wallet acquired");

        let wallet_heritage_configs = if let Some(ref wallet) = *wallet.read() {
            let wallet_name = wallet.name().to_owned();
            wallet
                .list_heritage_configs()
                .await
                .map_err(|e| {
                    log::error!(
                        "Error retrieving the wallet HeritageConfigs of wallet {}: {e}",
                        wallet_name
                    );
                    ()
                })
                .unwrap_or_default()
        } else {
            unreachable!("wait_resource barrier ensures we can't go there")
        };
        log::debug!("use_resource_wallet_heritage_configs - loaded");
        wallet_heritage_configs
    })
}

pub fn use_memo_fingerprint(
    wallet: dioxus::hooks::Resource<Wallet>,
) -> dioxus::signals::Memo<String> {
    dioxus::hooks::use_memo(move || {
        log::debug!("use_memo_fingerprint - compute");
        if let Some(ref wallet) = *wallet.read() {
            wallet
                .fingerprint()
                .map(|fg| fg.to_string())
                .unwrap_or_else(|e| {
                    log::warn!("{e}");
                    "-".to_owned()
                })
        } else {
            "-".to_owned()
        }
    })
}
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BalanceStrings {
    pub balance: Rc<str>,
    pub cur_balance: Rc<str>,
    pub obs_balance: Rc<str>,
}
impl Default for BalanceStrings {
    fn default() -> Self {
        Self {
            balance: "12345 BTC".into(),
            cur_balance: "12345 BTC".into(),
            obs_balance: "0 sat".into(),
        }
    }
}
pub fn use_memo_balance_strings(
    wallet_status: dioxus::hooks::Resource<Option<WalletStatus>>,
) -> dioxus::signals::Memo<Option<BalanceStrings>> {
    dioxus::hooks::use_memo(move || {
        log::debug!("use_memo_balance_strings - compute");
        match &*wallet_status.read() {
            Some(Some(ws)) => {
                let balance =
                    amount_to_string(Amount::from_sat(ws.balance.total_balance().get_total()))
                        .into();
                let cur_balance =
                    amount_to_string(Amount::from_sat(ws.balance.uptodate_balance().get_total()))
                        .into();
                let obs_balance =
                    amount_to_string(Amount::from_sat(ws.balance.obsolete_balance().get_total()))
                        .into();
                Some(BalanceStrings {
                    balance,
                    cur_balance,
                    obs_balance,
                })
            }
            Some(None) => {
                let ph: Rc<str> = "-".into();
                Some(BalanceStrings {
                    balance: ph.clone(),
                    cur_balance: ph.clone(),
                    obs_balance: ph,
                })
            }
            _ => None,
        }
    })
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LastSynced(pub Rc<str>);
impl Default for LastSynced {
    fn default() -> Self {
        Self("1970-01-01 00:00:00 UTC".into())
    }
}
pub fn use_memo_last_sync(
    wallet_status: dioxus::hooks::Resource<Option<WalletStatus>>,
) -> dioxus::signals::Memo<Option<LastSynced>> {
    dioxus::hooks::use_memo(move || {
        log::debug!("use_memo_last_sync - compute");
        match &*wallet_status.read() {
            Some(Some(ws)) => {
                let last_synced = timestamp_to_string(ws.last_sync_ts).into();
                Some(LastSynced(last_synced))
            }
            Some(None) => Some(LastSynced("-".into())),
            _ => None,
        }
    })
}
