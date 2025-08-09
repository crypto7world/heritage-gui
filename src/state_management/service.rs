use dioxus::prelude::*;

use futures_util::stream::StreamExt;
use serde::Deserialize;
use std::{
    collections::{HashMap, HashSet},
    pin::Pin,
    sync::Arc,
};
use tokio::sync::oneshot;

use btc_heritage_wallet::{
    btc_heritage::bitcoincore_rpc::jsonrpc::serde_json,
    heritage_provider,
    heritage_service_api_client::{
        DeviceAuthorizationResponse, Fingerprint, HeritageServiceClient, HeritageServiceConfig,
    },
    online_wallet, BoundFingerprint, DatabaseSingleItem,
};

use crate::utils::log_error;

use super::{
    database::{DatabaseCommand, DatabaseReloadEvent},
    event_bus::{subscribe_event, EventBus},
};

#[derive(Debug, Deserialize)]
pub struct UserId {
    // pub sub: Box<str>,
    // #[serde(rename = "cognito:username")]
    // pub cognito_username: Box<str>,
    pub preferred_username: Box<str>,
    pub email: Box<str>,
}

#[derive(Debug)]
pub struct ConnectedServiceStatus {
    pub user_id: UserId,
    serviceable_wallets: HashMap<String, Option<Fingerprint>>,
    serviceable_heritages: HashSet<Fingerprint>,
}
#[derive(Debug)]
pub enum ServiceStatus {
    Connected(ConnectedServiceStatus),
    Disconnected,
}
impl ServiceStatus {
    pub fn can_serve_wallet(&self, online_wallet: &online_wallet::ServiceBinding) -> bool {
        match self {
            ServiceStatus::Connected(css) => {
                let css_fg = css
                    .serviceable_wallets
                    .get(online_wallet.wallet_id())
                    .cloned()
                    .flatten();
                let sb_fg = online_wallet.fingerprint();

                match (css_fg, sb_fg) {
                    (Some(css_fg), Ok(sb_fg)) => css_fg == sb_fg,
                    (None, Err(_)) => true,
                    _ => false,
                }
            }
            ServiceStatus::Disconnected => false,
        }
    }
    pub fn can_serve_heritage(&self, service_binding: &heritage_provider::ServiceBinding) -> bool {
        match self {
            ServiceStatus::Connected(css) => css.serviceable_heritages.contains(
                &service_binding
                    .fingerprint()
                    .expect("always present for heritage_provider sb"),
            ),
            ServiceStatus::Disconnected => false,
        }
    }
}
pub static SERVICE_STATUS: GlobalSignal<Option<ServiceStatus>> = Signal::global(|| None);

type Callback = Box<
    dyn FnOnce(
        DeviceAuthorizationResponse,
    ) -> Pin<
        Box<
            dyn std::future::Future<
                Output = Result<(), btc_heritage_wallet::heritage_service_api_client::Error>,
            >,
        >,
    >,
>;

pub enum ServiceClientCommand {
    Connect {
        callback: Callback,
        result:
            oneshot::Sender<Result<(), btc_heritage_wallet::heritage_service_api_client::Error>>,
    },
    Disconnect,
    GetServiceClient {
        result: oneshot::Sender<HeritageServiceClient>,
    },
    /// Ask for a refresh of the Service status
    RefreshStatus,
    /// Inject a new Serviceable Wallet in the Service status without
    /// refreshing it. Does nothing if the ServiceStatus is not ServiceStatus::Connected(_)
    InjectServiceableWallet {
        wallet_id: String,
        fingerprint: Option<Fingerprint>,
    },
    /// Get current configuration
    GetConfig {
        result: oneshot::Sender<HeritageServiceConfig>,
    },
    /// Update configuration
    UpdateConfig {
        config: HeritageServiceConfig,
    },
    /// Internal trigger a refresh from the DB
    RefreshConfig,
}
impl core::fmt::Debug for ServiceClientCommand {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Connect { result, .. } => f
                .debug_struct("Connect")
                .field("result", result)
                .finish_non_exhaustive(),
            Self::Disconnect => f.debug_struct("Disconnect").finish_non_exhaustive(),
            Self::GetServiceClient { result } => f
                .debug_struct("GetServiceClient")
                .field("result", result)
                .finish(),
            Self::RefreshStatus => f.debug_struct("RefreshStatus").finish(),
            ServiceClientCommand::InjectServiceableWallet {
                wallet_id,
                fingerprint,
            } => f
                .debug_struct("InjectServiceableWallet")
                .field("wallet_id", wallet_id)
                .field("fingerprint", fingerprint)
                .finish(),
            Self::GetConfig { result } => {
                f.debug_struct("GetConfig").field("result", result).finish()
            }
            Self::UpdateConfig { config } => f
                .debug_struct("UpdateConfig")
                .field("config", config)
                .finish_non_exhaustive(),
            ServiceClientCommand::RefreshConfig => f.debug_struct("RefreshConfig").finish(),
        }
    }
}

pub(super) fn use_service_client_service(
    event_bus: EventBus,
    database_service: Coroutine<DatabaseCommand>,
) -> Coroutine<ServiceClientCommand> {
    let service_handle = use_coroutine(
        move |mut rx: UnboundedReceiver<ServiceClientCommand>| async move {
            log::info!("service_client_service (coroutine) - start");

            let mut curent_config = create_config(database_service).await;
            let mut service_client =
                service_client_from_database(database_service, curent_config.clone()).await;

            while let Some(cmd) = rx.next().await {
                log::debug!("service_client_service (coroutine) - Processing commmand {cmd:?}...");
                match cmd {
                    ServiceClientCommand::Connect { callback, result } => {
                        match service_client.login(callback).await {
                            Ok(()) => {
                                let mut database =
                                    super::helpers::get_database(database_service).await;

                                match service_client.persist_tokens_in_cache(&mut database).await {
                                    Ok(()) => (),
                                    Err(e) => {
                                        log::error!("Could not persist Heritage Service client Tokens in the database: {e}");
                                        ()
                                    }
                                };
                                update_service_status(service_client.clone());

                                result.send(Ok(())).expect("chanel failure");
                            }
                            Err(e) => {
                                result.send(Err(e)).expect("chanel failure");
                            }
                        }
                    }
                    ServiceClientCommand::Disconnect => {
                        service_client.logout().await;
                        update_service_status(service_client.clone());
                        let (database_result, rx) = oneshot::channel();
                        database_service.send(DatabaseCommand::ClearTokens {
                            result: database_result,
                        });
                        match rx.await.expect("database_service error") {
                            Ok(_) => (),
                            Err(e) => {
                                log::error!("Could not clear Heritage Service client Tokens in the database: {e}");
                                ()
                            }
                        };
                    }
                    ServiceClientCommand::GetServiceClient { result } => {
                        result.send(service_client.clone()).expect("chanel failure");
                    }
                    ServiceClientCommand::RefreshStatus => {
                        update_service_status(service_client.clone());
                    }
                    ServiceClientCommand::InjectServiceableWallet {
                        wallet_id,
                        fingerprint,
                    } => {
                        if let Some(ServiceStatus::Connected(ref mut ss)) = *SERVICE_STATUS.write()
                        {
                            ss.serviceable_wallets.insert(wallet_id, fingerprint);
                        }
                    }
                    ServiceClientCommand::GetConfig { result } => {
                        result.send(curent_config.clone()).expect("chanel failure");
                    }
                    ServiceClientCommand::UpdateConfig { config } => {
                        service_client =
                            service_client_from_database(database_service, config.clone()).await;
                        save_config(database_service, &config).await;
                        curent_config = config;
                    }
                    ServiceClientCommand::RefreshConfig => {
                        curent_config = create_config(database_service).await;
                        service_client =
                            service_client_from_database(database_service, curent_config.clone())
                                .await;
                    }
                }
                log::debug!("service_client_service (coroutine) - Command processed");
            }
        },
    );
    subscribe_event(event_bus, move |_event: DatabaseReloadEvent| {
        service_handle.send(ServiceClientCommand::RefreshConfig);
    });
    service_handle
}

fn update_service_status(client: HeritageServiceClient) {
    spawn(async move {
        log::debug!("update_service_status - start");
        *SERVICE_STATUS.write() = None;
        let service_status = if client.has_tokens().await {
            let user_id_task = async {
                client
                    .get_tokens()
                    .read()
                    .await
                    .as_ref()
                    .map(|t| {
                        serde_json::from_value(t.id_token().as_json())
                            .expect("id_token should always have the correct fields")
                    })
                    .expect("just verified we have tokens")
            };

            let (user_id, serviceable_wallets_res, serviceable_heritages_res) =
                tokio::join!(user_id_task, client.list_wallets(), client.list_heritages());

            let serviceable_wallets = serviceable_wallets_res
                .map(|wallet_list| {
                    wallet_list
                        .into_iter()
                        .map(|wallet_meta| (wallet_meta.id, wallet_meta.fingerprint))
                        .collect()
                })
                .map_err(log_error)
                .unwrap_or_default();

            let serviceable_heritages = serviceable_heritages_res
                .map(|heritage_list| {
                    heritage_list
                        .into_iter()
                        .filter_map(|heritage| heritage.heir_config.map(|hc| hc.fingerprint()))
                        .collect()
                })
                .map_err(log_error)
                .unwrap_or_default();

            ServiceStatus::Connected(ConnectedServiceStatus {
                user_id,
                serviceable_wallets,
                serviceable_heritages,
            })
        } else {
            ServiceStatus::Disconnected
        };
        log::debug!("update_service_status - set to {service_status:?}");
        *SERVICE_STATUS.write() = Some(service_status);
        log::debug!("update_service_status - finished");
    });
}

async fn create_config(database_service: Coroutine<DatabaseCommand>) -> HeritageServiceConfig {
    let default_config = HeritageServiceConfig::default();

    // First check if env vars are present, if yes create the config from it (with defaults from missing vars)
    let service_api_url = std::env::var("HERITAGE_SERVICE_API_URL")
        .ok()
        .map(Arc::from);
    let auth_url = std::env::var("HERITAGE_AUTH_URL").ok().map(Arc::from);
    let auth_client_id = std::env::var("HERITAGE_AUTH_CLIENT_ID").ok().map(Arc::from);
    if service_api_url.is_some() || auth_url.is_some() || auth_client_id.is_some() {
        return HeritageServiceConfig {
            service_api_url: service_api_url.unwrap_or(default_config.service_api_url),
            auth_url: auth_url.unwrap_or(default_config.auth_url),
            auth_client_id: auth_client_id.unwrap_or(default_config.auth_client_id),
        };
    }

    // Then check DB
    let database = super::helpers::get_database(database_service).await;
    match HeritageServiceConfig::load(&database) {
        Ok(config) => config,
        Err(e) => {
            match e {
                btc_heritage_wallet::errors::DbError::KeyDoesNotExists(_) => (),
                _ => log::error!("Could not load Heritage Service Config from database: {e}"),
            };
            // Then Default
            default_config
        }
    }
}

async fn service_client_from_database(
    database_service: Coroutine<DatabaseCommand>,
    config: HeritageServiceConfig,
) -> HeritageServiceClient {
    let service_client = HeritageServiceClient::from(config);
    let database = super::helpers::get_database(database_service).await;
    match service_client.load_tokens_from_cache(&database).await {
        Ok(()) => (),
        Err(e) => {
            log::error!("Could not load Heritage Service client Tokens from the database: {e}");
            ()
        }
    };

    update_service_status(service_client.clone());

    service_client
}

async fn save_config(database_service: Coroutine<DatabaseCommand>, config: &HeritageServiceConfig) {
    let mut database = super::helpers::get_database(database_service).await;
    match config.save(&mut database) {
        Ok(()) => (),
        Err(e) => {
            log::error!("Could not save Heritage Service Config in database: {e}");
            ()
        }
    }
}
