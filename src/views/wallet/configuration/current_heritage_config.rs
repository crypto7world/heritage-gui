use crate::prelude::*;

use btc_heritage_wallet::btc_heritage::{HeritageConfig, HeritageConfigVersion};

use crate::{
    components::{
        heritage_configuration::{create_form::NewHeritageConfigForm, UIHeritageConfig},
        modal::ConfigModal,
        svg::{DrawSvg, Edit},
        timestamp::UITimestamp,
    },
    utils::CheapClone,
};

#[component]
pub(super) fn CurrentHeritageConfig() -> Element {
    let heritage_configs_with_info = use_context::<FMemo<CheapClone<[HeritageConfigWithInfo]>>>();

    let current_heritage_config = use_memo(move || {
        log::debug!("use_memo_current_heritage_config - start compute");
        let current_heritage_config =
            heritage_configs_with_info.lrmap(|heritage_configs_with_info| {
                heritage_configs_with_info
                    .first()
                    .map(|hcwi| (hcwi.heritage_config.clone(), hcwi.firstuse_ts))
            });

        log::debug!("use_memo_current_heritage_config - finish compute");
        current_heritage_config
    });
    let current_heritage_config_for_display = use_memo(move || {
        log::debug!("use_memo_current_heritage_config_for_display - start compute");
        let current_heritage_config_for_display =
            current_heritage_config.lrmap(|current_heritage_config| {
                current_heritage_config
                    .clone()
                    .map(Show)
                    .unwrap_or_default()
            });

        log::debug!("use_memo_current_heritage_config_for_display - finish compute");
        current_heritage_config_for_display
    });
    let existing_heritage_config = use_memo(move || {
        log::debug!("use_memo_existing_heritage_config - start compute");
        let existing_heritage_config = match current_heritage_config.cloned() {
            Some(Ok(Some((h, _)))) => Some(h),
            _ => None,
        };
        log::debug!("use_memo_existing_heritage_config - finish compute");
        existing_heritage_config
    });

    let action_text = use_memo(move || {
        log::debug!("use_memo_action_text - start compute");
        let action_text = if existing_heritage_config.read().is_some() {
            "Update"
        } else {
            "Create"
        };

        log::debug!("use_memo_action_text - finish compute");
        action_text
    });

    let mut new_heritage_config_modal = use_signal(|| false);

    rsx! {
        div { class: "rounded-box border border-base-content/5 shadow-md my-4",
            div { class: "p-4 w-fit",
                h2 { class: "text-2xl font-bold mb-4", "Heritage Configuration" }

                div { class: "text-sm font-light mb-6",
                    "The Heritage Configuration defines the inheritance rules for your wallet. It specifies how long funds must remain untouched before your heirs can access them, and which heirs are authorized to inherit. "
                    "This configuration works in conjunction with your Account Extended Public Keys - each time you create or update a Heritage Configuration, the wallet generates new Bitcoin addresses using your Account XPubs. "
                    "The reference date shown below determines when the inheritance timelock begins counting for new transactions."
                }

                LoadedComponent::<Display<UICurrentHeritageConfigV1>> { input: current_heritage_config_for_display.into() }


                // Action buttons
                div { class: "flex gap-4 mt-4",
                    MaybeHighlight {
                        step: OnboardingStep::ClickCreateHeritageConfigurationButton1,
                        context_filter: consume_onboarding_context(),
                        button {
                            class: "btn btn-primary",
                            onclick: move |_| *new_heritage_config_modal.write() = true,
                            DrawSvg::<Edit> {}
                            {action_text}
                        }
                    }
                }
                ConfigModal {
                    is_open: new_heritage_config_modal,
                    title: "New Heritage Configuration",
                    NewHeritageConfigForm {
                        existing_heritage_config,
                        new_heritage_config_modal,
                    }
                }
            

            }
        }
    }
}

#[derive(Clone, PartialEq)]
struct UICurrentHeritageConfigV1 {
    firstuse_date: UITimestamp,
    ref_date: UITimestamp,
    heritage_config: UIHeritageConfig,
}
impl LoadedElement for UICurrentHeritageConfigV1 {
    type Loader = TransparentLoader;
    #[inline(always)]
    fn element<M: LoadedComponentInputMapper>(self, m: M) -> Element {
        rsx! {
            div { class: "flex flex-row gap-4",
                div { class: "text-nowrap text-xl",
                    "Reference date: "
                    span { class: "font-bold",
                        LoadedComponent { input: m.map(self.ref_date) }
                    }
                }
                div { class: "text-nowrap text-xl",
                    "Used since: "
                    span { class: "font-bold",
                        LoadedComponent { input: m.map(self.firstuse_date) }
                    }
                }
            }
            LoadedComponent { input: m.map(self.heritage_config) }
        }
    }
    fn place_holder() -> Self {
        Self {
            firstuse_date: UITimestamp::place_holder(),
            ref_date: UITimestamp::place_holder(),
            heritage_config: UIHeritageConfig::place_holder(),
        }
    }
}
impl FromRef<(CheapClone<HeritageConfig>, Option<u64>)> for UICurrentHeritageConfigV1 {
    fn from_ref(
        (heritage_configuration, firstuse_ts): &(CheapClone<HeritageConfig>, Option<u64>),
    ) -> Self {
        match heritage_configuration.version() {
            HeritageConfigVersion::V1 => Self {
                firstuse_date: firstuse_ts
                    .map(UITimestamp::new_date_only)
                    .unwrap_or(UITimestamp::never()),
                ref_date: UITimestamp::new_date_only(
                    heritage_configuration
                        .heritage_config_v1()
                        .unwrap()
                        .reference_timestamp
                        .as_u64(),
                ),
                heritage_config: UIHeritageConfig::from_ref(heritage_configuration.as_ref()),
            },
        }
    }
}
