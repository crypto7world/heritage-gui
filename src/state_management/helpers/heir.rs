use super::*;

pub async fn get_heir(
    database_service: Coroutine<DatabaseCommand>,
    name: CCStr,
) -> Result<Heir, String> {
    log::debug!("get_heir({name}) - start");
    let database = get_database(database_service).await;

    let heir = Heir::load(&database, name.as_ref()).map_err(log_error)?;

    log::debug!("get_heir({name}) - loaded");

    Ok(heir)
}

pub async fn delete_heir(
    database_service: Coroutine<DatabaseCommand>,
    heir: &Heir,
) -> Result<(), String> {
    log::debug!("delete_heir({heir:?}) - start");
    super::database::delete_dbitem(database_service, heir).await?;
    log::debug!("delete_heir({heir:?}) - finished");
    Ok(())
}

pub async fn strip_heir_seed(
    database_service: Coroutine<DatabaseCommand>,
    heir: &Heir,
) -> Result<(), String> {
    log::debug!("strip_heir_seed({heir:?}) - start");
    let heir_name = heir.name().to_owned();
    get_database(database_service)
        .await
        .blocking_operation(move |mut db| {
            let mut heir = Heir::load(&db, &heir_name)?;
            heir.strip_key_provider();
            heir.save(&mut db)
        })
        .await
        .map_err(log_error)?;
    log::debug!("strip_heir_seed({heir:?}) - finished");
    Ok(())
}
