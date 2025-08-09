use crate::prelude::*;

use btc_heritage_wallet::{
    btc_heritage::{HeirConfig, HeritageConfig},
    heritage_service_api_client::Heir,
};

use crate::{
    components::timestamp::UITimestamp,
    utils::{heir_config_type_to_string, CheapClone},
};

#[derive(Clone, PartialEq)]
pub struct UIHeritageConfigV1(Vec<HeritageConfigV1HeirProps>);
impl LoadedElement for UIHeritageConfigV1 {
    type Loader = SkeletonLoader;
    #[inline(always)]
    fn element<M: LoadedComponentInputMapper>(self, _m: M) -> Element {
        let heir_count = self.0.len();
        rsx! {
            div { class: "text-lg font-bold",
                "{heir_count} heir"
                if heir_count > 1 {
                    "s"
                }
            }
            ul { class: "timeline timeline-vertical timeline-compact",
                for props in self.0 {
                    HeritageConfigV1Heir { ..props }
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
                    let heir_config = CheapClone::from(h.heir_config.clone());
                    let locked_for_days = h.time_lock.as_u16();
                    let maturity_ts =
                        heritage_v1.reference_timestamp.as_u64() + h.time_lock.as_seconds();
                    HeritageConfigV1HeirProps {
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

#[component]
fn HeritageConfigV1Heir(
    position: usize,
    heir_config: CheapClone<HeirConfig>,
    locked_for_days: u16,
    maturity_ts: u64,
) -> Element {
    log::debug!("HeritageConfigurationsHistoryItemContentV1Heir Rendered");

    let heir_config_type = heir_config_type_to_string(&heir_config);
    let heir_config_fg = heir_config.fingerprint();

    let service_heirs = use_context::<FResource<Vec<CheapClone<Heir>>>>();

    let heirs = use_context::<Memo<Vec<CompositeHeir>>>();
    let heir = use_memo(move || {
        log::debug!("use_memo_heir - start compute");
        let heir = service_heirs.lmap(|_| {
            heirs
                .read()
                .iter()
                .find(|h| h.heir_config == heir_config)
                .cloned()
                .map(Show)
                .unwrap_or_default()
        });
        log::debug!("use_memo_heir - finish compute");
        heir
    });

    let maturity_date = UITimestamp::new_date_only(maturity_ts);

    use_drop(|| log::debug!("HeritageConfigurationsHistoryItemContentV1Heir Dropped"));
    rsx! {
        li { class: "w-fit group",
            hr { class: "bg-base-content" }
            div { class: "timeline-middle",
                div { class: "bg-primary rounded-full aspect-square content-center",
                    span { class: "m-1 font-bold text-(--color-primary-content)", "#{position}" }
                }
            }
            div { class: "timeline-end timeline-box flex flex-col",
                LoadedComponent::<Display<super::UIKnownHeir>> { input: heir.into() }
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
