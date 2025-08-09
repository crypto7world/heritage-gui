use super::*;

pub async fn get_database(database_service: Coroutine<DatabaseCommand>) -> Database {
    log::debug!("get_database - start");
    let (result, rx) = oneshot::channel();
    database_service.send(DatabaseCommand::GetDatabase { result });
    let database = rx.await.expect("database_service error");
    log::debug!("get_database - loaded");
    database
}

pub async fn blocking_db_service_operation<
    R: Send + 'static,
    F: FnOnce(Database) -> R + Send + 'static,
>(
    database_service: Coroutine<DatabaseCommand>,
    f: F,
) -> R {
    get_database(database_service)
        .await
        .blocking_operation(f)
        .await
}

pub async fn list_wallet_names(
    database_service: Coroutine<DatabaseCommand>,
) -> Result<Vec<CCStr>, String> {
    log::debug!("list_wallet_names - start");
    let (result, rx) = oneshot::channel();
    database_service.send(DatabaseCommand::Wallet(DatabaseItemCommand::ListNames {
        result,
    }));
    let wallet_names = rx
        .await
        .expect("database_service error")
        .map(|names| names.into_iter().map(CCStr::from).collect())
        .map_err(log_error);
    log::debug!("list_wallet_names - loaded");
    wallet_names
}
pub async fn list_wallets(
    database_service: Coroutine<DatabaseCommand>,
) -> Result<Vec<CheapClone<Wallet>>, String> {
    log::debug!("list_wallets - start");
    let (result, rx) = oneshot::channel();
    database_service.send(DatabaseCommand::Wallet(DatabaseItemCommand::ListItems {
        result,
    }));
    let wallets = rx
        .await
        .expect("database_service error")
        .map(|names| names.into_iter().map(CheapClone::from).collect())
        .map_err(log_error);
    log::debug!("list_wallets - loaded");
    wallets
}

pub async fn list_heirs(
    database_service: Coroutine<DatabaseCommand>,
) -> Result<Vec<CheapClone<Heir>>, String> {
    log::debug!("list_heirs - start");
    let (result, rx) = oneshot::channel();
    database_service.send(DatabaseCommand::Heir(DatabaseItemCommand::ListItems {
        result,
    }));
    let heirs = rx
        .await
        .expect("database_service error")
        .map(|names| names.into_iter().map(CheapClone::from).collect())
        .map_err(log_error);
    log::debug!("list_heirs - loaded");
    heirs
}

pub async fn list_heirwallet_names(
    database_service: Coroutine<DatabaseCommand>,
) -> Result<Vec<CCStr>, String> {
    log::debug!("list_heirwallet_names - start");
    let (result, rx) = oneshot::channel();
    database_service.send(DatabaseCommand::HeirWallet(
        DatabaseItemCommand::ListNames { result },
    ));
    let heirwallet_names = rx
        .await
        .expect("database_service error")
        .map(|names| names.into_iter().map(CCStr::from).collect())
        .map_err(log_error);
    log::debug!("list_heirwallet_names - loaded");
    heirwallet_names
}
pub async fn list_heirwallets(
    database_service: Coroutine<DatabaseCommand>,
) -> Result<Vec<CheapClone<HeirWallet>>, String> {
    log::debug!("list_heirwallets - start");
    let (result, rx) = oneshot::channel();
    database_service.send(DatabaseCommand::HeirWallet(
        DatabaseItemCommand::ListItems { result },
    ));
    let heirwallets = rx
        .await
        .expect("database_service error")
        .map(|names| names.into_iter().map(CheapClone::from).collect())
        .map_err(log_error);
    log::debug!("list_heirwallets - loaded");
    heirwallets
}

pub async fn delete_dbitem<T: DatabaseItem + Send + 'static>(
    database_service: Coroutine<DatabaseCommand>,
    item: &T,
) -> Result<(), String> {
    // We cannot use item.delete(&mut db) because we would have to
    // take ownership of it in order to send it to the blocking thread.
    // So instead we fall back on calling db.delete_item directly on the item key
    // This allow to present a delete_dbitem interface that take an item reference
    let item_key = T::name_to_key(item.name());
    get_database(database_service)
        .await
        .blocking_operation(move |mut db| db.delete_item::<T>(&item_key))
        .await
        .map_err(log_error)?;

    Ok(())
}
