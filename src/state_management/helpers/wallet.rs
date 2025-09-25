use std::sync::Arc;

use crate::prelude::alert_warn;

use super::*;

pub async fn init_wallet(
    database_service: Coroutine<DatabaseCommand>,
    service_client_service: Coroutine<ServiceClientCommand>,
    blockchain_provider_service: Coroutine<BlockchainProviderCommand>,
    mut wallet: Wallet,
) -> Result<Wallet, String> {
    log::debug!("init_wallet({wallet:?}) - start");

    match wallet.online_wallet_mut() {
        AnyOnlineWallet::None => (),
        AnyOnlineWallet::Service(service_binding) => {
            let service_client = heritage_service_client(service_client_service).await;

            // Because it is entirely possible that we are not connected to the service
            // when this is called or connected to the wrong account
            // We still give a service_client Singleton to the wallet so it is ready for when
            // the situation change
            //
            // SAFETY:
            // It is safe because we do not care at this stage if the service wallet is present or the correct one
            // This will only be important later
            unsafe {
                service_binding.init_service_client_unchecked(service_client);
            }
        }
        AnyOnlineWallet::Local(local_heritage_wallet) => {
            local_heritage_wallet
                .init_heritage_wallet(get_database(database_service).await)
                .await
                .map_err(log_error)?;
            match blockchain_factory(blockchain_provider_service).await {
                Ok(bcf) => local_heritage_wallet.init_blockchain_factory(bcf),
                Err(e) => {
                    log::warn!("{e}");
                    alert_warn(e);
                }
            };
        }
    };
    let wallet = match wallet.retry_fingerprints_control().await {
        Ok(true) => {
            // If it returned true, then an update was made, need to save.
            get_database(database_service)
                .await
                .blocking_operation(move |mut db| {
                    wallet.save(&mut db).map_err(log_error)?;
                    Ok::<_, String>(wallet)
                })
                .await?
        }
        Ok(false) => wallet,
        Err(e) => {
            log::warn!("Could not verify fingerprint: {e}");
            wallet
        }
    };
    log::debug!("init_wallet({wallet:?}) - finished");
    Ok(wallet)
}

pub async fn get_wallet(
    database_service: Coroutine<DatabaseCommand>,
    service_client_service: Coroutine<ServiceClientCommand>,
    blockchain_provider_service: Coroutine<BlockchainProviderCommand>,
    name: CCStr,
) -> Result<Wallet, String> {
    log::debug!("get_wallet({name}) - start");

    let wallet = {
        let name = name.to_string();
        get_database(database_service)
            .await
            .blocking_operation(move |db| Wallet::load(&db, &name))
            .await
            .map_err(log_error)?
    };

    log::debug!("get_wallet({name}) - loaded");

    init_wallet(
        database_service,
        service_client_service,
        blockchain_provider_service,
        wallet,
    )
    .await
}

pub async fn delete_wallet(
    database_service: Coroutine<DatabaseCommand>,
    wallet: &Wallet,
) -> Result<(), String> {
    log::debug!("delete_wallet({wallet:?}) - start");
    super::database::delete_dbitem(database_service, wallet).await?;
    log::debug!("delete_wallet({wallet:?}) - finished");
    Ok(())
}
pub async fn create_wallet(
    database_service: Coroutine<DatabaseCommand>,
    wallet: Wallet,
) -> Result<(), String> {
    log::debug!("create_wallet({wallet:?}) - start");

    get_database(database_service)
        .await
        .blocking_operation(move |mut db| wallet.create(&mut db))
        .await
        .map_err(log_error)?;

    log::debug!("create_wallet - finished");
    Ok(())
}

pub async fn save_wallet(
    database_service: Coroutine<DatabaseCommand>,
    wallet: Arc<Wallet>,
) -> Result<(), String> {
    log::debug!("save_wallet({wallet:?}) - start");

    get_database(database_service)
        .await
        .blocking_operation(move |mut db| wallet.save(&mut db))
        .await
        .map_err(log_error)?;

    log::debug!("save_wallet - finished");
    Ok(())
}
