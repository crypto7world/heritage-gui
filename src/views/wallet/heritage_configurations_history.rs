use dioxus::prelude::*;

use std::collections::HashMap;

use btc_heritage_wallet::{
    bitcoin::SignedAmount,
    btc_heritage::{heritage_config::HeritageExplorerTrait, utils::timestamp_now, HeritageConfig},
    heritage_service_api_client::HeritageUtxo,
};

use crate::{
    components::{
        badge::UIBadge, balance::UIBtcAmount, heritage_configuration::UIHeritageConfig,
        timestamp::UITimestamp,
    },
    loaded::prelude::*,
    utils::ArcType,
};

#[component]
pub(super) fn HeritageConfigurationsHistory() -> Element {
    log::debug!("HeritageConfigurationsHistory Rendered");

    let wallet_heritage_configs = use_context::<Resource<ArcType<[ArcType<HeritageConfig>]>>>();

    let wallet_utxos = use_context::<Resource<ArcType<[HeritageUtxo]>>>();
    let balance_by_heritage_config = use_memo(move || {
        log::debug!("use_memo_utxo_by_heritage_config - start compute");

        let balance_by_heritage_config = if let Some(utxos) = wallet_utxos.cloned() {
            let mut balance_by_heritage_config = HashMap::new();
            for utxo in utxos.iter() {
                let heritage_config = utxo.heritage_config.clone();
                let utxo_amount = utxo
                    .amount
                    .to_signed()
                    .expect("UTXO amount cannot be bigger than MAX_MONEY");
                balance_by_heritage_config
                    .entry(heritage_config)
                    .and_modify(|balance| *balance += utxo_amount)
                    .or_insert(utxo_amount);
            }
            Some(balance_by_heritage_config)
        } else {
            None
        };

        log::debug!("use_memo_utxo_by_heritage_config - finish compute");
        balance_by_heritage_config
    });

    let heritage_configurations_history_items = use_memo(move || {
        log::debug!("use_memo_heritage_configurations_history_items - start compute");
        let balance_by_heritage_config = &*balance_by_heritage_config.read();
        let heritage_configurations_history_items =
            wallet_heritage_configs
                .cloned()
                .map(|wallet_heritage_configs| {
                    wallet_heritage_configs
                        .iter()
                        .enumerate()
                        .map(|(idx, heritage_config)| {
                            let associated_balance = balance_by_heritage_config.as_ref().map(
                                |balance_by_heritage_config| {
                                    balance_by_heritage_config.get(heritage_config).cloned()
                                },
                            );
                            (idx, heritage_config.clone(), associated_balance)
                        })
                        .collect::<Vec<_>>()
                });
        log::debug!("use_memo_heritage_configurations_history_items - finish compute");
        heritage_configurations_history_items
    });

    use_drop(|| log::debug!("HeritageConfigurationsHistory Dropped"));
    rsx! {
        div { class: "overflow-x-auto rounded-box border border-base-content/5 bg-base-100 m-4",
            h2 { class: "text-h2 font-bold p-4", "Heritage Configurations History" }
            LoadedComponent::<Vec<UIHeritageConfigurationsHistoryItem>> { input: heritage_configurations_history_items.into() }
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ExpirationStatus {
    Current,
    Outdated,
    ExpireSoon,
    Expired,
}
impl ExpirationStatus {
    // SOON = 1 month
    // 30 days x 24 hours x 60 mins x 60 secs
    const SOON: u64 = 30 * 24 * 60 * 60;
}

#[derive(Debug, Clone, PartialEq)]
struct UIExpirationBadge(UIBadge);
impl LoadedElement for UIExpirationBadge {
    type Loader = SkeletonLoader;
    fn element<M: LoadedComponentInputMapper>(self, m: M) -> Element {
        self.0.element(m)
    }

    fn place_holder() -> Self {
        Self(UIBadge::place_holder())
    }
}
impl From<(ExpirationStatus, bool)> for UIExpirationBadge {
    fn from((expiration_status, with_balance): (ExpirationStatus, bool)) -> Self {
        let text = match expiration_status {
            ExpirationStatus::Current => "Current",
            ExpirationStatus::Outdated => "Outdated",
            ExpirationStatus::ExpireSoon => "Expiring",
            ExpirationStatus::Expired => "Expired",
        };
        let color_class = match expiration_status {
            ExpirationStatus::Current => "badge-success",
            ExpirationStatus::Outdated | ExpirationStatus::ExpireSoon if with_balance => {
                "badge-warning"
            }
            ExpirationStatus::Expired if with_balance => "badge-error",
            _ => "badge-soft badge-neutral",
        };
        let tooltip = match expiration_status {
            ExpirationStatus::Current => "This is the current Heritage Configuration",
            ExpirationStatus::ExpireSoon if with_balance => {
                "The Heritage Configuration will expire soon, move your bitcoins"
            }
            ExpirationStatus::ExpireSoon => "The Heritage Configuration will expire soon",
            ExpirationStatus::Outdated if with_balance => {
                "This Heritage Configuration should not be used, move your bitcoins to the recent one"
            }
            ExpirationStatus::Outdated => "This Heritage Configuration is obsolete",
            ExpirationStatus::Expired if with_balance =>  "This Heritage Configuration is expired, move your bitcoins",
            ExpirationStatus::Expired => "This Heritage Configuration is expired",
        };
        Self(UIBadge {
            text,
            color_class,
            tooltip,
        })
    }
}

#[derive(Clone, PartialEq)]
struct UIHeritageConfigurationsHistoryItem {
    expirationbadge: Option<UIExpirationBadge>,
    expiration: UITimestamp,
    heritage_config: UIHeritageConfig,
    associated_balance: Option<UIBtcAmount>,
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
                        div { class: "flex flex-row gap-8 items-center",
                            LoadedComponent::<UIExpirationBadge> { input: self.expirationbadge.into() }
                            div { class: "flex flex-col",
                                div { class: "text-sm font-light", "Expiration" }
                                div { class: "font-bold",
                                    LoadedComponent::<UITimestamp> { input: m.map(self.expiration) }
                                }
                            }
                            div { class: "flex flex-col",
                                div { class: "text-sm font-light", "Associated Balance" }
                                div { class: "font-bold",
                                    LoadedComponent::<UIBtcAmount> { input: self.associated_balance.into() }
                                }
                            }
                        }
                    }
                    div { class: "collapse-content",
                        LoadedComponent::<UIHeritageConfig> { input: m.map(self.heritage_config) }
                    }
                }
            }
        }
    }
    fn place_holder() -> Self {
        Self {
            expirationbadge: Some(UIExpirationBadge::place_holder()),
            expiration: UITimestamp::place_holder(),
            heritage_config: UIHeritageConfig::place_holder(),
            associated_balance: Some(UIBtcAmount::place_holder()),
        }
    }
}
impl FromRef<(usize, ArcType<HeritageConfig>, Option<Option<SignedAmount>>)>
    for UIHeritageConfigurationsHistoryItem
{
    fn from_ref(
        (idx, heritage_config, associated_balance): &(
            usize,
            ArcType<HeritageConfig>,
            Option<Option<SignedAmount>>,
        ),
    ) -> Self {
        let idx = *idx;
        let associated_balance = *associated_balance;

        let expiration_ts = heritage_config
            .iter_heir_configs()
            .take(1)
            .map(|hc| {
                heritage_config
                    .get_heritage_explorer(hc)
                    .expect("cannot be None as we are iterating heir_configs")
                    .get_spend_conditions()
                    .get_spendable_timestamp()
                    .expect("always present for heirs")
            })
            .next();

        let expiration = expiration_ts
            .map(|ts| UITimestamp::new_date_only(ts))
            .unwrap_or(UITimestamp::never());

        let now = timestamp_now();
        let is_newest = idx == 0;
        let status = if expiration_ts.is_some_and(|ts| ts < now) {
            ExpirationStatus::Expired
        } else if expiration_ts.is_some_and(|ts| ts < now + ExpirationStatus::SOON) {
            ExpirationStatus::ExpireSoon
        } else if is_newest {
            ExpirationStatus::Current
        } else {
            ExpirationStatus::Outdated
        };
        let expirationbadge = associated_balance.map(|associated_balance| {
            let with_balance = associated_balance.is_some_and(|b| b != SignedAmount::ZERO);
            UIExpirationBadge::from((status, with_balance))
        });
        let associated_balance = associated_balance
            .map(|associated_balance| UIBtcAmount::new(associated_balance, false));

        Self {
            expirationbadge,
            expiration,
            heritage_config: UIHeritageConfig::from_ref(heritage_config.as_ref()),
            associated_balance,
        }
    }
}
