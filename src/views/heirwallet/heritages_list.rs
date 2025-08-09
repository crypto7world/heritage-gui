use crate::prelude::*;

use std::collections::HashMap;

use crate::{
    components::{
        badge::{ExternalDependencyStatus, HeritageProviderType},
        heritages::UIHeritage,
    },
    utils::CCStr,
};

#[component]
pub(super) fn HeritagesList() -> Element {
    log::debug!("HeritagesList Rendered");

    let heritage_provider_status =
        use_context::<Memo<Option<(HeritageProviderType, ExternalDependencyStatus)>>>();
    let heirwallet_contextualized_heritages =
        use_context::<FMemo<HashMap<CCStr, ContextualizedHeritages>>>();

    let heritage_to_display_count = use_memo(move || {
        heirwallet_contextualized_heritages
            .lrmap_ok(|h| h.len())
            .unwrap_or_default()
    });

    let message = use_memo(move || {
        match heritage_provider_status() {
            Some((HeritageProviderType::Service, _)) => {
                "Note that the Heritage Service does not \
            necessarily return immature inheritances, depending on the permissions the owner gave you."
            }
            Some((HeritageProviderType::LocalWallet, _)) => {
                "The heir wallet may not be synchronized with the blockchain."
            }
            Some((HeritageProviderType::None, _)) => {
                "The heir wallet has no online capability and cannot see inheritances; \
            it may only sign transactions."
            }
            None => "",
        }
    });

    use_drop(|| log::debug!("HeritagesList Dropped"));

    rsx! {
        div { class: "rounded-box border border-base-content/5 shadow-md bg-base-100 my-4 max-w-7xl mx-auto",
            // Title
            h2 { class: "text-2xl font-bold p-4", "Inheritances List" }
            div { class: "p-4",
                if heritage_to_display_count() == 0 {
                    div { "No inheritances at this time" }
                    div { class: "w-sm", {message} }
                } else {
                    div { class: "flex flex-col gap-6",
                        LoadedComponent::<HashMap<CCStr,UIHeritage>> { input: heirwallet_contextualized_heritages.into() }
                    }
                }
            }
        }
    }
}
