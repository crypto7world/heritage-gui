use crate::utils::log_error;

use super::{config, database, database_mut};
use btc_heritage_wallet::{
    btc_heritage::bitcoincore_rpc::jsonrpc::serde_json,
    heritage_service_api_client::{
        auth::DeviceAuthorizationResponse, HeritageServiceClient, Tokens,
    },
};
use serde::Deserialize;
use std::{
    marker::PhantomData,
    ops::{Deref, DerefMut},
    sync::OnceLock,
};

static SERVICE_CLIENT: OnceLock<HeritageServiceClient> = OnceLock::new();

#[derive(Debug, Deserialize)]
pub struct UserId {
    // pub sub: Box<str>,
    // #[serde(rename = "cognito:username")]
    // pub cognito_username: Box<str>,
    pub preferred_username: Box<str>,
    pub email: Box<str>,
    #[serde(skip)]
    _phantom_data: PhantomData<()>,
}

pub fn is_connected() -> bool {
    log::debug!("is_connected()");
    match service_client().list_wallets() {
        Ok(_) => true,
        Err(e) => {
            log::warn!("{e}");
            false
        }
    }
}

pub fn connect<F>(callback: F) -> Result<UserId, String>
where
    F: FnOnce(
        DeviceAuthorizationResponse,
    ) -> Result<(), btc_heritage_wallet::heritage_service_api_client::Error>,
{
    log::info!("Retrieving Service Tokens");
    let config = config();
    let tokens = Tokens::new(
        &config.heritage_service_config.auth_url,
        &config.heritage_service_config.auth_client_id,
        callback,
    )
    .map_err(log_error)?;
    log::debug!("Persisting Tokens in DB");
    tokens.save(database_mut().deref_mut()).map_err(log_error)?;

    log::debug!("Instantiating the UserId struct");
    let user_id: UserId = serde_json::from_value(tokens.id_token().as_json()).map_err(log_error)?;

    log::debug!("Providing Tokens to the HeritageServiceClient");
    service_client().set_tokens(Some(tokens));

    Ok(user_id)
}

pub fn get_userid() -> Option<UserId> {
    match Tokens::load(database().deref())
        .expect("database error")
        .map(|t| serde_json::from_value(t.id_token().as_json()))
    {
        Some(Ok(user_id)) => Some(user_id),
        Some(Err(e)) => {
            log::error!("{e}");
            None
        }
        None => None,
    }
}

pub fn disconnect() -> Result<(), String> {
    log::debug!("Removing Tokens from the HeritageServiceClient");
    service_client().set_tokens(None);
    log::debug!("Removing Tokens in DB");
    btc_heritage_wallet::heritage_service_api_client::TokenCache::clear(database_mut().deref_mut())
        .map_err(log_error)?;
    Ok(())
}

pub fn service_client() -> HeritageServiceClient {
    log::debug!("service_client()");

    SERVICE_CLIENT
        .get_or_init(|| {
            let config = config();
            let db = database();
            let tokens = Tokens::load(db.deref())
                .expect("database error")
                .map(|mut t| {
                    if t.need_refresh() {
                        match t.refresh() {
                            Ok(_) => {
                                // Drop the read-lock in order to acquire the write-lock
                                drop(db);
                                log::info!("Refreshing tokens in DB");
                                t.save(database_mut().deref_mut())
                                    .map_err(|e| {
                                        log::warn!("Failed to write tokens in DB: {e}");
                                        e
                                    })
                                    .ok();
                            }

                            Err(e) => {
                                log::warn!("Failed to refresh tokens: {e}");
                            }
                        }
                    }
                    t
                });

            HeritageServiceClient::new(
                config.heritage_service_config.service_api_url.clone(),
                tokens,
            )
        })
        .clone()
}
