use dioxus::prelude::*;

use btc_heritage_wallet::{
    heritage_service_api_client::AccountXPubWithStatus, DatabaseItem, OnlineWallet, Wallet,
};

use crate::{
    state_management,
    utils::{wait_resource, ArcType},
};

/// Resource hook for fetching account extended public keys for a wallet
pub fn use_resource_wallet_account_xpubs(
    wallet: Resource<Wallet>,
) -> Resource<ArcType<[AccountXPubWithStatus]>> {
    use_resource(move || async move {
        log::debug!("use_resource_wallet_account_xpubs - start");

        // Subscribe to the service connection
        let _ = *state_management::CONNECTED_USER.read();

        log::debug!("use_resource_wallet_account_xpubs - waiting use_resource_wallet...");
        // Wait for wallet to finish
        wait_resource(wallet).await;
        log::debug!("use_resource_wallet_account_xpubs - use_resource_wallet acquired");

        let account_xpubs = if let Some(ref wallet) = *wallet.read() {
            let wallet_name = wallet.name().to_owned();
            wallet
                .list_account_xpubs()
                .await
                .map_err(|e| {
                    log::error!(
                        "Error retrieving the account XPubs of wallet {}: {e}",
                        wallet_name
                    );
                    ()
                })
                .unwrap_or_default()
                .into()
        } else {
            unreachable!("wait_resource barrier ensures we can't go there")
        };
        log::debug!("use_resource_wallet_account_xpubs - loaded");

        // tokio::time::sleep(std::time::Duration::from_secs(5)).await;

        account_xpubs
    })
}
