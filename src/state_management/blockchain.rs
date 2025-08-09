use dioxus::prelude::*;

use futures_util::stream::StreamExt;

use tokio::sync::oneshot;

use btc_heritage_wallet::{
    btc_heritage::bdk_types::{BlockchainFactory, GetHeight},
    online_wallet::{AnyBlockchainFactory, BlockchainProviderConfig},
    DatabaseSingleItem,
};

use super::{
    database::{DatabaseCommand, DatabaseReloadEvent},
    event_bus::{subscribe_event, EventBus},
};
use crate::utils::log_error;

/// Status of the blockchain provider connection
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BlockchainProviderStatus {
    /// Connected and ready
    Connected(u32),
    /// Disconnected
    Disconnected,
}

pub static BLOCKCHAIN_PROVIDER_STATUS: GlobalSignal<Option<BlockchainProviderStatus>> =
    Signal::global(|| None);

/// Commands for the blockchain provider service
#[derive(Debug)]
pub enum BlockchainProviderCommand {
    /// Get the blockchain factory (if available)
    GetBlockchainFactory {
        result: oneshot::Sender<Result<AnyBlockchainFactory, String>>,
    },
    /// Ask for a refresh of the Blockchain Provider status
    RefreshStatus,
    /// Get current configuration
    GetConfig {
        result: oneshot::Sender<BlockchainProviderConfig>,
    },
    /// Update configuration
    UpdateConfig { config: BlockchainProviderConfig },
    /// Internal trigger a refresh from the DB
    RefreshConfig,
}

/// Blockchain provider service coroutine
pub(super) fn use_blockchain_provider_service(
    event_bus: EventBus,
    database_service: Coroutine<DatabaseCommand>,
) -> Coroutine<BlockchainProviderCommand> {
    let service_handle = use_coroutine(
        move |mut rx: UnboundedReceiver<BlockchainProviderCommand>| async move {
            log::info!("blockchain_provider_service (coroutine) - start");

            let mut curent_config = create_config(database_service).await;
            let mut cached_factory =
                AnyBlockchainFactory::try_from(curent_config.clone()).map_err(log_error);
            update_blockchain_status(cached_factory.clone());

            while let Some(cmd) = rx.next().await {
                log::debug!(
                    "blockchain_provider_service (coroutine) - Processing command {cmd:?}..."
                );

                match cmd {
                    BlockchainProviderCommand::GetBlockchainFactory { result } => {
                        result
                            .send(cached_factory.clone())
                            .expect("channel failure");
                    }
                    BlockchainProviderCommand::RefreshStatus => {
                        cached_factory = AnyBlockchainFactory::try_from(curent_config.clone())
                            .map_err(log_error);
                        update_blockchain_status(cached_factory.clone())
                    }
                    BlockchainProviderCommand::GetConfig { result } => {
                        result.send(curent_config.clone()).expect("chanel failure")
                    }
                    BlockchainProviderCommand::UpdateConfig { config } => {
                        save_config(database_service, &config).await;
                        curent_config = config.clone();
                        cached_factory = AnyBlockchainFactory::try_from(config).map_err(log_error);
                        update_blockchain_status(cached_factory.clone());
                        log::info!("Blockchain provider configuration updated");
                    }
                    BlockchainProviderCommand::RefreshConfig => {
                        curent_config = create_config(database_service).await;
                        cached_factory = AnyBlockchainFactory::try_from(curent_config.clone())
                            .map_err(log_error);
                        update_blockchain_status(cached_factory.clone());
                    }
                }

                log::debug!("blockchain_provider_service (coroutine) - Command processed");
            }
        },
    );
    subscribe_event(event_bus, move |_event: DatabaseReloadEvent| {
        service_handle.send(BlockchainProviderCommand::RefreshConfig);
    });
    service_handle
}

fn update_blockchain_status(bcf: Result<AnyBlockchainFactory, String>) {
    spawn(async {
        log::debug!("update_blockchain_status - start");
        *BLOCKCHAIN_PROVIDER_STATUS.write() = None;
        let blockchain_provider_status = tokio::task::spawn_blocking(|| {
            let block_height = match bcf {
                Ok(AnyBlockchainFactory::Bitcoin(rpc_bcf)) => rpc_bcf
                    .build("osef", None)
                    .map_err(log_error)
                    .map(|rpc| rpc.get_height().map_err(log_error).ok())
                    .ok()
                    .flatten(),
                Ok(AnyBlockchainFactory::Electrum(electrum_bcf)) => {
                    electrum_bcf.get_height().map_err(log_error).ok()
                }
                Err(e) => {
                    log::error!("{e}");
                    None
                }
            };
            match block_height {
                Some(bh) => BlockchainProviderStatus::Connected(bh),
                None => BlockchainProviderStatus::Disconnected,
            }
        })
        .await
        .unwrap();
        log::debug!("update_blockchain_status - set to {blockchain_provider_status:?}");
        *BLOCKCHAIN_PROVIDER_STATUS.write() = Some(blockchain_provider_status);
        log::debug!("update_blockchain_status - finished");
    });
}

async fn create_config(database_service: Coroutine<DatabaseCommand>) -> BlockchainProviderConfig {
    let database = super::helpers::get_database(database_service).await;
    match BlockchainProviderConfig::load(&database) {
        Ok(config) => config,
        Err(e) => {
            match e {
                btc_heritage_wallet::errors::DbError::KeyDoesNotExists(_) => (),
                _ => log::error!("Could not load Blockchain Provider Config from database: {e}"),
            };
            BlockchainProviderConfig::default()
        }
    }
}

async fn save_config(
    database_service: Coroutine<DatabaseCommand>,
    config: &BlockchainProviderConfig,
) {
    let mut database = super::helpers::get_database(database_service).await;
    match config.save(&mut database) {
        Ok(()) => (),
        Err(e) => {
            log::error!("Could not save Blockchain Provider Config in database: {e}");
            ()
        }
    }
}
