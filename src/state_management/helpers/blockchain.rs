use btc_heritage_wallet::online_wallet::AnyBlockchainFactory;

use super::*;

pub fn refresh_blockchain_provider_status(
    blockchain_provider_service: Coroutine<BlockchainProviderCommand>,
) {
    log::debug!("refresh_blockchain_provider_status - start");
    blockchain_provider_service.send(BlockchainProviderCommand::RefreshStatus);
    log::debug!("refresh_blockchain_provider_status - finished");
}

pub async fn blockchain_factory(
    blockchain_provider_service: Coroutine<BlockchainProviderCommand>,
) -> Result<AnyBlockchainFactory, String> {
    log::debug!("blockchain_factory - start");
    let (result, waiter) = oneshot::channel();
    blockchain_provider_service.send(BlockchainProviderCommand::GetBlockchainFactory { result });
    let result = waiter.await.expect("blockchain_provider_service error");

    log::debug!("blockchain_factory - finished");
    result
}
