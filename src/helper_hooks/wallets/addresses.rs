use dioxus::prelude::*;

use btc_heritage_wallet::{
    btc_heritage::heritage_wallet::WalletAddress, DatabaseItem, OnlineWallet, Wallet,
};

use crate::{
    helper_hooks::async_init::AsyncSignal,
    state_management,
    utils::{wait_async_signal, ArcType},
};

/// Resource hook for retrieving all addresses associated with a wallet
///
/// # Examples
///
/// ```
/// let wallet = use_resource_wallet("my_wallet".into());
/// let addresses = use_resource_wallet_addresses(wallet);
/// ```
pub fn use_resource_wallet_addresses(
    wallet: AsyncSignal<Wallet>,
) -> Resource<ArcType<[WalletAddress]>> {
    use_resource(move || async move {
        log::debug!("use_resource_wallet_addresses - start");

        // Subscribe to the service connection
        let _ = *state_management::CONNECTED_USER.read();

        log::debug!("use_resource_wallet_addresses - waiting use_async_wallet...");
        // Wait for wallet to finish
        wait_async_signal(wallet).await;
        log::debug!("use_resource_wallet_addresses - use_async_wallet acquired");

        let wallet_addresses = if let Some(ref wallet) = *wallet.read() {
            let wallet_name = wallet.name().to_owned();
            wallet
                .list_addresses()
                .await
                .map_err(|e| {
                    log::error!(
                        "Error retrieving the wallet addresses of wallet {}: {e}",
                        wallet_name
                    );
                    ()
                })
                .unwrap_or_default()
                .into()
        } else {
            unreachable!("wait_resource barrier ensures we can't go there")
        };
        log::debug!("use_resource_wallet_addresses - loaded");
        wallet_addresses
    })
}

/// Resource hook for retrieving the next available address for a wallet
///
/// # Examples
///
/// ```
/// let wallet = use_resource_wallet("my_wallet".into());
/// let address = use_resource_wallet_address(wallet);
/// ```
pub fn use_resource_wallet_address(wallet: AsyncSignal<Wallet>) -> Resource<Option<String>> {
    use_resource(move || async move {
        log::debug!("use_resource_wallet_address - start");

        // Subscribe to the service connection
        let _ = *state_management::CONNECTED_USER.read();

        log::debug!("use_resource_wallet_address - waiting use_async_wallet...");
        // Wait for wallet to finish
        wait_async_signal(wallet).await;
        log::debug!("use_resource_wallet_address - use_async_wallet acquired");

        let wallet_address = if let Some(ref wallet) = *wallet.read() {
            let wallet_name = wallet.name().to_owned();
            wallet
                .get_address()
                .await
                .map_err(|e| {
                    log::error!(
                        "Error retrieving the next address of wallet {}: {e}",
                        wallet_name
                    );
                    ()
                })
                .ok()
        } else {
            unreachable!("wait_resource barrier ensures we can't go there")
        };
        log::debug!("use_resource_wallet_address - loaded");
        wallet_address
    })
}
