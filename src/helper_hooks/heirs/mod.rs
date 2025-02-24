use dioxus::prelude::*;

use std::sync::Arc;

use crate::{
    state_management::{self, use_database_service, use_service_client_service},
    utils::RcType,
};

pub fn use_resource_database_heirs() -> Resource<Vec<RcType<btc_heritage_wallet::Heir>>> {
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

pub fn use_resource_service_heirs(
) -> Resource<Vec<RcType<btc_heritage_wallet::heritage_service_api_client::Heir>>> {
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
