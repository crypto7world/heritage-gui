use dioxus::prelude::*;

use std::collections::HashMap;

use btc_heritage_wallet::Heir as DbHeir;
use btc_heritage_wallet::{
    btc_heritage::HeirConfig, heritage_service_api_client::Heir as ServiceHeir,
};

use crate::{
    state_management::{self, use_database_service, use_service_client_service},
    utils::{EqRcType, RcStr, RcType},
};

pub fn use_resource_database_heirs() -> Resource<Vec<RcType<DbHeir>>> {
    let database_service = use_database_service();
    use_resource(move || async move {
        log::debug!("use_resource_database_heirs - start");
        let heirs = state_management::list_heirs(database_service)
            .await
            .unwrap_or_default();
        log::debug!("use_resource_database_heirs - loaded");
        heirs
    })
}

pub fn use_resource_service_heirs() -> Resource<Vec<RcType<ServiceHeir>>> {
    let service_client_service = use_service_client_service();
    use_resource(move || async move {
        log::debug!("use_resource_service_heirs - start");
        let _ = *state_management::CONNECTED_USER.read();
        let service_client =
            state_management::heritage_service_client(service_client_service).await;
        let heirs = service_client
            .list_heirs()
            .await
            .unwrap_or_default()
            .into_iter()
            .map(RcType::from)
            .collect();
        log::debug!("use_resource_service_heirs - loaded");
        heirs
    })
}

#[derive(Debug, Clone, PartialEq)]
pub struct CompositeHeir {
    pub name: RcStr,
    pub heir_config: RcType<HeirConfig>,
    pub db_heir: Option<EqRcType<DbHeir>>,
    pub service_heir: Option<EqRcType<ServiceHeir>>,
}

pub fn use_memo_heirs(
    database_heirs: Resource<Vec<RcType<DbHeir>>>,
    service_heirs: Resource<Vec<RcType<ServiceHeir>>>,
) -> Memo<HashMap<RcType<HeirConfig>, CompositeHeir>> {
    use_memo(move || {
        log::debug!("use_memo_heirs - start");
        let mut heirs = if let Some(ref database_heirs) = *database_heirs.read() {
            database_heirs
                .into_iter()
                .map(|db_heir| {
                    let name = RcStr::from(&db_heir.name);

                    let heir_config = RcType::from(db_heir.heir_config.clone());

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
        if let Some(ref service_heirs) = *service_heirs.read() {
            for service_heir in service_heirs {
                heirs
                    .entry(RcType::from(service_heir.heir_config.clone()))
                    .and_modify(|composite_heir| {
                        composite_heir.service_heir = Some(service_heir.clone().into());
                    })
                    .or_insert_with(|| {
                        let name = RcStr::from(&service_heir.display_name);
                        let heir_config = RcType::from(service_heir.heir_config.clone());
                        CompositeHeir {
                            name,
                            heir_config,
                            db_heir: None,
                            service_heir: Some(service_heir.clone().into()),
                        }
                    });
            }
        }
        log::debug!("use_memo_heirs - loaded");
        heirs
    })
}
