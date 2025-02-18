mod blockchain;
mod config;
mod database;
mod service;

use btc_heritage_wallet::{
    heritage_service_api_client::DeviceAuthorizationResponse, AnyKeyProvider, AnyOnlineWallet,
    Wallet,
};
use dioxus::hooks::{use_coroutine, use_coroutine_handle};
use tokio::sync::oneshot;

use crate::utils::log_error;
use database::{DatabaseCommand, DatabaseItemCommand};
use service::ServiceClientCommand;

pub use service::CONNECTED_USER;

pub fn init_services() {
    log::debug!("init_services - start");
    use_coroutine(database::database_service);
    use_coroutine(service::service_client_service);
    log::debug!("init_services - finished");
}

pub async fn connect<F, Fut>(callback: F) -> Result<(), String>
where
    F: FnOnce(DeviceAuthorizationResponse) -> Fut + 'static,
    Fut: std::future::Future<
            Output = Result<(), btc_heritage_wallet::heritage_service_api_client::Error>,
        > + 'static,
{
    log::debug!("connect - start");
    let service_client_service = use_coroutine_handle::<ServiceClientCommand>();
    let (result, waiter) = oneshot::channel();
    service_client_service.send(ServiceClientCommand::Connect {
        callback: Box::new(|dar| Box::pin(callback(dar))),
        result,
    });
    let result = waiter
        .await
        .expect("service_client_service error")
        .map_err(log_error);

    log::debug!("connect - finished");
    result
}

pub async fn disconnect() -> Result<bool, String> {
    log::debug!("disconnect - start");
    let service_client_service = use_coroutine_handle::<ServiceClientCommand>();
    let (result, waiter) = oneshot::channel();
    service_client_service.send(ServiceClientCommand::Disconnect { result });
    let result = waiter
        .await
        .expect("service_client_service error")
        .map_err(log_error);

    log::debug!("disconnect - finished");
    result
}

pub async fn list_wallet_names() -> Result<Vec<String>, String> {
    log::debug!("list_wallet_names - start");
    let database_service = use_coroutine_handle::<DatabaseCommand>();
    let (result, rx) = oneshot::channel();
    database_service.send(DatabaseCommand::Wallet(
        DatabaseItemCommand::ListDatabaseItemNames { result },
    ));
    let wallet_names = rx.await.expect("database_service error").map_err(log_error);
    log::debug!("list_wallet_names - loaded");
    wallet_names
}

pub async fn list_heir_names() -> Result<Vec<String>, String> {
    log::debug!("list_heir_names - start");
    let database_service = use_coroutine_handle::<DatabaseCommand>();
    let (result, rx) = oneshot::channel();
    database_service.send(DatabaseCommand::Heir(
        DatabaseItemCommand::ListDatabaseItemNames { result },
    ));
    let heir_names = rx.await.expect("database_service error").map_err(log_error);
    log::debug!("list_heir_names - loaded");
    heir_names
}

pub async fn list_heir_wallet_names() -> Result<Vec<String>, String> {
    log::debug!("list_heir_wallet_names - start");
    let database_service = use_coroutine_handle::<DatabaseCommand>();
    let (result, rx) = oneshot::channel();
    database_service.send(DatabaseCommand::HeirWallet(
        DatabaseItemCommand::ListDatabaseItemNames { result },
    ));
    let heir_wallet_names = rx.await.expect("database_service error").map_err(log_error);
    log::debug!("list_heir_wallet_names - loaded");
    heir_wallet_names
}

pub async fn get_wallet(name: &str) -> Result<Wallet, String> {
    log::debug!("get_wallet({name}) - start");
    let database_service = use_coroutine_handle::<DatabaseCommand>();
    let service_client_service = use_coroutine_handle::<ServiceClientCommand>();
    let (result, rx) = oneshot::channel();

    let name = unsafe { std::mem::transmute::<&str, &'static str>(name) };
    database_service.send(DatabaseCommand::Wallet(
        DatabaseItemCommand::LoadDatabaseItem { name, result },
    ));
    let mut wallet = rx
        .await
        .expect("database_service error")
        .map_err(log_error)?;

    match wallet.key_provider_mut() {
        AnyKeyProvider::None => (),
        AnyKeyProvider::LocalKey(_lk) => (),
        AnyKeyProvider::Ledger(ledger) => ledger.init_ledger_client().await.map_err(log_error)?,
    };
    match wallet.online_wallet_mut() {
        AnyOnlineWallet::None => (),
        AnyOnlineWallet::Service(sb) => {
            let (result, waiter) = oneshot::channel();
            service_client_service.send(ServiceClientCommand::GetServiceClient { result });
            let service_client = waiter.await.expect("service_client_service error");
            sb.init_service_client_unchecked(service_client);
        }
        AnyOnlineWallet::Local(_lw) => (),
    };

    log::debug!("get_wallet({name}) - loaded");
    Ok(wallet)
}
