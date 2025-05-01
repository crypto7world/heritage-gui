use dioxus::prelude::*;

use btc_heritage_wallet::{btc_heritage::HeritageConfig, DatabaseItem, OnlineWallet, Wallet};

use crate::{
    state_management,
    utils::{wait_resource, RcType},
};

pub fn use_resource_wallet_heritage_configs(
    wallet: Resource<Wallet>,
) -> Resource<RcType<[RcType<HeritageConfig>]>> {
    use_resource(move || async move {
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
                .into_iter()
                .map(RcType::new)
                .collect()
        } else {
            unreachable!("wait_resource barrier ensures we can't go there")
        };
        log::debug!("use_resource_wallet_heritage_configs - loaded");
        wallet_heritage_configs
    })
}
