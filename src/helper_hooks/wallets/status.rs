use dioxus::prelude::*;

use btc_heritage_wallet::{
    bitcoin::SignedAmount, btc_heritage::HeritageWalletBalance, online_wallet::WalletStatus,
    BoundFingerprint, DatabaseItem, OnlineWallet, Wallet,
};

use crate::{
    components::{misc::DisplayTimestamp, wallet::DisplayBtcAmount},
    state_management::{self, use_database_service},
    utils::{wait_resource, LoadedElement, RcStr},
};

pub fn use_resource_wallet(name: RcStr) -> Resource<Wallet> {
    let database_service = use_database_service();
    use_resource(move || {
        let name = name.clone();
        async move {
            log::debug!("use_resource_wallet - start");
            let wallet = state_management::get_wallet(database_service, name)
                .await
                .expect(
                    "wallet should exist and I have nothing smart to do with this error anyway",
                );
            log::debug!("use_resource_wallet - loaded");
            wallet
        }
    })
}

pub fn use_resource_wallet_status(wallet: Resource<Wallet>) -> Resource<Option<WalletStatus>> {
    use_resource(move || async move {
        log::debug!("use_resource_wallet_status - start");

        // Subscribe to the service connection
        let _ = *state_management::CONNECTED_USER.read();

        log::debug!("use_resource_wallet_status - waiting use_resource_wallet...");
        // Wait for wallet to finish
        wait_resource(wallet).await;
        log::debug!("use_resource_wallet_status - use_resource_wallet acquired");

        // tokio::time::sleep(std::time::Duration::from_secs(5)).await;

        let wallet_status = if let Some(ref wallet) = *wallet.read() {
            wallet
                .get_wallet_status()
                .await
                .map_err(|e| {
                    log::error!(
                        "Error retrieving the wallet status of wallet {}: {e}",
                        wallet.name()
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

#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct Balances {
    pub balance: LoadedElement<DisplayBtcAmount>,
    pub cur_balance: LoadedElement<DisplayBtcAmount>,
    pub obs_balance: LoadedElement<DisplayBtcAmount>,
}

impl From<&HeritageWalletBalance> for Balances {
    fn from(value: &HeritageWalletBalance) -> Self {
        let balance = SignedAmount::from_sat(value.total_balance().get_total() as i64);
        let cur_balance = SignedAmount::from_sat(value.uptodate_balance().get_total() as i64);
        let obs_balance = SignedAmount::from_sat(value.obsolete_balance().get_total() as i64);
        Balances {
            balance: LoadedElement::Loaded(balance.into()),
            cur_balance: LoadedElement::Loaded(cur_balance.into()),
            obs_balance: LoadedElement::Loaded(obs_balance.into()),
        }
    }
}
impl From<Option<&WalletStatus>> for Balances {
    fn from(value: Option<&WalletStatus>) -> Self {
        value.map(|ws| Self::from(&ws.balance)).unwrap_or_default()
    }
}

pub fn use_memo_display_balances(wallet_status: Resource<Option<WalletStatus>>) -> Memo<Balances> {
    use_memo(move || {
        log::debug!("use_memo_display_balances - start compute");
        let balances = match &*wallet_status.read() {
            Some(ows) => Balances::from(ows.as_ref()),
            None => Balances {
                balance: LoadedElement::Loading,
                cur_balance: LoadedElement::Loading,
                obs_balance: LoadedElement::Loading,
            },
        };
        log::debug!("use_memo_display_balances - finish compute");
        balances
    })
}

pub fn use_memo_last_sync(
    wallet_status: Resource<Option<WalletStatus>>,
) -> Memo<LoadedElement<DisplayTimestamp>> {
    use_memo(move || {
        log::debug!("use_memo_last_sync - start compute");
        let last_sync = match &*wallet_status.read() {
            Some(Some(ws)) => LoadedElement::Loaded(DisplayTimestamp::Ts(ws.last_sync_ts)),
            Some(None) => LoadedElement::Loaded(DisplayTimestamp::None),
            None => LoadedElement::Loading,
        };
        log::debug!("use_memo_last_sync - finish compute");
        last_sync
    })
}
