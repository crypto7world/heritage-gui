use super::*;

use btc_heritage_wallet::{
    heritage_service_api_client::HeritageServiceConfig, online_wallet::BlockchainProviderConfig,
};

pub async fn update_application_config(
    database_service: Coroutine<DatabaseCommand>,
    config: ApplicationConfig,
) -> Result<(), String> {
    log::debug!("update_application_config - start");
    let (result, rx) = oneshot::channel();
    database_service.send(DatabaseCommand::UpdateConfig { config, result });
    log::debug!("update_application_config - finished");
    rx.await.expect("database_service error")
}

pub async fn get_service_config(
    service_client_service: Coroutine<ServiceClientCommand>,
) -> HeritageServiceConfig {
    log::debug!("get_service_config - start");
    let (result, rx) = oneshot::channel();
    service_client_service.send(ServiceClientCommand::GetConfig { result });
    let result = rx.await.expect("service_client_service error");

    log::debug!("get_service_config - finished");
    result
}
pub fn update_service_config(
    service_client_service: Coroutine<ServiceClientCommand>,
    config: HeritageServiceConfig,
) {
    log::debug!("update_service_config - start");
    service_client_service.send(ServiceClientCommand::UpdateConfig { config });
    log::debug!("update_service_config - finished");
}

pub async fn get_blockchain_provider_config(
    blockchain_service: Coroutine<BlockchainProviderCommand>,
) -> BlockchainProviderConfig {
    log::debug!("get_blockchain_provider_config - start");
    let (result, rx) = oneshot::channel();
    blockchain_service.send(BlockchainProviderCommand::GetConfig { result });
    let result = rx.await.expect("blockchain_service error");

    log::debug!("get_blockchain_provider_config - finished");
    result
}
pub fn update_blockchain_provider_config(
    blockchain_service: Coroutine<BlockchainProviderCommand>,
    config: BlockchainProviderConfig,
) {
    log::debug!("update_blockchain_provider_config - start");
    blockchain_service.send(BlockchainProviderCommand::UpdateConfig { config });
    log::debug!("update_blockchain_provider_config - finished");
}
