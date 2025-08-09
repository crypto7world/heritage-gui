use super::*;

pub async fn get_heirwallet(
    database_service: Coroutine<DatabaseCommand>,
    service_client_service: Coroutine<ServiceClientCommand>,
    name: CCStr,
) -> Result<HeirWallet, String> {
    log::debug!("get_heirwallet({name}) - start");
    let database = get_database(database_service).await;

    let mut heirwallet = HeirWallet::load(&database, name.as_ref()).map_err(log_error)?;

    match heirwallet.heritage_provider_mut() {
        AnyHeritageProvider::None => (),
        AnyHeritageProvider::Service(service_binding) => {
            let service_client = heritage_service_client(service_client_service).await;
            service_binding.init_service_client(service_client);
        }
        AnyHeritageProvider::LocalWallet(local_wallet) => {
            local_wallet
                .local_heritage_wallet_mut()
                .init_heritage_wallet(database)
                .await
                .map_err(log_error)?;
        }
    };

    log::debug!("get_heirwallet({name}) - loaded");

    Ok(heirwallet)
}

pub async fn delete_heirwallet(
    database_service: Coroutine<DatabaseCommand>,
    heirwallet: &HeirWallet,
) -> Result<(), String> {
    log::debug!("delete_heirwallet({heirwallet:?}) - start");
    super::database::delete_dbitem(database_service, heirwallet).await?;
    log::debug!("delete_heirwallet({heirwallet:?}) - finished");
    Ok(())
}
