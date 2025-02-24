use dioxus::prelude::*;

use crate::{
    state_management::{self, use_database_service},
    utils::RcStr,
};

pub fn use_resource_heirwallet_names() -> Resource<Vec<RcStr>> {
    let database_service = use_database_service();
    use_resource(move || async move {
        log::debug!("use_resource_heir_names - start");
        let heir_wallet_names = state_management::list_heir_wallet_names(database_service)
            .await
            .unwrap_or_default();
        log::debug!("use_resource_heir_names - loaded");
        heir_wallet_names
    })
}
