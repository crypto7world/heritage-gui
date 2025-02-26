use dioxus::prelude::*;

use futures_util::stream::StreamExt;
use std::rc::Rc;
use tokio::sync::oneshot;

use btc_heritage_wallet::{
    errors::DbError,
    heritage_service_api_client::{TokenCache, Tokens},
    Database, DatabaseItem, Heir, HeirWallet, Wallet,
};

use crate::utils::{RcStr, RcType};

pub enum DatabaseItemCommand<DBI: DatabaseItem + 'static> {
    CreateDatabaseItem {
        item: RcType<DBI>,
        result: oneshot::Sender<Result<(), DbError>>,
    },
    LoadDatabaseItem {
        name: RcStr,
        result: oneshot::Sender<Result<DBI, DbError>>,
    },
    SaveDatabaseItem {
        item: RcType<DBI>,
        result: oneshot::Sender<Result<(), DbError>>,
    },
    ListDatabaseItemNames {
        result: oneshot::Sender<Result<Vec<RcStr>, DbError>>,
    },
    ListDatabaseItems {
        result: oneshot::Sender<Result<Vec<RcType<DBI>>, DbError>>,
    },
}
impl<DBI: DatabaseItem + 'static> core::fmt::Debug for DatabaseItemCommand<DBI> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::CreateDatabaseItem { .. } => {
                f.debug_struct("CreateDatabaseItem").finish_non_exhaustive()
            }
            Self::LoadDatabaseItem { .. } => {
                f.debug_struct("LoadDatabaseItem").finish_non_exhaustive()
            }
            Self::SaveDatabaseItem { .. } => {
                f.debug_struct("SaveDatabaseItem").finish_non_exhaustive()
            }
            Self::ListDatabaseItemNames { .. } => f
                .debug_struct("ListDatabaseItemNames")
                .finish_non_exhaustive(),
            Self::ListDatabaseItems { .. } => {
                f.debug_struct("ListDatabaseItem").finish_non_exhaustive()
            }
        }
    }
}

pub enum TokensCommand {
    Load {
        result: oneshot::Sender<
            Result<Option<Tokens>, btc_heritage_wallet::heritage_service_api_client::Error>,
        >,
    },
    Save {
        tokens: Rc<Tokens>,
        result:
            oneshot::Sender<Result<(), btc_heritage_wallet::heritage_service_api_client::Error>>,
    },
    Clear {
        result:
            oneshot::Sender<Result<bool, btc_heritage_wallet::heritage_service_api_client::Error>>,
    },
}
impl core::fmt::Debug for TokensCommand {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Load { .. } => f.debug_struct("Load").finish_non_exhaustive(),
            Self::Save { .. } => f.debug_struct("Save").finish_non_exhaustive(),
            Self::Clear { .. } => f.debug_struct("Clear").finish_non_exhaustive(),
        }
    }
}

#[derive(Debug)]
pub enum DatabaseCommand {
    Wallet(DatabaseItemCommand<Wallet>),
    Heir(DatabaseItemCommand<Heir>),
    HeirWallet(DatabaseItemCommand<HeirWallet>),
    Tokens(TokensCommand),
}

pub(super) async fn database_service(mut rx: UnboundedReceiver<DatabaseCommand>) {
    log::info!("database_service (coroutine) - start");
    let config = super::config::config();
    let mut database = Database::new(&config.datadir, config.network)
        .await
        .expect("Could not open the database");
    while let Some(cmd) = rx.next().await {
        log::debug!("database_service (coroutine) - Processing commmand {cmd:?}...");
        match cmd {
            DatabaseCommand::Wallet(database_item_command) => {
                process_db_item_command(&mut database, database_item_command).await
            }
            DatabaseCommand::Heir(database_item_command) => {
                process_db_item_command(&mut database, database_item_command).await
            }
            DatabaseCommand::HeirWallet(database_item_command) => {
                process_db_item_command(&mut database, database_item_command).await
            }
            DatabaseCommand::Tokens(tokens_command) => match tokens_command {
                TokensCommand::Load { result } => {
                    result
                        .send(Tokens::load(&database).await)
                        .expect("chanel failure");
                }
                TokensCommand::Save { tokens, result } => result
                    .send(tokens.save(&mut database).await)
                    .expect("chanel failure"),
                TokensCommand::Clear { result } => result
                    .send(TokenCache::clear(&mut database).await)
                    .expect("chanel failure"),
            },
        }
        log::debug!("database_service (coroutine) - Command processed");
    }
}

async fn process_db_item_command<DBI: std::fmt::Debug + DatabaseItem + Sync>(
    db: &mut Database,
    cmd: DatabaseItemCommand<DBI>,
) {
    match cmd {
        DatabaseItemCommand::CreateDatabaseItem { item, result } => {
            result.send(item.create(db).await).expect("chanel failure");
        }
        DatabaseItemCommand::LoadDatabaseItem { name, result } => {
            result
                .send(DBI::load(db, &name).await)
                .expect("chanel failure");
        }
        DatabaseItemCommand::SaveDatabaseItem { item, result } => {
            result.send(item.save(db).await).expect("chanel failure");
        }
        DatabaseItemCommand::ListDatabaseItemNames { result } => {
            result
                .send(
                    DBI::list_names(db)
                        .await
                        .map(|strings| strings.into_iter().map(RcStr::from).collect()),
                )
                .expect("chanel failure");
        }
        DatabaseItemCommand::ListDatabaseItems { result } => {
            result
                .send(
                    DBI::all_in_db(db)
                        .await
                        .map(|r| r.into_iter().map(RcType::from).collect()),
                )
                .expect("chanel failure");
        }
    }
}
