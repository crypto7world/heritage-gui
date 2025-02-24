use dioxus::prelude::*;

use futures_util::stream::StreamExt;
use serde::Deserialize;
use std::{pin::Pin, sync::Arc};
use tokio::sync::oneshot;

use btc_heritage_wallet::{
    btc_heritage::bitcoincore_rpc::jsonrpc::serde_json,
    heritage_service_api_client::{DeviceAuthorizationResponse, HeritageServiceClient, Tokens},
};

use super::database::{DatabaseCommand, TokensCommand};

#[derive(Debug, Deserialize)]
pub struct UserId {
    // pub sub: Box<str>,
    // #[serde(rename = "cognito:username")]
    // pub cognito_username: Box<str>,
    pub preferred_username: Box<str>,
    pub email: Box<str>,
    #[serde(skip)]
    _phantom_data: std::marker::PhantomData<()>,
}
pub static CONNECTED_USER: GlobalSignal<Option<UserId>> = Signal::global(|| None);

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
    Disconnect {
        result:
            oneshot::Sender<Result<bool, btc_heritage_wallet::heritage_service_api_client::Error>>,
    },
    GetServiceClient {
        result: oneshot::Sender<HeritageServiceClient>,
    },
}
impl core::fmt::Debug for ServiceClientCommand {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Connect { .. } => f.debug_struct("Connect").finish_non_exhaustive(),
            Self::Disconnect { .. } => f.debug_struct("Disconnect").finish_non_exhaustive(),
            Self::GetServiceClient { .. } => {
                f.debug_struct("GetServiceClient").finish_non_exhaustive()
            }
        }
    }
}

pub(super) async fn service_client_service(mut rx: UnboundedReceiver<ServiceClientCommand>) {
    log::info!("service_client_service (coroutine) - start");
    let config = super::config::config();
    let database_service = use_coroutine_handle::<DatabaseCommand>();
    let (tokens_tx, tokens_rx) = oneshot::channel();
    database_service.send(DatabaseCommand::Tokens(TokensCommand::Load {
        result: tokens_tx,
    }));
    let tokens = tokens_rx
        .await
        .expect("database_service error")
        .expect("database error");
    let service_client = HeritageServiceClient::new(
        config.heritage_service_config.service_api_url.clone(),
        tokens,
    );
    update_connected_user(&service_client).await;
    while let Some(cmd) = rx.next().await {
        log::debug!("service_client_service (coroutine) - Processing commmand {cmd:?}...");
        match cmd {
            ServiceClientCommand::Connect { callback, result } => {
                match Tokens::new(
                    &config.heritage_service_config.auth_url,
                    &config.heritage_service_config.auth_client_id,
                    callback,
                )
                .await
                {
                    Ok(tokens) => {
                        let (tokens_result, tokens_waiter) = oneshot::channel();
                        let tokens = Arc::new(tokens);
                        database_service.send(DatabaseCommand::Tokens(TokensCommand::Save {
                            tokens: tokens.clone(),
                            result: tokens_result,
                        }));
                        tokens_waiter
                            .await
                            .expect("database_service error")
                            .expect("failed to persist tokens");
                        let tokens = Arc::into_inner(tokens);
                        if tokens.is_none() {
                            log::error!("Could not take back the ownership of Tokens after saving them in the database")
                        }
                        service_client.set_tokens(tokens).await;
                        update_connected_user(&service_client).await;
                        result.send(Ok(())).expect("chanel failure");
                    }
                    Err(e) => {
                        result.send(Err(e)).expect("chanel failure");
                    }
                }
            }
            ServiceClientCommand::Disconnect { result } => {
                service_client.set_tokens(None).await;
                update_connected_user(&service_client).await;
                let (tokens_result, tokens_waiter) = oneshot::channel();
                database_service.send(DatabaseCommand::Tokens(TokensCommand::Clear {
                    result: tokens_result,
                }));
                result
                    .send(tokens_waiter.await.expect("database_service error"))
                    .expect("chanel failure")
            }
            ServiceClientCommand::GetServiceClient { result } => {
                result.send(service_client.clone()).expect("chanel failure")
            }
        }
        log::debug!("service_client_service (coroutine) - Command processed");
    }
}

async fn update_connected_user(client: &HeritageServiceClient) {
    log::debug!("update_connected_user - start");
    let connected_user = client.get_tokens().read().await.as_ref().map(|t| {
        serde_json::from_value(t.id_token().as_json())
            .expect("id_token should always have the correct fields")
    });
    log::debug!("update_connected_user - set to {connected_user:?}");
    *CONNECTED_USER.write() = connected_user;
    log::debug!("update_connected_user - finished");
}
