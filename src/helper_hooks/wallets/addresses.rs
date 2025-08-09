use crate::prelude::*;

use std::collections::{HashMap, HashSet};

use btc_heritage_wallet::{
    bitcoin::{bip32::ChildNumber, Address},
    btc_heritage::heritage_wallet::WalletAddress,
    DatabaseItem, OnlineWallet, Wallet,
};

use crate::utils::{CCStr, CheapClone};

/// Resource hook for retrieving all addresses associated with a wallet
///
/// # Examples
///
/// ```
/// let wallet = use_resource_wallet("my_wallet".into());
/// let addresses = use_resource_wallet_addresses(wallet);
/// ```
pub fn use_resource_wallet_addresses(
    wallet: AsyncSignal<Wallet>,
) -> FResource<CheapClone<[WalletAddress]>> {
    use_resource(move || async move {
        log::debug!("use_resource_wallet_addresses - start");

        let wallet_addresses = wallet
            .with(async |wallet| {
                let wallet_name = wallet.name().to_owned();
                wallet
                    .list_addresses()
                    .await
                    .map_err(|e| {
                        let error = format!(
                            "Error retrieving the wallet addresses of wallet {}: {e}",
                            wallet_name
                        );
                        log::error!("{error}");
                        CCStr::from(error)
                    })
                    .map(Into::into)
            })
            .await;
        log::debug!("use_resource_wallet_addresses - loaded");

        wallet_addresses
    })
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WalletAddressWithInfo {
    pub wallet_address: CheapClone<WalletAddress>,
    pub heritage_config_infos: LResult<HeritageConfigWithInfo>,
    pub tx_stats: LResult<CheapClone<[TransactionStats]>>,
    pub utxo_stats: LResult<UtxoStats>,
}

pub fn use_memo_addresses_with_info(
    wallet_addresses: FResource<CheapClone<[WalletAddress]>>,
    heritage_configs_with_info_indexed_by_origin_info: FMemo<
        HashMap<AccountXPubOrigin, HeritageConfigWithInfo>,
    >,
    tx_stats_by_address: FMemo<HashMap<Address, CheapClone<[TransactionStats]>>>,
    utxo_stats_by_address: FMemo<HashMap<Address, UtxoStats>>,
) -> FMemo<CheapClone<[WalletAddressWithInfo]>> {
    use_memo(move || {
        log::debug!("use_memo_addresses_with_info - start compute");

        let heritage_configs_with_info_indexed_by_origin_info =
            &*heritage_configs_with_info_indexed_by_origin_info.read();
        let tx_stats_by_address = &*tx_stats_by_address.read();
        let utxo_stats_by_address = &*utxo_stats_by_address.read();

        let addresses_with_info = wallet_addresses.lrmap(|wallet_addresses| {
            wallet_addresses
                .iter()
                .cloned()
                .map(|wallet_address| {
                    let wallet_address = CheapClone::new(wallet_address);

                    let xpub_origin = AccountXPubOrigin::try_from(wallet_address.origin())
                        .expect("wallet address always have correct origin info");
                    let heritage_config_infos = heritage_configs_with_info_indexed_by_origin_info
                        .lrmap(|heritage_configs_with_info_indexed_by_origin_info| {
                            heritage_configs_with_info_indexed_by_origin_info
                                .get(&xpub_origin)
                                .expect("there is always an heritage config for an address")
                                .clone()
                        });
                    let address = wallet_address.address();
                    let tx_stats = tx_stats_by_address.lrmap(|tx_stats_by_address| {
                        tx_stats_by_address
                            .get(address)
                            .cloned()
                            .unwrap_or_default()
                    });
                    let utxo_stats = utxo_stats_by_address.lrmap(|utxo_stats_by_address| {
                        utxo_stats_by_address
                            .get(address)
                            .cloned()
                            .unwrap_or_default()
                    });
                    WalletAddressWithInfo {
                        wallet_address,
                        heritage_config_infos,
                        tx_stats,
                        utxo_stats,
                    }
                })
                .collect()
        });
        log::debug!("use_memo_addresses_with_info - finish compute");
        addresses_with_info
    })
}

pub fn use_memo_ready_to_use_address(
    addresses_with_info: FMemo<CheapClone<[WalletAddressWithInfo]>>,
) -> Memo<Option<Option<CheapClone<WalletAddress>>>> {
    use_memo(move || {
        log::debug!("use_memo_ready_to_use_address - start compute");

        let ready_to_use_address = addresses_with_info.lrmap_ok(|addresses_with_info| {
            addresses_with_info
                .iter()
                .filter_map(|wallet_address_with_info| {
                    let address_is_not_change = wallet_address_with_info.wallet_address.origin().1
                        [3]
                        == ChildNumber::Normal { index: 0 };
                    let address_is_unused = match &wallet_address_with_info.tx_stats {
                        Some(Ok(tx_stats)) if tx_stats.is_empty() => true,
                        _ => false,
                    };
                    let address_is_current = match &wallet_address_with_info.heritage_config_infos {
                        Some(Ok(heritage_config_infos))
                            if matches!(
                                heritage_config_infos.expiration_status,
                                ExpirationStatus::Current
                            ) =>
                        {
                            true
                        }
                        _ => false,
                    };
                    (address_is_unused && address_is_current && address_is_not_change)
                        .then(|| wallet_address_with_info.wallet_address.clone())
                })
                .rev()
                .next()
        });
        log::debug!("use_memo_ready_to_use_address - finish compute");
        ready_to_use_address
    })
}

pub fn use_memo_addresses_set(
    wallet_addresses: FResource<CheapClone<[WalletAddress]>>,
) -> FMemo<HashSet<Address>> {
    use_memo(move || {
        log::debug!("use_memo_addresses_set - start compute");
        let addresses_set = wallet_addresses.lrmap(|wallet_addresses| {
            wallet_addresses
                .iter()
                .map(|wallet_address| (**wallet_address).clone())
                .collect()
        });
        log::debug!("use_memo_addresses_set - finish compute");
        addresses_set
    })
}
