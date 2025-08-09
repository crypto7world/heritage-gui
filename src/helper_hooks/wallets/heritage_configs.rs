use crate::prelude::*;

use std::collections::HashMap;

use btc_heritage_wallet::{
    bitcoin::{bip32::DerivationPath, Amount},
    btc_heritage::{
        heritage_config::HeritageExplorerTrait, utils::timestamp_now, AccountXPub, HeritageConfig,
    },
    heritage_service_api_client::{Fingerprint, SubwalletConfigMeta},
    DatabaseItem, OnlineWallet, Wallet,
};

use crate::utils::{CCStr, CheapClone};

pub fn use_resource_wallet_subwallet_configs(
    wallet: AsyncSignal<Wallet>,
) -> FResource<CheapClone<[SubwalletConfigMeta]>> {
    use_resource(move || async move {
        log::debug!("use_resource_wallet_subwallet_configs - start");

        let wallet_heritage_configs = wallet
            .with(async |wallet| {
                let wallet_name = wallet.name().to_owned();
                wallet
                    .list_subwallet_configs()
                    .await
                    .map_err(|e| {
                        let error = format!(
                            "Error retrieving the wallet HeritageConfigs of wallet {}: {e}",
                            wallet_name
                        );
                        log::error!("{error}");
                        CCStr::from(error)
                    })
                    .map(Into::into)
            })
            .await;
        log::debug!("use_resource_wallet_subwallet_configs - loaded");
        wallet_heritage_configs
    })
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ExpirationStatus {
    Current,
    Outdated,
    ExpireSoon,
    Expired,
}
impl ExpirationStatus {
    // SOON = 1 month
    // 30 days x 24 hours x 60 mins x 60 secs
    pub const SOON: u64 = 30 * 24 * 60 * 60;
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HeritageConfigWithInfo {
    pub account_xpub: CheapClone<AccountXPub>,
    pub heritage_config: CheapClone<HeritageConfig>,
    pub firstuse_ts: Option<u64>,
    pub expiration_ts: Option<u64>,
    pub expiration_status: ExpirationStatus,
    pub balance: LResult<Amount>,
}

pub fn use_memo_heritage_configs_with_info(
    wallet_subwallet_configs: FResource<CheapClone<[SubwalletConfigMeta]>>,
    balance_by_heritage_config: FMemo<HashMap<HeritageConfig, Amount>>,
) -> FMemo<CheapClone<[HeritageConfigWithInfo]>> {
    let memo_balances = use_memo(move || {
        log::debug!("use_memo_heritage_config_with_info_balances - start compute");

        let balances = wallet_subwallet_configs
            .lrmap_ok(|wallet_subwallet_configs| {
                wallet_subwallet_configs
                    .iter()
                    .map(|swcm| {
                        balance_by_heritage_config.lrmap(|balance_by_heritage_config| {
                            balance_by_heritage_config
                                .get(&swcm.heritage_config)
                                .cloned()
                                .unwrap_or_default()
                        })
                    })
                    .collect::<Vec<_>>()
            })
            .unwrap_or_default();
        log::debug!("use_memo_heritage_config_with_info_balances - finish compute");
        balances
    });
    let memo_expiration_ts = use_memo(move || {
        log::debug!("use_memo_heritage_config_with_info_expiration_ts - start compute");

        let expiration_ts = wallet_subwallet_configs
            .lrmap_ok(|wallet_subwallet_configs| {
                wallet_subwallet_configs
                    .iter()
                    .map(|swcm| {
                        let heritage_config = &swcm.heritage_config;
                        heritage_config
                            .iter_heir_configs()
                            .take(1)
                            .map(|hc| {
                                heritage_config
                                    .get_heritage_explorer(hc)
                                    .expect("cannot be None as we are iterating heir_configs")
                                    .get_spend_conditions()
                                    .get_spendable_timestamp()
                                    .expect("always present for heirs")
                            })
                            .next()
                    })
                    .collect::<Vec<_>>()
            })
            .unwrap_or_default();
        log::debug!("use_memo_heritage_config_with_info_expiration_ts - finish compute");
        expiration_ts
    });
    use_memo(move || {
        log::debug!("use_memo_heritage_config_with_info - start compute");

        let now = timestamp_now();

        let heritage_config_with_info =
            wallet_subwallet_configs.lrmap(|wallet_subwallet_configs| {
                wallet_subwallet_configs
                    .iter()
                    .zip(&*memo_expiration_ts.read())
                    .zip(&*memo_balances.read())
                    .enumerate()
                    .map(|(idx, ((swcm, expiration_ts), balance))| {
                        let expiration_ts = *expiration_ts;
                        let balance = balance.clone();
                        let account_xpub = CheapClone::new(swcm.account_xpub.clone());
                        let heritage_config = CheapClone::new(swcm.heritage_config.clone());
                        let firstuse_ts = swcm.firstuse_ts;
                        let is_newest = idx == 0;
                        let expiration_status = if expiration_ts.is_some_and(|ts| ts < now) {
                            ExpirationStatus::Expired
                        } else if expiration_ts.is_some_and(|ts| ts < now + ExpirationStatus::SOON)
                        {
                            ExpirationStatus::ExpireSoon
                        } else if is_newest {
                            ExpirationStatus::Current
                        } else {
                            ExpirationStatus::Outdated
                        };
                        HeritageConfigWithInfo {
                            account_xpub,
                            heritage_config,
                            firstuse_ts,
                            expiration_ts,
                            expiration_status,
                            balance,
                        }
                    })
                    .collect::<_>()
            });
        log::debug!("use_memo_heritage_config_with_info - finish compute");
        heritage_config_with_info
    })
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct AccountXPubOrigin {
    fingerprint: Fingerprint,
    derivation_path: DerivationPath,
}
impl TryFrom<&(Fingerprint, DerivationPath)> for AccountXPubOrigin {
    type Error = &'static str;

    fn try_from((fingerprint, dp): &(Fingerprint, DerivationPath)) -> Result<Self, Self::Error> {
        if dp.len() < 3 {
            return Err("Derivation Path too short");
        }
        // Make sure the Derivation Path is level3, all hardened
        let derivation_path = dp
            .into_iter()
            .take(3)
            .map(|c| {
                if c.is_hardened() {
                    Ok(*c)
                } else {
                    Err("Derivation Path has non-hardened children")
                }
            })
            .collect::<Result<_, _>>()?;

        Ok(Self {
            fingerprint: *fingerprint,
            derivation_path,
        })
    }
}

pub fn use_memo_heritage_configs_with_info_indexed_by_origin_info(
    heritage_configs_with_info: FMemo<CheapClone<[HeritageConfigWithInfo]>>,
) -> FMemo<HashMap<AccountXPubOrigin, HeritageConfigWithInfo>> {
    use_memo(move || {
        log::debug!("use_memo_heritage_configs_with_info_indexed_by_origin_info - start compute");

        let heritage_configs_with_info_indexed_by_origin_info =
            heritage_configs_with_info.lrmap(|heritage_configs_with_info| {
                heritage_configs_with_info
                    .iter()
                    .map(|heritage_config_with_info| {
                        let axpub_desckey = heritage_config_with_info
                            .account_xpub
                            .descriptor_public_key();
                        let origin_info = AccountXPubOrigin {
                            fingerprint: axpub_desckey.master_fingerprint(),
                            derivation_path: axpub_desckey
                                .full_derivation_path()
                                .expect("not multipath"),
                        };
                        (origin_info, heritage_config_with_info.clone())
                    })
                    .collect()
            });
        log::debug!("use_memo_heritage_configs_with_info_indexed_by_origin_info - finish compute");
        heritage_configs_with_info_indexed_by_origin_info
    })
}

pub fn use_memo_heritage_configs_with_info_indexed_by_heritage_config(
    heritage_configs_with_info: FMemo<CheapClone<[HeritageConfigWithInfo]>>,
) -> FMemo<HashMap<CheapClone<HeritageConfig>, HeritageConfigWithInfo>> {
    use_memo(move || {
        log::debug!(
            "use_memo_heritage_configs_with_info_indexed_by_heritage_config - start compute"
        );

        let heritage_configs_with_info_indexed_by_origin_info =
            heritage_configs_with_info.lrmap(|heritage_configs_with_info| {
                heritage_configs_with_info
                    .iter()
                    .map(|heritage_config_with_info| {
                        let heritage_config = heritage_config_with_info.heritage_config.clone();
                        (heritage_config, heritage_config_with_info.clone())
                    })
                    .collect()
            });
        log::debug!(
            "use_memo_heritage_configs_with_info_indexed_by_heritage_config - finish compute"
        );
        heritage_configs_with_info_indexed_by_origin_info
    })
}
