use crate::prelude::*;

use std::collections::HashMap;

use btc_heritage_wallet::{
    btc_heritage::HeirConfig, heritage_service_api_client::Heir as ServiceHeir, Heir as DbHeir,
};

use crate::utils::{log_error_ccstr, CCStr, CheapClone, EqCheapClone};

pub fn use_resource_database_heirs() -> Resource<Vec<CheapClone<DbHeir>>> {
    let database_service = state_management::use_database_service();
    use_resource(move || async move {
        log::debug!("use_resource_database_heirs - start");
        let heirs = state_management::list_heirs(database_service)
            .await
            .unwrap_or_default();
        log::debug!("use_resource_database_heirs - loaded");
        heirs
    })
}

pub fn use_async_heir(name: CCStr) -> AsyncSignal<DbHeir> {
    let database_service = state_management::use_database_service();
    helper_hooks::use_async_init(move || {
        let name = name.clone();
        async move {
            log::debug!("use_async_heir - start");
            let heir = state_management::get_heir(database_service, name)
                .await
                .expect("should exist and I have nothing smart to do with this error anyway");
            log::debug!("use_async_heir - loaded");
            heir
        }
    })
}

pub fn use_resource_service_heirs() -> FResource<Vec<CheapClone<ServiceHeir>>> {
    let service_client_service = state_management::use_service_client_service();
    use_resource(move || async move {
        log::debug!("use_resource_service_heirs - start");

        // Read the SERVICE_STATUS so that we refresh when the SERVICE_STATUS is refreshed
        let _ = *state_management::SERVICE_STATUS.read();

        let service_client =
            state_management::heritage_service_client(service_client_service).await;

        let heirs = service_client
            .list_heirs()
            .await
            .map_err(log_error_ccstr)
            .map(|heirs| heirs.into_iter().map(CheapClone::from).collect());

        log::debug!("use_resource_service_heirs - loaded");

        heirs
    })
}

#[derive(Debug, Clone, PartialEq)]
pub struct CompositeHeir {
    pub name: CCStr,
    pub heir_config: CheapClone<HeirConfig>,
    pub db_heir: Option<EqCheapClone<DbHeir>>,
    pub service_heir: Option<Option<EqCheapClone<ServiceHeir>>>,
}

pub fn use_memo_heirs(
    database_heirs: Resource<Vec<CheapClone<DbHeir>>>,
    service_heirs: FResource<Vec<CheapClone<ServiceHeir>>>,
) -> Memo<Vec<CompositeHeir>> {
    use_memo(move || {
        log::debug!("use_memo_heirs - start");
        let mut heirs = if let Some(ref database_heirs) = *database_heirs.read() {
            database_heirs
                .into_iter()
                .map(|db_heir| {
                    let name = CCStr::from(&db_heir.name);

                    let heir_config = CheapClone::from(db_heir.heir_config.clone());

                    (
                        heir_config.clone(),
                        CompositeHeir {
                            name,
                            heir_config,
                            db_heir: Some(db_heir.clone().into()),
                            service_heir: None,
                        },
                    )
                })
                .collect::<HashMap<_, _>>()
        } else {
            Default::default()
        };
        if let Some(Ok(ref service_heirs)) = *service_heirs.read() {
            for service_heir in service_heirs {
                heirs
                    .entry(CheapClone::from(service_heir.heir_config.clone()))
                    .and_modify(|composite_heir| {
                        composite_heir.service_heir = Some(Some(service_heir.clone().into()));
                    })
                    .or_insert_with(|| {
                        let name = CCStr::from(&service_heir.display_name);
                        let heir_config = CheapClone::from(service_heir.heir_config.clone());
                        CompositeHeir {
                            name,
                            heir_config,
                            db_heir: None,
                            service_heir: Some(Some(service_heir.clone().into())),
                        }
                    });
            }
            // Service is loaded, so we need to ensure service_heir file is Some(...)
            for (_, heir) in heirs.iter_mut() {
                if heir.service_heir.is_none() {
                    heir.service_heir = Some(None);
                }
            }
        }
        let mut heirs = heirs.into_iter().map(|(_, ch)| ch).collect::<Vec<_>>();
        heirs.sort_by_key(|ch| ch.name.clone());
        log::debug!("use_memo_heirs - loaded");
        heirs
    })
}
