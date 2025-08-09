use crate::prelude::*;

use std::collections::HashMap;

use btc_heritage_wallet::{
    bitcoin::{Address, Amount, OutPoint},
    btc_heritage::{bdk_types::BlockTime, HeritageConfig},
    heritage_service_api_client::HeritageUtxo,
    DatabaseItem, OnlineWallet, Wallet,
};

use crate::utils::{CCStr, CheapClone};

use super::{ExpirationStatus, HeritageConfigWithInfo};

pub fn use_resource_wallet_utxos(
    wallet: AsyncSignal<Wallet>,
) -> FResource<CheapClone<[HeritageUtxo]>> {
    use_resource(move || async move {
        log::debug!("use_resource_wallet_utxos - start");

        let wallet_utxos = wallet
            .with(async |wallet| {
                let wallet_name = wallet.name().to_owned();
                wallet
                    .list_heritage_utxos()
                    .await
                    .map_err(|e| {
                        let error = format!(
                            "Error retrieving the wallet UTXOs of wallet {}: {e}",
                            wallet_name
                        );
                        log::error!("{error}");
                        CCStr::from(error)
                    })
                    .map(Into::into)
            })
            .await;

        log::debug!("use_resource_wallet_utxos - loaded");

        wallet_utxos
    })
}

pub fn use_memo_balance_by_heritage_config(
    wallet_utxos: FResource<CheapClone<[HeritageUtxo]>>,
) -> FMemo<HashMap<HeritageConfig, Amount>> {
    use_memo(move || {
        log::debug!("use_memo_balance_by_heritage_config - start compute");

        let balance_by_heritage_config = wallet_utxos.lrmap(|utxos| {
            let mut balance_by_heritage_config = HashMap::new();
            for utxo in utxos.iter() {
                let heritage_config = utxo.heritage_config.clone();
                let utxo_amount = utxo.amount;
                balance_by_heritage_config
                    .entry(heritage_config)
                    .and_modify(|balance| *balance += utxo_amount)
                    .or_insert(utxo_amount);
            }
            balance_by_heritage_config
        });

        log::debug!("use_memo_balance_by_heritage_config - finish compute");
        balance_by_heritage_config
    })
}
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct SimpleUtxo {
    pub outpoint: OutPoint,
    pub amount: Amount,
}
impl From<&HeritageUtxo> for SimpleUtxo {
    fn from(utxo: &HeritageUtxo) -> Self {
        Self {
            outpoint: utxo.outpoint,
            amount: utxo.amount,
        }
    }
}
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct UtxoStats {
    pub count: usize,
    pub balance: Amount,
    pub utxos: CheapClone<[SimpleUtxo]>,
}
impl UtxoStats {
    fn count_utxo(&mut self, utxo: &HeritageUtxo) {
        self.count += 1;
        self.balance += utxo.amount;
    }
}

pub fn use_memo_utxo_stats_by_address(
    wallet_utxos: FResource<CheapClone<[HeritageUtxo]>>,
) -> FMemo<HashMap<Address, UtxoStats>> {
    use_memo(move || {
        log::debug!("use_memo_utxo_stats_by_address - start compute");

        let utxo_stats_by_address = wallet_utxos.lrmap(|wallet_utxos| {
            let mut utxo_stats_by_address = HashMap::new();
            let mut simple_utxos_by_address = HashMap::new();
            for utxo in wallet_utxos.iter() {
                let address = (*utxo.address).clone();
                utxo_stats_by_address
                    .entry(address.clone())
                    .or_insert(UtxoStats::default())
                    .count_utxo(utxo);
                simple_utxos_by_address
                    .entry(address)
                    .or_insert(Vec::new())
                    .push(SimpleUtxo::from(utxo))
            }
            for (address, utxo_stats) in utxo_stats_by_address.iter_mut() {
                utxo_stats.utxos = simple_utxos_by_address
                    .remove(address)
                    .expect("hashmap were filled together so have the same keys")
                    .into();
            }
            utxo_stats_by_address
        });

        log::debug!("use_memo_utxo_stats_by_address - finish compute");
        utxo_stats_by_address
    })
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UtxoWithInfo {
    pub outpoint: OutPoint,
    pub amount: Amount,
    pub confirmation_time: Option<BlockTime>,
    pub address: CCStr,
    pub heritage_config_expiration: Option<ExpirationStatus>,
}
pub fn use_memo_utxo_with_info(
    wallet_utxos: FResource<CheapClone<[HeritageUtxo]>>,
    heritage_configs_with_info_indexed_by_heritage_config: FMemo<
        HashMap<CheapClone<HeritageConfig>, HeritageConfigWithInfo>,
    >,
) -> FMemo<CheapClone<[UtxoWithInfo]>> {
    use_memo(move || {
        log::debug!("use_memo_utxo_with_info - start compute");

        let utxo_with_info = wallet_utxos.lrmap(|wallet_utxos| {
            wallet_utxos
                .iter()
                .map(|wallet_utxo| {
                    let HeritageUtxo {
                        outpoint, amount, ..
                    } = *wallet_utxo;

                    let confirmation_time = wallet_utxo.confirmation_time.clone();
                    let address = CCStr::from((*wallet_utxo.address).to_string());
                    let heritage_config_expiration =
                        heritage_configs_with_info_indexed_by_heritage_config.lrmap_ok(|index| {
                            index
                                .get(&wallet_utxo.heritage_config)
                                .expect("data integrity mandates that it is present")
                                .expiration_status
                        });
                    UtxoWithInfo {
                        outpoint,
                        amount,
                        confirmation_time,
                        address,
                        heritage_config_expiration,
                    }
                })
                .collect()
        });

        log::debug!("use_memo_utxo_with_info - finish compute");
        utxo_with_info
    })
}
