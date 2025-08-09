use crate::prelude::*;

use btc_heritage_wallet::heritage_service_api_client::Fingerprint;

use super::*;

pub async fn connect<F, Fut>(
    service_client_service: Coroutine<ServiceClientCommand>,
    callback: F,
) -> Result<(), String>
where
    F: FnOnce(DeviceAuthorizationResponse) -> Fut + 'static,
    Fut: std::future::Future<
            Output = Result<(), btc_heritage_wallet::heritage_service_api_client::Error>,
        > + 'static,
{
    log::debug!("connect - start");
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

pub async fn disconnect(service_client_service: Coroutine<ServiceClientCommand>) {
    log::debug!("disconnect - start");
    service_client_service.send(ServiceClientCommand::Disconnect);
    log::debug!("disconnect - finished");
}

pub async fn heritage_service_client(
    service_client_service: Coroutine<ServiceClientCommand>,
) -> HeritageServiceClient {
    log::debug!("heritage_service_client - start");
    let (result, waiter) = oneshot::channel();
    service_client_service.send(ServiceClientCommand::GetServiceClient { result });
    let result = waiter.await.expect("service_client_service error");

    log::debug!("heritage_service_client - finished");
    result
}

pub fn refresh_service_status(service_client_service: Coroutine<ServiceClientCommand>) {
    log::debug!("refresh_service_status - start");
    service_client_service.send(ServiceClientCommand::RefreshStatus);
    log::debug!("refresh_service_status - finished");
}

pub fn inject_serviceable_wallet(
    service_client_service: Coroutine<ServiceClientCommand>,
    wallet_id: String,
    fingerprint: Option<Fingerprint>,
) {
    log::debug!("inject_serviceable_wallet - start");
    service_client_service.send(ServiceClientCommand::InjectServiceableWallet {
        wallet_id,
        fingerprint,
    });
    log::debug!("inject_serviceable_wallet - finished");
}

pub fn use_service_key() -> Memo<CCStr> {
    use_memo(move || match *state_management::SERVICE_STATUS.read() {
        Some(ServiceStatus::Connected(ref css)) => {
            CCStr::from(css.user_id.preferred_username.as_ref())
        }
        Some(ServiceStatus::Disconnected) => CCStr::from("disconnected"),
        None => CCStr::from("none"),
    })
}
