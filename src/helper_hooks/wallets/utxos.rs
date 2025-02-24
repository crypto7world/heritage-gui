use dioxus::prelude::*;

use std::{collections::HashMap, sync::Arc};

use btc_heritage_wallet::{
    bitcoin::SignedAmount, btc_heritage::HeritageConfig, heritage_service_api_client::HeritageUtxo,
    DatabaseItem, OnlineWallet, Wallet,
};

use crate::{state_management, utils::wait_resource};

pub fn use_resource_wallet_utxos(wallet: Resource<Wallet>) -> Resource<Arc<[HeritageUtxo]>> {
    use_resource(move || async move {
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
                .into()
        } else {
            unreachable!("wait_resource barrier ensures we can't go there")
        };
        log::debug!("use_resource_wallet_utxos - loaded");
        wallet_utxos
    })
}
