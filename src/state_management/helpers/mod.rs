mod blockchain;
mod config;
mod database;
mod heir;
mod heirwallet;
mod ledger;
mod service;
mod wallet;

use dioxus::prelude::*;

use btc_heritage_wallet::{
    heritage_service_api_client::{DeviceAuthorizationResponse, HeritageServiceClient},
    AnyHeritageProvider, AnyOnlineWallet, Database, DatabaseItem, Heir, HeirWallet, Wallet,
};
use tokio::sync::oneshot;

use crate::utils::{log_error, CCStr, CheapClone};

use super::{
    blockchain::BlockchainProviderCommand,
    database::{ApplicationConfig, DatabaseCommand, DatabaseItemCommand},
    ledger::{LedgerStatus, LEDGER_STATUS},
    service::ServiceClientCommand,
};

pub use blockchain::*;
pub use config::*;
pub use database::*;
pub use heir::*;
pub use heirwallet::*;
pub use ledger::*;
pub use service::*;
pub use wallet::*;

pub fn use_blockchain_provider_service() -> Coroutine<BlockchainProviderCommand> {
    use_coroutine_handle()
}

pub fn use_database_service() -> Coroutine<DatabaseCommand> {
    use_coroutine_handle()
}

pub fn use_service_client_service() -> Coroutine<ServiceClientCommand> {
    use_coroutine_handle()
}
