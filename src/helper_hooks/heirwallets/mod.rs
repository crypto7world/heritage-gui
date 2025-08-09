use crate::prelude::*;

use std::collections::{HashMap, HashSet};

use btc_heritage_wallet::{
    AnyHeritageProvider, BoundFingerprint, DatabaseItem, HeirWallet, Heritage, HeritageProvider,
    OnlineWallet,
};

use crate::{
    components::badge::{ExternalDependencyStatus, HeritageProviderType, KeyProviderType},
    utils::{CCStr, CheapClone, EqCheapClone},
};

pub fn use_resource_heirwallet_names() -> Resource<Vec<CCStr>> {
    let database_service = state_management::use_database_service();
    use_resource(move || async move {
        log::debug!("use_resource_heirwallet_names - start");
        let heirwallet_names = state_management::list_heirwallet_names(database_service)
            .await
            .unwrap_or_default();
        log::debug!("use_resource_heirwallet_names - loaded");
        heirwallet_names
    })
}

pub fn use_async_heirwallet(name: CCStr) -> AsyncSignal<HeirWallet> {
    let database_service = state_management::use_database_service();
    let service_client_service = state_management::use_service_client_service();
    helper_hooks::use_async_init(move || {
        let name = name.clone();
        async move {
            log::debug!("use_async_heirwallet - start");
            let wallet =
                state_management::get_heirwallet(database_service, service_client_service, name)
                    .await
                    .expect("should exist and I have nothing smart to do with this error anyway");
            log::debug!("use_async_heirwallet - loaded");
            wallet
        }
    })
}

pub fn use_memo_heirwallet_fingerprint(heirwallet: AsyncSignal<HeirWallet>) -> Memo<CCStr> {
    use_memo(move || {
        log::debug!("use_memo_heirwallet_fingerprint - start compute");
        let fingerprint = heirwallet
            .lmap(|heirwallet| {
                heirwallet
                    .fingerprint()
                    .map(|fg| fg.to_string().into())
                    .unwrap_or_else(|e| {
                        log::warn!("{e}");
                        "-".into()
                    })
            })
            .unwrap_or_else(|| CCStr::from("Loading..."));
        log::debug!("use_memo_heirwallet_fingerprint - finish compute");
        fingerprint
    })
}

pub fn use_resource_heirwallet_heritages(
    heirwallet: AsyncSignal<HeirWallet>,
) -> FResource<HashMap<CCStr, Vec<CheapClone<Heritage>>>> {
    use_resource(move || async move {
        log::debug!("use_resource_heirwallet_heritages - start");

        let heirwallet_heritages = heirwallet
            .with(async |heirwallet: &HeirWallet| {
                heirwallet.list_heritages().await.map_err(|e| {
                    let error = format!(
                        "Error retrieving the heritages of {}: {e}",
                        heirwallet.name()
                    );
                    log::error!("{error}");
                    CCStr::from(error)
                })
            })
            .await;

        let heirwallet_heritages = heirwallet_heritages.map(|heirwallet_heritages| {
            heirwallet_heritages
                .into_iter()
                .fold(HashMap::new(), |mut hm, heritage| {
                    hm.entry(CCStr::from(heritage.heritage_id.as_str()))
                        .or_insert(Vec::new())
                        .push(CheapClone::from(heritage));
                    hm
                })
        });

        log::debug!("use_resource_heirwallet_heritages - loaded");

        heirwallet_heritages
    })
}
pub fn use_memo_heirwallet_contextualized_heritages(
    heirwallet: AsyncSignal<HeirWallet>,
    heirwallet_heritages: FResource<HashMap<CCStr, Vec<CheapClone<btc_heritage_wallet::Heritage>>>>,
    service_heritages: FResource<HashMap<CCStr, ContextualizedHeritages>>,
) -> FMemo<HashMap<CCStr, ContextualizedHeritages>> {
    use_memo(move || {
        log::debug!("use_memo_heirwallet_contextualized_heritages - start");

        let heritage_provider_is_service = heirwallet
            .lmap(|heirwallet| {
                matches!(
                    heirwallet.heritage_provider(),
                    AnyHeritageProvider::Service(_)
                )
            })
            .unwrap_or_default();

        let opt_res_h = &*service_heritages.read();
        let get_owner = |heritage_id: &CCStr| match opt_res_h {
            Some(Ok(h)) => match h.get(heritage_id) {
                Some(ContextualizedHeritages {
                    context: HeritageContext::Service { owner },
                    ..
                }) => owner.clone(),
                _ => None,
            },
            _ => None,
        };

        let heirwallet_name = heirwallet
            .lmap(|heirwallet| CCStr::from(heirwallet.name()))
            .unwrap_or_default();

        let heirwallet_contextualized_heritages = heirwallet_heritages.lrmap(|h| {
            h.iter()
                .map(|(heritage_id, heritages)| {
                    let spend_infos = (heirwallet_name.clone(), heritage_id.clone());
                    let context = if heritage_provider_is_service {
                        HeritageContext::WalletService {
                            spend_infos,
                            owner: get_owner(heritage_id),
                        }
                    } else {
                        HeritageContext::WalletLocal { spend_infos }
                    };
                    let heritages = heritages.iter().cloned().map(EqCheapClone::from).collect();
                    (
                        heritage_id.clone(),
                        ContextualizedHeritages { context, heritages },
                    )
                })
                .collect()
        });

        log::debug!("use_memo_heirwallet_contextualized_heritages - loaded");

        heirwallet_contextualized_heritages
    })
}

/// Only return Some for Local HeritageProvider, as the Heritage Service sync do not depends on the Heir Wallet at all.
pub fn use_resource_heirwallet_local_lastsync(
    heirwallet: AsyncSignal<HeirWallet>,
) -> FResource<Option<u64>> {
    use_resource(move || async move {
        log::debug!("use_resource_heirwallet_lastsync - start");

        let heirwallet_lastsync = heirwallet
            .with(
                async |heirwallet: &HeirWallet| match heirwallet.heritage_provider() {
                    AnyHeritageProvider::LocalWallet(lw) => Some(
                        lw.local_heritage_wallet()
                            .get_wallet_status()
                            .await
                            .map(|ws| ws.last_sync_ts)
                            .map_err(|e| CCStr::from(e.to_string())),
                    ),
                    _ => None,
                },
            )
            .await
            .transpose();

        log::debug!("use_resource_heirwallet_lastsync - loaded");

        heirwallet_lastsync
    })
}

pub fn use_memo_heritage_provider_status(
    heirwallet: AsyncSignal<HeirWallet>,
) -> Memo<Option<(HeritageProviderType, ExternalDependencyStatus)>> {
    use_memo(move || {
        log::debug!("use_memo_heritage_provider_status - start compute");
        let result = heirwallet.lmap(|heirwallet| match heirwallet.heritage_provider() {
            AnyHeritageProvider::None => {
                (HeritageProviderType::None, ExternalDependencyStatus::None)
            }
            AnyHeritageProvider::Service(service_binding) => (
                HeritageProviderType::Service,
                match state_management::SERVICE_STATUS.read().as_ref() {
                    Some(ss) if ss.can_serve_heritage(service_binding) => {
                        ExternalDependencyStatus::Available
                    }
                    _ => ExternalDependencyStatus::Unavailable,
                },
            ),
            AnyHeritageProvider::LocalWallet(_) => (
                HeritageProviderType::LocalWallet,
                match state_management::BLOCKCHAIN_PROVIDER_STATUS() {
                    Some(BlockchainProviderStatus::Connected(_)) => {
                        ExternalDependencyStatus::Available
                    }
                    _ => ExternalDependencyStatus::Unavailable,
                },
            ),
        });
        log::debug!("use_memo_heritage_provider_status - finish compute");
        result
    })
}
pub fn use_memo_heirwallet_keyprovider_status(
    heirwallet: AsyncSignal<HeirWallet>,
) -> Memo<Option<(KeyProviderType, ExternalDependencyStatus)>> {
    use_memo(move || {
        log::debug!("use_memo_heirwallet_keyprovider_status - start compute");
        let result = heirwallet
            .lmap(|heirwallet| super::utils::keyprovider_status(heirwallet.key_provider(), None));
        log::debug!("use_memo_heirwallet_keyprovider_status - finish compute");
        result
    })
}

#[derive(Debug, Clone, PartialEq)]
pub enum HeritageContext {
    WalletLocal {
        spend_infos: (CCStr, CCStr),
    },
    WalletService {
        spend_infos: (CCStr, CCStr),
        owner: Option<CCStr>,
    },
    Service {
        owner: Option<CCStr>,
    },
}
#[derive(Debug, PartialEq)]
pub struct ContextualizedHeritages {
    pub context: HeritageContext,
    pub heritages: Vec<EqCheapClone<btc_heritage_wallet::Heritage>>,
}
pub fn use_resource_service_heritages() -> FResource<HashMap<CCStr, ContextualizedHeritages>> {
    let service_client_service = state_management::use_service_client_service();
    use_resource(move || async move {
        log::debug!("use_resource_service_heritages - start");

        // Read the SERVICE_STATUS so that we refresh when the SERVICE_STATUS is refreshed
        let _ = *state_management::SERVICE_STATUS.read();

        let heritage_service =
            state_management::heritage_service_client(service_client_service).await;
        let service_heritages = heritage_service
            .list_heritages()
            .await
            .map(|heritages| {
                heritages
                    .into_iter()
                    .fold(HashMap::new(), |mut h, heritage| {
                        let heritage_id = CCStr::from(&heritage.heritage_id);
                        h.entry(heritage_id).or_insert_with(Vec::new).push(heritage);
                        h
                    })
                    .into_iter()
                    .map(|(heritage_id, heritages)| {
                        assert!(heritages
                            .windows(2)
                            .all(|pair| pair[0].owner_email == pair[1].owner_email));
                        let owner = heritages
                            .get(0)
                            .map(|h| h.owner_email.as_ref().map(CCStr::from))
                            .flatten();
                        (
                            heritage_id,
                            ContextualizedHeritages {
                                context: HeritageContext::Service { owner },
                                heritages: heritages
                                    .into_iter()
                                    .map(|h| {
                                        EqCheapClone::from(CheapClone::from(
                                            btc_heritage_wallet::Heritage::from(h),
                                        ))
                                    })
                                    .collect(),
                            },
                        )
                    })
                    .collect()
            })
            .map_err(|e| {
                log::error!("Error querying heritages from service: {e}");
                CCStr::from(e.to_string())
            });
        log::debug!("use_resource_service_heritages - loaded");
        service_heritages
    })
}

pub fn use_memo_service_only_heritages(
    service_heritages: FResource<HashMap<CCStr, ContextualizedHeritages>>,
) -> FMemo<HashMap<CCStr, ContextualizedHeritages>> {
    let database_service = state_management::use_database_service();
    let db_wallet_fgs = use_resource(move || async move {
        let db_heirwallets = state_management::list_heirwallets(database_service)
            .await
            .unwrap_or_default();
        db_heirwallets
            .iter()
            .filter_map(|heirwallet| match heirwallet.heritage_provider() {
                AnyHeritageProvider::Service(service_binding) => Some(
                    service_binding
                        .fingerprint()
                        .expect("always present for heritage_provider sb"),
                ),
                _ => None,
            })
            .collect::<HashSet<_>>()
    });
    use_memo(move || {
        log::debug!("use_memo_heritage_provider_status - start compute");

        let db_wallet_fgs_ref = &*db_wallet_fgs.read();
        let contain_fg = |fg| match db_wallet_fgs_ref {
            Some(hs) => hs.contains(&fg),
            None => false,
        };

        let service_only_heritages = service_heritages.lrmap(|h| {
            h.iter()
                .filter_map(|(heritage_id, contextualized_heritages)| {
                    let heritages = contextualized_heritages
                        .heritages
                        .iter()
                        .filter(|h| !contain_fg(h.heir_config.fingerprint()))
                        .cloned()
                        .collect::<Vec<_>>();
                    (!heritages.is_empty()).then(|| {
                        (
                            heritage_id.clone(),
                            ContextualizedHeritages {
                                context: contextualized_heritages.context.clone(),
                                heritages,
                            },
                        )
                    })
                })
                .collect::<HashMap<CCStr, ContextualizedHeritages>>()
        });
        log::debug!("use_memo_heritage_provider_status - finish compute");
        service_only_heritages
    })
}
