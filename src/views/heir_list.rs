use btc_heritage_wallet::btc_heritage::HeirConfig;
use dioxus::prelude::*;

use std::ops::Deref;

use crate::{
    components::loaded::{
        badge::UIHeirBadges, ComponentMapper, FromRef, ImplDirectIntoLoadedElementInputMarker,
        LoadedComponent, LoadedElement,
    },
    helper_hooks::{
        use_memo_heirs, use_resource_database_heirs, use_resource_service_heirs, CompositeHeir,
    },
    utils::{ArcStr, ArcType},
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
    let heirs_map = use_memo_heirs(database_heirs, service_heirs);
    let service_loading = service_heirs.read().is_none();
    let heirs = use_memo(move || {
        let service_loading = service_heirs.read().is_none();
        heirs_map()
            .values()
            .cloned()
            .map(|ch| (ch, service_loading))
            .collect::<ArcType<[_]>>()
    });

    use_drop(|| log::debug!("HeirList Dropped"));
    rsx! {
        div { class: "container mx-auto grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-3 xl:grid-cols-4 3xl:grid-cols-5 gap-12",
            LoadedComponent::<ArcType<[UIHeirItem]>> { input: heirs.into() }
            if service_loading {
                LoadedComponent::<UIHeirItem> { input: None::<UIHeirItem>.into() }
            }
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
struct UIHeirItem {
    name: ArcStr,
    heir_config_type: &'static str,
    heir_config_fingerprint: ArcStr,
    badges: UIHeirBadges,
    service_loading: bool,
}
impl ImplDirectIntoLoadedElementInputMarker for UIHeirItem {}
impl FromRef<(CompositeHeir, bool)> for UIHeirItem {
    fn from_ref((composite_heir, service_loading): &(CompositeHeir, bool)) -> Self {
        let badges = UIHeirBadges::from_ref(composite_heir);

        let CompositeHeir {
            name, heir_config, ..
        } = composite_heir;
        let name = name.clone();
        let heir_config_type = match heir_config.deref() {
            HeirConfig::SingleHeirPubkey(_) => "Public Key",
            HeirConfig::HeirXPubkey(_) => "Extended Public Key",
        };
        let heir_config_fingerprint = ArcStr::from(heir_config.fingerprint().to_string());

        Self {
            name,
            heir_config_type,
            heir_config_fingerprint,
            badges: badges.into(),
            service_loading: *service_loading,
        }
    }
}
impl LoadedElement for UIHeirItem {
    #[inline(always)]
    fn element<CM: ComponentMapper>(self, mapper: CM) -> Element {
        rsx! {
            div { class: "w-full aspect-square content-center",
                div { class: "card card-lg border shadow-xl size-fit mx-auto",
                    div { class: "card-body aspect-square w-auto max-h-fit",

                        div { class: "card-title text-3xl font-black",
                            LoadedComponent { input: mapper.map(self.name) }
                        }
                        div { class: "flex flex-col",
                            div { class: "font-light", "Type" }
                            div { class: "text-lg font-bold text-nowrap",
                                LoadedComponent { input: mapper.map(self.heir_config_type) }
                            }
                        }
                        div { class: "flex flex-col",
                            div { class: "font-light text-nowrap", "Key Fingerprint" }
                            div { class: "text-lg font-bold",
                                LoadedComponent { input: mapper.map(self.heir_config_fingerprint) }
                            }
                        }

                        div { class: "grow" }

                        div { class: "flex flex-row flex-wrap justify-center gap-2",
                            LoadedComponent { input: mapper.map(self.badges) }
                        }
                    }
                }
            }
        }
    }
    fn place_holder() -> Self {
        Self {
            name: ArcStr::place_holder(),
            heir_config_type: <&'static str>::place_holder(),
            heir_config_fingerprint: ArcStr::place_holder(),
            badges: UIHeirBadges::place_holder(),
            service_loading: false,
        }
    }
    #[inline(always)]
    fn visible_place_holder() -> bool {
        true
    }
}
