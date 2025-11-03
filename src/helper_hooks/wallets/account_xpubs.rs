use crate::prelude::*;

use btc_heritage_wallet::{
    heritage_service_api_client::AccountXPubWithStatus, DatabaseItem, OnlineWallet, Wallet,
};

use crate::utils::{CCStr, CheapClone};

/// Resource hook for fetching account extended public keys for a wallet
pub fn use_resource_wallet_account_xpubs(
    wallet: AsyncSignal<Wallet>,
) -> FResource<CheapClone<[AccountXPubWithStatus]>> {
    use_resource(move || async move {
        log::debug!("use_resource_wallet_account_xpubs - start");

        super::subscribe_service_status_if_service_wallet(&wallet);

        let account_xpubs = wallet
            .with(async |wallet| {
                let wallet_name = wallet.name().to_owned();
                wallet
                    .list_account_xpubs()
                    .await
                    .map_err(|e| {
                        let error = format!(
                            "Error retrieving the account XPubs of wallet {}: {e}",
                            wallet_name
                        );
                        log::error!("{error}");
                        CCStr::from(error)
                    })
                    .map(Into::into)
            })
            .await;
        log::debug!("use_resource_wallet_account_xpubs - loaded");

        account_xpubs
    })
}
