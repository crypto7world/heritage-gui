mod async_init;
mod heirs;
mod heirwallets;
mod utils;
mod wallets;

pub mod prelude {
    pub use super::async_init::AsyncSignal;
    pub use super::heirs::CompositeHeir;
    pub use super::heirwallets::{ContextualizedHeritages, HeritageContext};
    pub use super::utils::{FMemo, FResource, LResult, LoadableFaillibleMapper, LoadableMapper};
    pub use super::wallets::{
        AccountXPubOrigin, ExpirationStatus, HeritageConfigWithInfo, SimpleUtxo,
        TransactionHistoryItem, TransactionHistoryItemOwnedIO, TransactionStats, TxIO, UtxoStats,
        UtxoWithInfo, WalletAddressWithInfo,
    };
    pub mod helper_hooks {
        pub use super::super::async_init::use_async_init;
        pub use super::super::heirs::{
            use_async_heir, use_memo_heirs, use_resource_database_heirs, use_resource_service_heirs,
        };
        pub use super::super::heirwallets::{
            use_async_heirwallet, use_memo_heirwallet_contextualized_heritages,
            use_memo_heirwallet_fingerprint, use_memo_heirwallet_keyprovider_status,
            use_memo_heritage_provider_status, use_memo_service_only_heritages,
            use_resource_heirwallet_heritages, use_resource_heirwallet_local_lastsync,
            use_resource_heirwallet_names, use_resource_service_heritages,
        };
        pub use super::super::utils::use_memo_resource;
        pub use super::super::wallets::{
            use_async_wallet, use_memo_addresses_set, use_memo_addresses_with_info,
            use_memo_balance_by_heritage_config, use_memo_fingerprint,
            use_memo_heritage_configs_with_info,
            use_memo_heritage_configs_with_info_indexed_by_heritage_config,
            use_memo_heritage_configs_with_info_indexed_by_origin_info,
            use_memo_ledger_registered_policies, use_memo_ledger_unregistered_policies,
            use_memo_ready_to_use_address, use_memo_transaction_history_items,
            use_memo_tx_stats_by_address, use_memo_utxo_stats_by_address, use_memo_utxo_with_info,
            use_memo_wallet_keyprovider_status, use_memo_wallet_online_status,
            use_memo_wallet_uses_ledger, use_resource_service_only_wallets,
            use_resource_service_wallets, use_resource_wallet_account_xpubs,
            use_resource_wallet_addresses, use_resource_wallet_descriptor_backup,
            use_resource_wallet_names, use_resource_wallet_status,
            use_resource_wallet_subwallet_configs, use_resource_wallet_transactions,
            use_resource_wallet_utxos,
        };
    }
}
