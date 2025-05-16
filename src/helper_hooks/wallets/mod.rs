mod account_xpubs;
mod addresses;
mod backup;
mod heritage_configs;
mod status;
mod transactions;
mod utxos;

pub use account_xpubs::*;
pub use addresses::*;
pub use backup::*;
use btc_heritage_wallet::{
    heritage_service_api_client::HeritageWalletMeta, AnyOnlineWallet, Wallet,
};
pub use heritage_configs::*;
pub use status::*;
pub use transactions::*;
pub use utxos::*;

use dioxus::prelude::*;

use std::collections::HashSet;

use crate::{
    state_management::{self, use_database_service, use_service_client_service},
    utils::{ArcStr, ArcType},
};

pub fn use_resource_wallet_names() -> Resource<Vec<ArcStr>> {
    let database_service = use_database_service();
    use_resource(move || async move {
        log::debug!("use_resource_wallet_names - start");
        let wallet_names = state_management::list_wallet_names(database_service)
            .await
            .unwrap_or_default();
        log::debug!("use_resource_wallet_names - loaded");
        wallet_names
    })
}

pub fn use_resource_wallet(name: ArcStr) -> Resource<Wallet> {
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

pub fn use_resource_service_only_wallets() -> Resource<Vec<ArcType<HeritageWalletMeta>>> {
    let database_service = use_database_service();
    let service_client_service = use_service_client_service();
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

        // Subscribe to the service connection
        let _ = *state_management::CONNECTED_USER.read();
        let heritage_service =
            state_management::heritage_service_client(service_client_service).await;
        let service_wallets = heritage_service.list_wallets().await.unwrap_or_default();

        let service_only_wallet = service_wallets
            .into_iter()
            .filter_map(|w| (!db_wallet_ids.contains(w.id.as_str())).then(|| w.into()))
            .collect();
        log::debug!("use_resource_service_only_wallets - loaded");
        service_only_wallet
    })
}
