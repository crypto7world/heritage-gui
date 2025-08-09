use std::path::PathBuf;

use dioxus::prelude::*;

use futures_util::stream::StreamExt;
use tokio::sync::oneshot;

use btc_heritage_wallet::{
    bitcoin::Network, btc_heritage::utils::bitcoin_network, errors::DbError,
    heritage_service_api_client::TokenCache, Database, DatabaseItem, Heir, HeirWallet, Wallet,
};

use super::event_bus::EventBus;

pub enum DatabaseItemCommand<DBI: DatabaseItem + Send + 'static> {
    ListNames {
        result: oneshot::Sender<Result<Vec<String>, DbError>>,
    },
    ListItems {
        result: oneshot::Sender<Result<Vec<DBI>, DbError>>,
    },
}
impl<DBI: DatabaseItem + 'static + Send> core::fmt::Debug for DatabaseItemCommand<DBI> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::ListNames { .. } => f.debug_struct("ListNames").finish_non_exhaustive(),
            Self::ListItems { .. } => f.debug_struct("ListItems").finish_non_exhaustive(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ApplicationConfig {
    pub network: Network,
    pub datadir: PathBuf,
}
impl Default for ApplicationConfig {
    fn default() -> Self {
        let mut datadir: PathBuf = dirs_next::home_dir().unwrap_or_default();
        datadir.push(".heritage-wallet");
        Self {
            network: bitcoin_network::get(),
            datadir,
        }
    }
}
pub static APPLICATION_CONFIG: GlobalSignal<ApplicationConfig> =
    Signal::global(|| ApplicationConfig::default());

/// Event fired when the database is reloaded or changed
#[derive(Debug, Clone, Copy)]
pub struct DatabaseReloadEvent;
impl super::event_bus::EventId for DatabaseReloadEvent {
    fn event_id() -> &'static str {
        "database_reload"
    }
}

#[derive(Debug)]
pub enum DatabaseCommand {
    Wallet(DatabaseItemCommand<Wallet>),
    Heir(DatabaseItemCommand<Heir>),
    HeirWallet(DatabaseItemCommand<HeirWallet>),
    ClearTokens {
        result:
            oneshot::Sender<Result<bool, btc_heritage_wallet::heritage_service_api_client::Error>>,
    },
    GetDatabase {
        result: oneshot::Sender<Database>,
    },
    /// Update configuration
    UpdateConfig {
        config: ApplicationConfig,
        result: oneshot::Sender<Result<(), String>>,
    },
}

pub(super) fn use_database_service(event_bus_service: EventBus) -> Coroutine<DatabaseCommand> {
    use_coroutine(
        move |mut rx: UnboundedReceiver<DatabaseCommand>| async move {
            log::info!("database_service (coroutine) - start");

            let current_config = create_config();
            let mut database = {
                let datadir = current_config.datadir.clone();
                tokio::task::spawn_blocking(move || Database::new(&datadir, current_config.network))
                    .await
                    .unwrap()
                    .expect("Could not open the database")
            };
            *APPLICATION_CONFIG.write() = current_config;

            while let Some(cmd) = rx.next().await {
                log::debug!("database_service (coroutine) - Processing commmand {cmd:?}...");
                match cmd {
                    DatabaseCommand::Wallet(database_item_command) => {
                        process_db_item_command(&database, database_item_command).await
                    }
                    DatabaseCommand::Heir(database_item_command) => {
                        process_db_item_command(&database, database_item_command).await
                    }
                    DatabaseCommand::HeirWallet(database_item_command) => {
                        process_db_item_command(&database, database_item_command).await
                    }
                    DatabaseCommand::ClearTokens { result } => result
                        .send(TokenCache::clear(&mut database).await)
                        .expect("chanel failure"),
                    DatabaseCommand::GetDatabase { result } => {
                        result.send(database.clone()).expect("chanel failure")
                    }
                    DatabaseCommand::UpdateConfig { config, result } => {
                        if APPLICATION_CONFIG.peek().network != config.network {
                            bitcoin_network::set(config.network);
                        }
                        match {
                            let datadir = config.datadir.clone();
                            let network = config.network;
                            tokio::task::spawn_blocking(move || Database::new(&datadir, network))
                                .await
                                .unwrap()
                        } {
                            Ok(new_db) => {
                                database = new_db;
                                *APPLICATION_CONFIG.write() = config;
                                super::event_bus::publish_event(
                                    event_bus_service,
                                    DatabaseReloadEvent,
                                );
                                result.send(Ok(())).expect("chanel failure")
                            }
                            Err(e) => result
                                .send(Err(format!("Could not open the database: {e}")))
                                .expect("chanel failure"),
                        }
                    }
                }
                log::debug!("database_service (coroutine) - Command processed");
            }
        },
    )
}

async fn process_db_item_command<DBI: std::fmt::Debug + DatabaseItem + Send>(
    db: &Database,
    cmd: DatabaseItemCommand<DBI>,
) {
    match cmd {
        DatabaseItemCommand::ListNames { result } => {
            result
                .send(
                    db.clone()
                        .blocking_operation(move |db| DBI::list_names(&db))
                        .await,
                )
                .expect("chanel failure");
        }
        DatabaseItemCommand::ListItems { result } => {
            result
                .send(
                    db.clone()
                        .blocking_operation(move |db| DBI::all_in_db(&db))
                        .await,
                )
                .expect("chanel failure");
        }
    }
}

fn create_config() -> ApplicationConfig {
    let mut default_config = ApplicationConfig::default();

    // First check if env vars are present, if yes create the config from it (with defaults from missing vars)
    if let Some(datadir) = std::env::var("HERITAGE_WALLET_HOME")
        .ok()
        .map(|s| s.parse().expect("valid path string"))
    {
        default_config.datadir = datadir;
    }
    default_config
}
