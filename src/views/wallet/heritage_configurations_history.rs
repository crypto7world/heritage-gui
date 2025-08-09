use crate::prelude::*;

use btc_heritage_wallet::bitcoin::Amount;

use crate::{
    components::{
        balance::UIBtcAmount,
        heritage_configuration::{UIExpirationBadge, UIHeritageConfig},
        timestamp::UITimestamp,
    },
    utils::CheapClone,
};

#[component]
pub(super) fn HeritageConfigurationsHistory() -> Element {
    log::debug!("HeritageConfigurationsHistory Rendered");

    let heritage_configs_with_info = use_context::<FMemo<CheapClone<[HeritageConfigWithInfo]>>>();

    use_drop(|| log::debug!("HeritageConfigurationsHistory Dropped"));
    rsx! {
        div { class: "max-h-[calc(100vh-var(--spacing)*32)] overflow-y-auto overflow-x-auto rounded-box border border-base-content/5 shadow-md bg-base-100 my-4",
            h2 { class: "sticky top-0 z-10 bg-base-100 text-2xl font-bold p-4",
                "Heritage Configurations History"
            }
            LoadedComponent::<CheapClone<[UIHeritageConfigurationsHistoryItem]>> { input: heritage_configs_with_info.into() }
        }
    }
}

#[derive(Clone, PartialEq)]
struct UIHeritageConfigurationsHistoryItem {
    expirationbadge: UIExpirationBadge,
    expiration: UITimestamp,
    firstuse: UITimestamp,
    heritage_config: UIHeritageConfig,
    associated_balance: LResult<UIBtcAmount>,
}
impl LoadedElement for UIHeritageConfigurationsHistoryItem {
    type Loader = TransparentLoader;
    #[inline(always)]
    fn element<M: LoadedComponentInputMapper>(self, m: M) -> Element {
        rsx! {
            div { class: "m-4",
                div { class: "collapse collapse-plus bg-base-100 border border-base-300",
                    input { r#type: "checkbox", tabindex: "-1" }
                    div { class: "collapse-title",
                        // div { class: "flex flex-row gap-8 items-center",
                        div { class: "grid grid-rows-1 grid-cols-[repeat(3,calc(var(--spacing)*32))_auto] gap-8 items-center",
                            div { class: "z-10",
                                LoadedComponent::<UIExpirationBadge> { input: m.map(self.expirationbadge) }
                            }
                            div { class: "flex flex-col",
                                div { class: "text-sm font-light", "First Use" }
                                div { class: "font-bold",
                                    LoadedComponent::<UITimestamp> { input: m.map(self.firstuse) }
                                }
                            }
                            div { class: "flex flex-col",
                                div { class: "text-sm font-light", "Expiration" }
                                div { class: "font-bold",
                                    LoadedComponent::<UITimestamp> { input: m.map(self.expiration) }
                                }
                            }
                            div { class: "flex flex-col",
                                div { class: "text-sm font-light", "Associated Balance" }
                                div { class: "font-bold",
                                    LoadedComponent::<UIBtcAmount> { input: m.lc_map(self.associated_balance.into()) }
                                }
                            }
                        }
                    }
                    div { class: "collapse-content",
                        StaticLoadedComponent { input: m.map(()),
                            div {
                                {self.expirationbadge.tooltip()}
                                "."
                            }
                        }
                        LoadedComponent { input: m.map(self.heritage_config) }
                    }
                }
            }
        }
    }
    fn place_holder() -> Self {
        Self {
            expirationbadge: UIExpirationBadge::place_holder(),
            expiration: UITimestamp::place_holder(),
            firstuse: UITimestamp::place_holder(),
            heritage_config: UIHeritageConfig::place_holder(),
            associated_balance: None,
        }
    }
}

impl LoadedSuccessConversionMarker
    for TypeCouple<HeritageConfigWithInfo, UIHeritageConfigurationsHistoryItem>
{
}
impl FromRef<HeritageConfigWithInfo> for UIHeritageConfigurationsHistoryItem {
    fn from_ref(value: &HeritageConfigWithInfo) -> Self {
        let HeritageConfigWithInfo {
            ref heritage_config,
            expiration_ts,
            expiration_status,
            ref balance,
            firstuse_ts,
            ..
        } = *value;

        let expiration = expiration_ts
            .map(|ts| UITimestamp::new_date_only(ts))
            .unwrap_or(UITimestamp::never());

        let expirationbadge = UIExpirationBadge::from((
            expiration_status,
            match balance {
                Some(Ok(b)) if *b != Amount::ZERO => true,
                _ => false,
            },
        ));
        let associated_balance = balance.lrmap(|balance| UIBtcAmount::from(*balance));

        let firstuse = firstuse_ts
            .map(UITimestamp::new_date_only)
            .unwrap_or(UITimestamp::never());

        Self {
            expirationbadge,
            expiration,
            firstuse,
            heritage_config: UIHeritageConfig::from_ref(heritage_config.as_ref()),
            associated_balance,
        }
    }
}
