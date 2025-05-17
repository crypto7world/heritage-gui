use dioxus::prelude::*;

use std::{collections::HashMap, ops::Deref};

use btc_heritage_wallet::{
    bitcoin::SignedAmount,
    btc_heritage::{
        heritage_config::HeritageExplorerTrait, utils::timestamp_now, HeirConfig, HeritageConfig,
        HeritageConfigVersion,
    },
    heritage_service_api_client::{Heir, HeritageUtxo},
};

use crate::{
    components::{
        badge::{UIBadge, UIHeirBadges},
        balance::UIBtcAmount,
        timestamp::UITimestamp,
    },
    helper_hooks::CompositeHeir,
    loaded::prelude::*,
    utils::{ArcStr, ArcType},
};

#[derive(Debug, Clone, PartialEq)]
struct UIKnownHeir {
    name: ArcStr,
    email: Option<ArcStr>,
    badges: UIHeirBadges,
}
impl LoadedElement for UIKnownHeir {
    type Loader = TransparentLoader;
    #[inline(always)]
    fn element<M: LoadedComponentInputMapper>(self, m: M) -> Element {
        rsx! {
            div { class: "flex flex-row gap-4",
                div {
                    span { class: "text-xl font-bold mr-2",
                        LoadedComponent { input: m.map(self.name) }
                    }
                    if let Some(email) = self.email {
                        span { class: "text-base font-semibold", "({email})" }
                    }
                }
                div { class: "grow" }
                LoadedComponent { input: m.map(self.badges) }
            }
        }
    }
    fn place_holder() -> Self {
        Self {
            name: ArcStr::place_holder(),
            email: None,
            badges: UIHeirBadges::place_holder(),
        }
    }
}
impl FromRef<CompositeHeir> for UIKnownHeir {
    fn from_ref(composite_heir: &CompositeHeir) -> Self {
        let badges = UIHeirBadges::from_ref(composite_heir);
        let CompositeHeir {
            name, service_heir, ..
        } = composite_heir;

        let email = service_heir
            .as_ref()
            .map(|service_heir| ArcStr::from(service_heir.main_contact.email.to_string()));

        Self {
            name: name.clone(),
            email,
            badges,
        }
    }
}

#[derive(Clone, PartialEq)]
enum UIHeritageConfig {
    V1(UIHeritageConfigV1),
}
impl LoadedElement for UIHeritageConfig {
    type Loader = TransparentLoader;
    #[inline(always)]
    fn element<M: LoadedComponentInputMapper>(self, m: M) -> Element {
        match self {
            UIHeritageConfig::V1(uiheritage_config_v1) => uiheritage_config_v1.element(m),
        }
    }
    fn place_holder() -> Self {
        Self::V1(UIHeritageConfigV1::place_holder())
    }
}
impl FromRef<HeritageConfig> for UIHeritageConfig {
    fn from_ref(heritage_configuration: &HeritageConfig) -> Self {
        match heritage_configuration.version() {
            HeritageConfigVersion::V1 => {
                Self::V1(UIHeritageConfigV1::from_ref(heritage_configuration))
            }
        }
    }
}

#[derive(Clone, PartialEq)]
struct UIHeritageConfigV1(Vec<HeritageConfigurationsHistoryItemContentV1HeirProps>);
impl LoadedElement for UIHeritageConfigV1 {
    type Loader = SkeletonLoader;
    #[inline(always)]
    fn element<M: LoadedComponentInputMapper>(self, _m: M) -> Element {
        let heir_count = self.0.len();
        rsx! {
            div { class: "collapse-content",
                div { class: "text-lg font-bold",
                    "{heir_count} heir"
                    if heir_count > 1 {
                        "s"
                    }
                }
                ul { class: "timeline timeline-vertical timeline-compact",
                    for props in self.0 {
                        HeritageConfigurationsHistoryItemContentV1Heir { ..props }
                    }
                }
            }
        }
    }
    fn place_holder() -> Self {
        Self(vec![])
    }
}
impl FromRef<HeritageConfig> for UIHeritageConfigV1 {
    fn from_ref(heritage_configuration: &HeritageConfig) -> Self {
        let heritage_v1 = heritage_configuration
            .heritage_config_v1()
            .expect("verified it is v1");

        Self(
            heritage_v1
                .iter_heritages()
                .enumerate()
                .map(|(i, h)| {
                    let heir_config = ArcType::from(h.heir_config.clone());
                    let locked_for_days = h.time_lock.as_u16();
                    let maturity_ts =
                        heritage_v1.reference_timestamp.as_u64() + h.time_lock.as_seconds();
                    HeritageConfigurationsHistoryItemContentV1HeirProps {
                        position: i + 1,
                        heir_config,
                        locked_for_days,
                        maturity_ts,
                    }
                })
                .collect(),
        )
    }
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
                    LoadedComponent::<UIHeritageConfig> { input: m.map(self.heritage_config) }
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

#[component]
fn HeritageConfigurationsHistoryItemContentV1Heir(
    position: usize,
    heir_config: ArcType<HeirConfig>,
    locked_for_days: u16,
    maturity_ts: u64,
) -> Element {
    log::debug!("HeritageConfigurationsHistoryItemContentV1Heir Rendered");

    let heir_config_type = match heir_config.deref() {
        HeirConfig::SingleHeirPubkey(_) => "Public Key",
        HeirConfig::HeirXPubkey(_) => "Extended Public Key",
    };
    let heir_config_fg = heir_config.fingerprint();

    let service_heirs = use_context::<Resource<Vec<ArcType<Heir>>>>();
    let service_loading = service_heirs.read().is_none();

    let heirs = use_context::<Memo<HashMap<ArcType<HeirConfig>, CompositeHeir>>>();
    let heir = use_memo(move || heirs.read().get(&heir_config).cloned().map(|v| (v,)));

    let maturity_date = UITimestamp::new_date_only(maturity_ts);

    use_drop(|| log::debug!("HeritageConfigurationsHistoryItemContentV1Heir Dropped"));
    rsx! {
        li { class: "w-fit group",
            hr { class: "bg-base-content" }
            div { class: "timeline-middle",
                div { class: "bg-primary rounded-full aspect-square content-center",
                    span { class: "m-1 font-bold", "#{position}" }
                }
            }
            div { class: "timeline-end timeline-box flex flex-col",
                if service_loading {
                    LoadedComponent::<UIKnownHeir> { input: None::<UIKnownHeir>.into() }
                } else {
                    LoadedComponent::<UIKnownHeir> { input: heir.into() }
                }
                div { class: "flex flex-row gap-8",
                    div { class: "flex flex-col",
                        div { class: "font-light", "Key Type" }
                        div { class: "text-lg font-bold", "{heir_config_type}" }
                    }
                    div { class: "flex flex-col",
                        div { class: "font-light", "Key Fingerprint" }
                        div { class: "text-lg font-bold", "{heir_config_fg}" }
                    }
                    div { class: "flex flex-col",
                        div { class: "font-light", "Locked for" }
                        div { class: "text-lg font-bold", "{locked_for_days} days" }
                    }
                    div { class: "flex flex-col",
                        div { class: "font-light", "Maturity Date" }
                        div { class: "text-lg font-bold",
                            AlwaysLoadedComponent { input: maturity_date }
                        }
                    }
                }
            }
            hr { class: "bg-base-content group-last:hidden" }
        }
    }
}
