use btc_heritage_wallet::{btc_heritage::HeirConfig, AnyKeyProvider};
use dioxus::prelude::*;

use std::ops::Deref;

use crate::{
    components::misc::{Badge, HeirBadgeType, SkeletonBadgeType},
    helper_hooks::{
        use_memo_heirs, use_resource_database_heirs, use_resource_service_heirs, CompositeHeir,
    },
    utils::RcStr,
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

    let database_heirs = use_resource_database_heirs();
    let service_heirs = use_resource_service_heirs();
    let heirs = use_memo_heirs(database_heirs, service_heirs);

    let heirs = use_memo(move || {
        heirs
            .read()
            .values()
            .map(|composite_heir| {
                let CompositeHeir {
                    name,
                    heir_config,
                    db_heir,
                    service_heir,
                } = composite_heir;
                let name = name.clone();
                let heir_config_type = match heir_config.deref() {
                    HeirConfig::SingleHeirPubkey(_) => "Public Key",
                    HeirConfig::HeirXPubkey(_) => "Extended Public Key",
                };
                let heir_config_fingerprint = RcStr::from(heir_config.fingerprint().to_string());

                let database_badge = db_heir.iter().map(|_| HeirBadgeType::Database);
                let key_provider_badge = db_heir
                    .iter()
                    .map(|db_heir| match db_heir.key_provider() {
                        AnyKeyProvider::None => None,
                        AnyKeyProvider::LocalKey(_) => Some(HeirBadgeType::LocalKeyProvider),
                        AnyKeyProvider::Ledger(_) => Some(HeirBadgeType::LedgerKeyProvider),
                    })
                    .flatten();
                let service_badge = if let Some(_) = service_heir {
                    Some(HeirBadgeType::Service)
                } else {
                    None
                };
                let badges = database_badge
                    .chain(key_provider_badge)
                    .chain(service_badge.into_iter())
                    .collect();
                let service_heir_id = if service_heirs.read().is_some() {
                    Some(
                        service_heir
                            .as_ref()
                            .map(|service_heir| RcStr::from(&service_heir.id)),
                    )
                } else {
                    None
                };
                HeirItemProps {
                    name,
                    heir_config_type,
                    heir_config_fingerprint,
                    badges,
                    service_heir_id,
                }
            })
            .collect::<Vec<_>>()
    });

    use_drop(|| log::debug!("HeirList Dropped"));
    rsx! {
        div { class: "container mx-auto grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-3 xl:grid-cols-4 3xl:grid-cols-5 gap-12",
            for heir_item_prop in heirs() {
                div { class: "w-full aspect-square content-center",
                    HeirItem { ..heir_item_prop }
                }
            }
            if service_heirs().is_none() {
                div { class: "w-full aspect-square content-center", PlaceHolderHeirItem {} }
            }
        
        }
    }
}

#[component]
fn HeirItem(
    name: RcStr,
    heir_config_type: &'static str,
    heir_config_fingerprint: RcStr,
    badges: Vec<HeirBadgeType>,
    service_heir_id: Option<Option<RcStr>>,
) -> Element {
    log::debug!("HeirItem Rendered");

    let service_loading = service_heir_id.is_none();

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
                div { class: "flex flex-row flex-wrap justify-center gap-2",
                    for badge in badges {
                        Badge { badge }
                    }
                    if service_loading {
                        Badge { badge: SkeletonBadgeType }
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
                div { class: "flex flex-row flex-wrap gap-1",
                    Badge { badge: SkeletonBadgeType }
                    Badge { badge: SkeletonBadgeType }
                }
            }
        }
    }
}
