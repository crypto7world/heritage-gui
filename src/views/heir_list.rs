use btc_heritage_wallet::{btc_heritage::HeirConfig, AnyKeyProvider};
use dioxus::prelude::*;

use std::{collections::HashMap, ops::Deref, rc::Rc, str::FromStr, sync::Arc};

use crate::{
    helper_hooks::{use_resource_database_heirs, use_resource_service_heirs},
    state_management,
    utils::{EqRcType, LoadedElement, PlaceHolder, RcStr, RcType},
};

#[component]
pub fn HeirListView() -> Element {
    log::debug!("HeirListView Rendered");

    use_drop(|| log::debug!("HeirListView Dropped"));
    rsx! {
        super::TitledView {
            title: "Heirs",
            subtitle: "Heirs that you can reference in the Heritage configuration of your wallets.",
            HeirList {}
        }
    }
}

#[component]
fn HeirList() -> Element {
    log::debug!("HeirList Rendered");
    let database_service = state_management::use_database_service();
    let service_client_service = state_management::use_service_client_service();

    let database_heirs = use_resource_database_heirs();
    let service_heirs = use_resource_service_heirs();

    let database_heirs_index = use_memo(move || {
        let index: HashMap<RcType<HeirConfig>, EqRcType<btc_heritage_wallet::Heir>> =
            HashMap::new();
        if let Some(database_heirs) = database_heirs() {
            for database_heir in database_heirs {
                let heir_config = RcType::from(database_heir.heir_config.clone());
                index.insert(heir_config, database_heir.into());
            }
        }
        index
    });

    let heir_item_props = use_resource(move || async move {
        log::debug!("use_resource_heir_item_props - start");
        let mut heir_item_props = state_management::list_heirs(database_service)
            .await
            .unwrap_or_default()
            .into_iter()
            .map(|aheir| {
                let db_heir = Arc::into_inner(aheir).expect("should be the only one");
                let name = RcStr::from_str(&db_heir.name).unwrap();
                let heir_config = Rc::new(db_heir.heir_config.clone());
                let db_heir = Some(RcType::from(db_heir));
                (
                    heir_config.clone(),
                    HeirItemProps {
                        name,
                        heir_config,
                        db_heir,
                        service_heir: None,
                    },
                )
            })
            .collect::<HashMap<_, _>>();

        // Subscribe to the service connection
        let _ = *state_management::CONNECTED_USER.read();
        let service_client =
            state_management::heritage_service_client(service_client_service).await;
        for service_heir in service_client.list_heirs().await.unwrap_or_default() {
            let heir_config = Rc::new(service_heir.heir_config.clone());
            let service_heir = RcType::from(service_heir);
            heir_item_props
                .entry(heir_config.clone())
                .and_modify(|hip| hip.service_heir = Some(service_heir.clone()))
                .or_insert_with(|| {
                    let name = RcStr::from_str(&service_heir.display_name).unwrap();
                    HeirItemProps {
                        name,
                        heir_config,
                        db_heir: None,
                        service_heir: Some(service_heir),
                    }
                });
        }

        log::debug!("use_resource_heir_item_props - loaded");
        heir_item_props.into_values().collect::<Vec<_>>()
    });

    use_drop(|| log::debug!("HeirList Dropped"));
    rsx! {
        div { class: "container mx-auto grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-3 xl:grid-cols-4 3xl:grid-cols-5 gap-12",
            if let Some(heir_item_props) = heir_item_props.cloned() {
                for heir_item_prop in heir_item_props {
                    div { class: "w-full aspect-square content-center",
                        HeirItem { ..heir_item_prop }
                    }
                }
            }
        }
    }
}

#[component]
fn HeirItem(
    name: RcStr,
    heir_config: Rc<HeirConfig>,
    db_heir: Option<RcType<btc_heritage_wallet::Heir>>,
    service_heir: Option<RcType<btc_heritage_wallet::heritage_service_api_client::Heir>>,
) -> Element {
    log::debug!("HeirItem Rendered");

    let hc_type = match heir_config.deref() {
        HeirConfig::SingleHeirPubkey(_) => "Public Key",
        HeirConfig::HeirXPubkey(_) => "Extended Public Key",
    };
    let fingerprint = heir_config.fingerprint().to_string();

    let key_provider = db_heir
        .map(|h| match h.key_provider() {
            AnyKeyProvider::None => None,
            AnyKeyProvider::LocalKey(_) => Some(("Local Key", "badge-secondary")),
            AnyKeyProvider::Ledger(_) => Some(("Ledger", "badge-secondary")),
        })
        .flatten();
    let exported_to_service = service_heir.is_some();

    use_drop(|| log::debug!("HeirItem Dropped"));
    rsx! {
        div { class: "card card-lg border shadow-xl size-fit mx-auto",
            div { class: "card-body aspect-square w-auto max-h-fit",
                div { class: "card-title text-3xl font-black", "{name}" }

                div { class: "flex flex-col",
                    div { class: "font-light", "Type" }
                    div { class: "text-lg font-bold text-nowrap", "{hc_type}" }
                }
                div { class: "flex flex-col",
                    div { class: "font-light text-nowrap", "Key Fingerprint" }
                    div { class: "text-lg font-bold", "{fingerprint}" }
                }
                div { class: "grow" }
                div { class: "mx-auto grid grid-cols-2 gap-6",
                    if let Some((content, color)) = key_provider {
                        div { class: "col-start-1 badge shadow-xl text-nowrap {color}",
                            {content}
                        }
                    }
                    if exported_to_service {
                        div { class: "col-start-2 badge shadow-xl text-nowrap badge-success",
                            "Service"
                        }
                    }
                }
            
            }
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
struct ServiceHeirId(Option<RcStr>);
impl PlaceHolder for ServiceHeirId {
    fn place_holder() -> Self {
        Self(Some(RcStr::from_str("123456").unwrap()))
    }
}

#[component]
fn HeirItem2(
    name: RcStr,
    heir_config_type: &'static str,
    heir_config_fingerprint: RcStr,
    key_provider_badge: Option<(&'static str, &'static str)>,
    service_heir_id: LoadedElement<ServiceHeirId>,
) -> Element {
    log::debug!("HeirItem Rendered");
    let (is_place_holder, service_heir_id) = service_heir_id.extract();

    use_drop(|| log::debug!("HeirItem Dropped"));
    rsx! {
        div { class: "card card-lg border shadow-xl size-fit mx-auto",
            div { class: "card-body aspect-square w-auto max-h-fit",

                div { class: "card-title text-3xl font-black", {name} }
                div { class: "flex flex-col",
                    div { class: "font-light", "Type" }
                    div { class: "text-lg font-bold text-nowrap", {heir_config_type} }
                }
                div { class: "flex flex-col",
                    div { class: "font-light text-nowrap", "Key Fingerprint" }
                    div { class: "text-lg font-bold", {heir_config_fingerprint} }
                }

                div { class: "grow" }
                div { class: "mx-auto grid grid-cols-2 gap-6",
                    if let Some((content, color)) = key_provider_badge {
                        div { class: "col-start-1 badge shadow-xl text-nowrap {color}",
                            {content}
                        }
                    }
                    if let ServiceHeirId(Some(_service_heir_id)) = service_heir_id {
                        div {
                            class: "col-start-2 badge shadow-xl text-nowrap badge-success",
                            class: if is_place_holder { "skeleton text-transparent" },
                            "Service"
                        }
                    }
                
                }
            }
        }
    }
}

#[component]
fn PlaceHolderHeirItem() -> Element {
    log::debug!("PlaceHolderHeirItem Rendered");
    use_drop(|| log::debug!("PlaceHolderHeirItem Dropped"));
    rsx! {
        div { class: "card card-lg border shadow-xl size-fit mx-auto",
            div { class: "card-body aspect-square w-auto max-h-fit",
                div { class: "card-title text-3xl font-black skeleton text-transparent",
                    "some_name"
                }
                div { class: "flex flex-col",
                    div { class: "font-light skeleton text-transparent", "Type" }
                    div { class: "text-lg font-bold text-nowrap skeleton text-transparent",
                        "Extended Public Key"
                    }
                }
                div { class: "flex flex-col",
                    div { class: "font-light text-nowrap skeleton text-transparent",
                        "Key Fingerprint"
                    }
                    div { class: "text-lg font-bold skeleton text-transparent", "12345678" }
                }

                div { class: "grow" }
                div { class: "mx-auto grid grid-cols-2 gap-6",
                    div { class: "col-start-1 badge shadow-xl text-nowrap badge-secondary skeleton text-transparent",
                        "Local Key"
                    }
                    div { class: "col-start-2 badge shadow-xl text-nowrap badge-success skeleton text-transparent",
                        "Service"
                    }
                }
            }
        }
    }
}
