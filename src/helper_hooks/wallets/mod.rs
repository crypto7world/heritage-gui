use crate::prelude::*;

mod account_xpubs;
mod addresses;
mod backup;
mod heritage_configs;
mod ledger;
mod status;
mod transactions;
mod utxos;

pub use account_xpubs::*;
pub use addresses::*;
pub use backup::*;
pub use heritage_configs::*;
pub use ledger::*;
pub use status::*;
pub use transactions::*;
pub use utxos::*;

use std::collections::HashSet;

use btc_heritage_wallet::{
    heritage_service_api_client::HeritageWalletMeta, AnyOnlineWallet, Wallet,
};

use crate::utils::{CCStr, CheapClone};

use super::{
    async_init::{use_async_init, AsyncSignal},
    utils::LoadableMapper,
};

fn subscribe_service_status_if_service_wallet(wallet: &AsyncSignal<Wallet>) {
    if let Some(ref wallet) = *wallet.read() {
        if matches!(wallet.online_wallet(), AnyOnlineWallet::Service(_)) {
            // Read the SERVICE_STATUS so that we refresh when the SERVICE_STATUS is refreshed
            let _ = *state_management::SERVICE_STATUS.read();
        }
    }
}

pub fn use_resource_wallet_names() -> Resource<Vec<CCStr>> {
    let database_service = state_management::use_database_service();
    use_resource(move || async move {
        log::debug!("use_resource_wallet_names - start");
        let wallet_names = state_management::list_wallet_names(database_service)
            .await
            .unwrap_or_default();
        log::debug!("use_resource_wallet_names - loaded");
        wallet_names
    })
}

pub fn use_async_wallet(name: CCStr) -> AsyncSignal<Wallet> {
    let database_service = state_management::use_database_service();
    let service_client_service = state_management::use_service_client_service();
    let blockchain_provider_service = state_management::use_blockchain_provider_service();
    use_async_init(move || {
        let name = name.clone();
        async move {
            log::debug!("use_async_wallet - start");
            let wallet = state_management::get_wallet(
                database_service,
                service_client_service,
                blockchain_provider_service,
                name,
            )
            .await
            .expect("should exist and I have nothing smart to do with this error anyway");
            log::debug!("use_async_wallet - loaded");
            wallet
        }
    })
}
pub fn use_resource_service_wallets() -> Resource<Vec<CheapClone<HeritageWalletMeta>>> {
    let service_client_service = state_management::use_service_client_service();
    use_resource(move || async move {
        log::debug!("use_resource_service_wallets - start");

        // Read the SERVICE_STATUS so that we refresh when the SERVICE_STATUS is refreshed
        let _ = *state_management::SERVICE_STATUS.read();

        let heritage_service =
            state_management::heritage_service_client(service_client_service).await;
        let service_wallets = heritage_service.list_wallets().await.unwrap_or_default();

        let service_wallets = service_wallets.into_iter().map(|w| w.into()).collect();
        log::debug!("use_resource_service_wallets - loaded");
        service_wallets
    })
}

pub fn use_resource_service_only_wallets() -> Resource<Vec<CheapClone<HeritageWalletMeta>>> {
    let database_service = state_management::use_database_service();
    let service_wallets = use_resource_service_wallets();
    use_resource(move || async move {
        log::debug!("use_resource_service_only_wallets - start");

        let db_wallets = state_management::list_wallets(database_service)
            .await
            .unwrap_or_default();
        let db_wallet_ids = db_wallets
            .iter()
            .filter_map(|wallet| match wallet.online_wallet() {
                AnyOnlineWallet::Service(sb) => Some(sb.wallet_id()),
                _ => None,
            })
            .collect::<HashSet<_>>();

        let service_only_wallet = service_wallets
            .lmap(|service_wallets| {
                service_wallets
                    .iter()
                    .filter(|&w| !db_wallet_ids.contains(w.id.as_str()))
                    .cloned()
                    .collect()
            })
            .unwrap_or_default();
        log::debug!("use_resource_service_only_wallets - loaded");
        service_only_wallet
    })
}
