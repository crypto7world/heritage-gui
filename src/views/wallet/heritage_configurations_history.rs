use dioxus::prelude::*;

use std::{collections::HashMap, ops::Deref};

use btc_heritage_wallet::{
    bitcoin::SignedAmount,
    btc_heritage::{
        heritage_config::HeritageExplorerTrait, utils::timestamp_now, HeirConfig, HeritageConfig,
        HeritageConfigVersion,
    },
    heritage_service_api_client::HeritageUtxo,
};

use crate::{
    components::{
        misc::{Badge, Date, DisplayTimestamp, DisplayTimestampStyle, HeirBadgeType},
        wallet::BtcAmount,
    },
    helper_hooks::CompositeHeir,
    utils::{LoadedElement, PlaceHolder, RcType},
};

#[component]
pub(super) fn HeritageConfigurationsHistory() -> Element {
    log::debug!("HeritageConfigurationsHistory Rendered");

    let wallet_heritage_configs = use_context::<Resource<RcType<[RcType<HeritageConfig>]>>>();
    let wallet_utxos = use_context::<Resource<RcType<[HeritageUtxo]>>>();

    let balance_by_heritage_config = use_memo(move || {
        log::debug!("use_memo_utxo_by_heritage_config - start compute");
        let mut balance_by_heritage_config = HashMap::new();

        if let Some(utxos) = wallet_utxos.cloned() {
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
        }

        log::debug!("use_memo_utxo_by_heritage_config - finish compute");
        balance_by_heritage_config
    });

    let heritage_configurations_history_items = use_memo(move || {
        log::debug!("use_memo_heritage_configurations_history_items - start compute");
        let balance_by_heritage_config = &*balance_by_heritage_config.read();
        let heritage_configurations_history_items =
            if let Some(wallet_heritage_configs) = wallet_heritage_configs.cloned() {
                wallet_heritage_configs
                    .iter()
                    .enumerate()
                    .map(|(idx, hc)| {
                        let associated_balance = match balance_by_heritage_config.get(hc) {
                            Some(amount) => LoadedElement::Loaded(*amount),
                            None => LoadedElement::Loading,
                        };
                        HeritageConfigurationsHistoryItemProps {
                            is_current: idx == 0,
                            heritage_configuration: Some(hc.clone()),
                            associated_balance,
                        }
                    })
                    .collect()
            } else {
                vec![HeritageConfigurationsHistoryItemProps {
                    is_current: true,
                    heritage_configuration: None,
                    associated_balance: LoadedElement::Loading,
                }]
            };
        log::debug!("use_memo_heritage_configurations_history_items - finish compute");
        heritage_configurations_history_items
    });

    use_drop(|| log::debug!("HeritageConfigurationsHistory Dropped"));
    rsx! {
        div { class: "overflow-x-auto rounded-box border border-base-content/5 bg-base-100 m-4",
            h2 { class: "text-h2 font-bold p-4", "Heritage Configurations History" }
            for heritage_configurations_history_item in heritage_configurations_history_items() {
                HeritageConfigurationsHistoryItem { ..heritage_configurations_history_item }
            }
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

struct HeritageConfigurationExpirationParams {
    with_balance: bool,
    is_current: bool,
    expiration_ts: u64,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct HeritageConfigurationExpirationStatus {
    with_balance: bool,
    status: ExpirationStatus,
}
impl PlaceHolder for HeritageConfigurationExpirationStatus {
    fn place_holder() -> Self {
        Self {
            with_balance: false,
            status: ExpirationStatus::Current,
        }
    }
}
impl From<HeritageConfigurationExpirationParams> for HeritageConfigurationExpirationStatus {
    fn from(
        HeritageConfigurationExpirationParams {
            with_balance,
            is_current,
            expiration_ts,
        }: HeritageConfigurationExpirationParams,
    ) -> Self {
        let now = timestamp_now();
        let status = if expiration_ts < now {
            ExpirationStatus::Expired
        } else if expiration_ts < now + ExpirationStatus::SOON {
            ExpirationStatus::ExpireSoon
        } else if is_current {
            ExpirationStatus::Current
        } else {
            ExpirationStatus::Outdated
        };
        Self {
            with_balance,
            status,
        }
    }
}

#[component]
fn HeritageConfigurationsHistoryItem(
    is_current: bool,
    heritage_configuration: Option<RcType<HeritageConfig>>,
    associated_balance: LoadedElement<SignedAmount>,
) -> Element {
    log::debug!("HeritageConfigurationsHistoryItem Rendered");

    let expiration_ts = heritage_configuration
        .as_ref()
        .map(|heritage_configuration| {
            heritage_configuration
                .iter_heir_configs()
                .take(1)
                .map(|hc| {
                    heritage_configuration
                        .get_heritage_explorer(hc)
                        .expect("cannot be None as we are iterating heir_configs")
                        .get_spend_conditions()
                        .get_spendable_timestamp()
                        .expect("always present for heirs")
                })
                .next()
        });

    let heritage_configuration_expiration_status = match expiration_ts {
        Some(Some(expiration_ts)) => LoadedElement::Loaded(
            HeritageConfigurationExpirationStatus::from(HeritageConfigurationExpirationParams {
                with_balance: match associated_balance {
                    LoadedElement::Loaded(amount) => amount.is_positive(),
                    LoadedElement::Loading => false,
                },
                is_current,
                expiration_ts,
            }),
        ),
        _ => LoadedElement::Loading,
    };

    let expiration_ts = match expiration_ts {
        Some(Some(expiration_ts)) => LoadedElement::Loaded(DisplayTimestamp::Ts(expiration_ts)),
        Some(None) => LoadedElement::Loaded(DisplayTimestamp::Never),
        None => LoadedElement::Loading,
    };

    use_drop(|| log::debug!("HeritageConfigurationsHistoryItem Dropped"));
    rsx! {
        div { class: "m-4",
            div { class: "collapse collapse-plus bg-base-100 border border-base-300",
                input { r#type: "checkbox", tabindex: "-1" }
                div { class: "collapse-title",
                    div { class: "flex flex-row gap-8 items-center",
                        ExpirationStatusBadge { heritage_configuration_expiration_status }
                        div { class: "flex flex-col",
                            div { class: "text-sm font-light", "Expiration" }
                            div { class: "font-bold",
                                Date {
                                    timestamp: expiration_ts,
                                    display_style: DisplayTimestampStyle::DateOnly,
                                }
                            }
                        }
                        div { class: "flex flex-col",
                            div { class: "text-sm font-light", "Associated Balance" }
                            div { class: "font-bold",
                                BtcAmount { amount: associated_balance.map(|inner| inner.into()) }
                            }
                        }
                    }
                }
                if let Some(heritage_configuration) = heritage_configuration {
                    match heritage_configuration.version() {
                        HeritageConfigVersion::V1 => {
                            rsx! {
                                HeritageConfigurationsHistoryItemContentV1 { heritage_configuration }
                            }
                        }
                    }
                }
            }
        }
    }
}

#[component]
pub fn ExpirationStatusBadge(
    heritage_configuration_expiration_status: LoadedElement<HeritageConfigurationExpirationStatus>,
) -> Element {
    let (is_place_holder, heritage_configuration_expiration_status) =
        heritage_configuration_expiration_status.extract();

    let HeritageConfigurationExpirationStatus {
        with_balance,
        status,
    } = heritage_configuration_expiration_status;

    let content = match status {
        ExpirationStatus::Current => "Current",
        ExpirationStatus::Outdated => "Outdated",
        ExpirationStatus::ExpireSoon => "Expiring",
        ExpirationStatus::Expired => "Expired",
    };
    let color = match status {
        ExpirationStatus::Current => "badge-success",
        ExpirationStatus::Outdated | ExpirationStatus::ExpireSoon if with_balance => {
            "badge-warning"
        }
        ExpirationStatus::Expired if with_balance => "badge-error",
        _ => "badge-soft badge-neutral",
    };
    rsx! {
        div {
            class: "badge badge-lg text-nowrap font-black",
            class: if is_place_holder { "skeleton" } else { "{color}" },
            {content}
        }
    }
}

#[component]
fn HeritageConfigurationsHistoryItemContentV1(
    heritage_configuration: RcType<HeritageConfig>,
) -> Element {
    log::debug!("HeritageConfigurationsHistoryItemContentV1 Rendered");

    let heritage_v1 = heritage_configuration
        .heritage_config_v1()
        .expect("just verified it is v1");

    let last_position = heritage_v1.iter_heritages().size_hint().0;
    log::debug!("last_position={last_position}");

    let heritages = heritage_v1.iter_heritages().enumerate().map(|(i, h)| {
        let heir_config = RcType::from(h.heir_config.clone());
        let locked_for_days = h.time_lock.as_u16();
        let maturity_ts = heritage_v1.reference_timestamp.as_u64() + h.time_lock.as_seconds();
        (i + 1, heir_config, locked_for_days, maturity_ts)
    });

    use_drop(|| log::debug!("HeritageConfigurationsHistoryItemContentV1 Dropped"));
    rsx! {
        div { class: "collapse-content",
            div { class: "text-lg font-bold",
                "{last_position} heir"
                if last_position > 1 {
                    "s"
                }
            }
            ul { class: "timeline timeline-vertical timeline-compact",
                for (position , heir_config , locked_for_days , maturity_ts) in heritages {
                    HeritageConfigurationsHistoryItemContentV1Heir {
                        position,
                        heir_config,
                        locked_for_days,
                        maturity_ts,
                        is_last: position == last_position,
                    }
                }
            }
        }
    }
}

#[component]
fn HeritageConfigurationsHistoryItemContentV1Heir(
    position: usize,
    heir_config: RcType<HeirConfig>,
    locked_for_days: u16,
    maturity_ts: u64,
    is_last: bool,
) -> Element {
    log::debug!("HeritageConfigurationsHistoryItemContentV1Heir Rendered");

    let heir_config_type = match heir_config.deref() {
        HeirConfig::SingleHeirPubkey(_) => "Public Key",
        HeirConfig::HeirXPubkey(_) => "Extended Public Key",
    };
    let heir_config_fg = heir_config.fingerprint();

    let heirs = use_context::<Memo<HashMap<RcType<HeirConfig>, CompositeHeir>>>();
    let heir = heirs.read().get(&heir_config).cloned();

    use_drop(|| log::debug!("HeritageConfigurationsHistoryItemContentV1Heir Dropped"));
    rsx! {
        li { class: "w-fit",
            hr { class: "bg-base-content" }
            div { class: "timeline-middle",
                div { class: "bg-primary rounded-full aspect-square content-center",
                    span { class: "m-1 font-bold", "#{position}" }
                }
            }
            div { class: "timeline-end timeline-box flex flex-col",
                if let Some(heir) = heir {
                    KnownHeir { heir }
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
                            Date {
                                timestamp: DisplayTimestamp::from(maturity_ts).into(),
                                display_style: DisplayTimestampStyle::DateOnly,
                            }
                        }
                    }
                }
            }
            if !is_last {
                hr { class: "bg-base-content" }
            }
        }
    }
}

#[component]
fn KnownHeir(heir: CompositeHeir) -> Element {
    let CompositeHeir {
        name,
        db_heir,
        service_heir,
        ..
    } = heir;

    let badges = db_heir
        .iter()
        .map(|_| HeirBadgeType::Database)
        .chain(service_heir.iter().map(|_| HeirBadgeType::Service));
    let email = service_heir
        .as_ref()
        .map(|service_heir| &service_heir.main_contact.email);

    rsx! {
        div { class: "flex flex-row gap-4",
            div {
                span { class: "text-xl font-bold mr-2", {name} }
                if let Some(email) = email {
                    span { class: "text-base font-semibold", "({email})" }
                }
            }
            div { class: "grow" }
            for badge in badges {
                Badge { badge }
            }
        }
    }
}
