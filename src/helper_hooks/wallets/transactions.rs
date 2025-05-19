use dioxus::prelude::*;

use btc_heritage_wallet::{
    heritage_service_api_client::TransactionSummary, DatabaseItem, OnlineWallet, Wallet,
};

use crate::{
    helper_hooks::async_init::AsyncSignal,
    state_management,
    utils::{wait_async_signal, ArcType},
};

pub fn use_resource_wallet_transactions(
    wallet: AsyncSignal<Wallet>,
) -> Resource<ArcType<[TransactionSummary]>> {
    use_resource(move || async move {
        log::debug!("use_resource_wallet_transactions - start");

        // Subscribe to the service connection
        let _ = *state_management::CONNECTED_USER.read();

        log::debug!("use_resource_wallet_transactions - waiting use_async_wallet...");
        // Wait for wallet to finish
        wait_async_signal(wallet).await;
        log::debug!("use_resource_wallet_transactions - use_async_wallet acquired");

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
                .into()
        } else {
            unreachable!("wait_resource barrier ensures we can't go there")
        };
        log::debug!("use_resource_wallet_transactions - loaded");
        wallet_txs
    })
}
