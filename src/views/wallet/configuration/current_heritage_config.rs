use btc_heritage_wallet::btc_heritage::{HeritageConfig, HeritageConfigVersion};
use dioxus::prelude::*;

use crate::{
    components::{
        heritage_configuration::{create_form::NewHeritageConfigForm, UIHeritageConfig},
        misc::Modal,
        timestamp::UITimestamp,
    },
    loaded::prelude::*,
    utils::ArcType,
};

#[component]
pub(super) fn CurrentHeritageConfig() -> Element {
    let wallet_heritage_configs = use_context::<Resource<ArcType<[ArcType<HeritageConfig>]>>>();

    let current_heritage_config = use_memo(move || {
        log::debug!("use_memo_utxo_by_heritage_config - start compute");
        let current_heritage_config =
            wallet_heritage_configs
                .cloned()
                .map(|wallet_heritage_configs| {
                    wallet_heritage_configs
                        .first()
                        .cloned()
                        .map(Show)
                        .unwrap_or_default()
                });

        log::debug!("use_memo_utxo_by_heritage_config - finish compute");
        current_heritage_config
    });

    let action_text = use_memo(move || {
        log::debug!("use_memo_action_text - start compute");
        let action_text = if let Some(Show(_)) = current_heritage_config() {
            "Update"
        } else {
            "Create"
        };

        log::debug!("use_memo_action_text - finish compute");
        action_text
    });

    let mut new_heritage_config_modal = use_signal(|| false);

    rsx! {
        div { class: "container mx-auto p-4 w-fit",
            h2 { class: "text-2xl font-bold mb-4", "Current Heritage Configuration" }
            LoadedComponent::<Display<UICurrentHeritageConfigV1>> { input: current_heritage_config.into() }


            // Action buttons
            div { class: "flex gap-4 mt-4 justify-center",
                button {
                    class: "btn btn-primary btn-lg",
                    onclick: move |_| *new_heritage_config_modal.write() = true,
                    svg {
                        class: "size-8 fill-current",
                        xmlns: "http://www.w3.org/2000/svg",
                        view_box: "0 0 24 24",
                        path { d: "M20.71,7.04C20.37,7.38 20.04,7.71 20.03,8.04C20,8.36 20.34,8.69 20.66,9C21.14,9.5 21.61,9.95 21.59,10.44C21.57,10.93 21.06,11.44 20.55,11.94L16.42,16.08L15,14.66L19.25,10.42L18.29,9.46L16.87,10.87L13.12,7.12L16.96,3.29C17.35,2.9 18,2.9 18.37,3.29L20.71,5.63C21.1,6 21.1,6.65 20.71,7.04M3,17.25L12.56,7.68L16.31,11.43L6.75,21H3V17.25Z" }
                    }
                    {action_text}
                }
            }
            Modal { is_open: new_heritage_config_modal, persistent: true,
                div { class: "flex flex-col gap-2 justify-between w-[calc(100vw*0.8)] h-[calc(100vh*0.8)]",

                    div { class: "flex flex-row justify-between",
                        h2 { class: "text-2xl font-bold mb-4", "New Heritage Configuration" }
                        button {
                            class: "btn btn-circle btn-outline btn-primary btn-lg p-2",
                            onclick: move |_| *new_heritage_config_modal.write() = false,
                            svg {
                                class: "size-8 fill-current",
                                xmlns: "http://www.w3.org/2000/svg",
                                view_box: "0 0 24 24",
                                path { d: "M19,6.41L17.59,5L12,10.59L6.41,5L5,6.41L10.59,12L5,17.59L6.41,19L12,13.41L17.59,19L19,17.59L13.41,12L19,6.41Z" }
                            }
                        }
                    }
                    NewHeritageConfigForm {}

                    // Action buttons
                    div { class: "flex gap-4 mt-4 justify-center",
                        button {
                            class: "btn btn-primary",
                            onclick: move |_| *new_heritage_config_modal.write() = true,
                            svg {
                                class: "size-8 fill-current",
                                xmlns: "http://www.w3.org/2000/svg",
                                view_box: "0 0 24 24",
                                path { d: "M20.71,7.04C20.37,7.38 20.04,7.71 20.03,8.04C20,8.36 20.34,8.69 20.66,9C21.14,9.5 21.61,9.95 21.59,10.44C21.57,10.93 21.06,11.44 20.55,11.94L16.42,16.08L15,14.66L19.25,10.42L18.29,9.46L16.87,10.87L13.12,7.12L16.96,3.29C17.35,2.9 18,2.9 18.37,3.29L20.71,5.63C21.1,6 21.1,6.65 20.71,7.04M3,17.25L12.56,7.68L16.31,11.43L6.75,21H3V17.25Z" }
                            }
                            "Create New"
                        }
                        button {
                            class: "btn btn-primary",
                            onclick: move |_| *new_heritage_config_modal.write() = false,
                            svg {
                                class: "size-8 fill-current",
                                xmlns: "http://www.w3.org/2000/svg",
                                view_box: "0 0 24 24",
                                path { d: "M12 2C17.5 2 22 6.5 22 12S17.5 22 12 22 2 17.5 2 12 6.5 2 12 2M12 4C10.1 4 8.4 4.6 7.1 5.7L18.3 16.9C19.3 15.5 20 13.8 20 12C20 7.6 16.4 4 12 4M16.9 18.3L5.7 7.1C4.6 8.4 4 10.1 4 12C4 16.4 7.6 20 12 20C13.9 20 15.6 19.4 16.9 18.3Z" }
                            }

                            "Cancel"
                        }
                    }
                }
            }
        
        }
    }
}

#[derive(Clone, PartialEq)]
struct UICurrentHeritageConfigV1 {
    ref_date: UITimestamp,
    heritage_config: UIHeritageConfig,
}
impl LoadedElement for UICurrentHeritageConfigV1 {
    type Loader = TransparentLoader;
    #[inline(always)]
    fn element<M: LoadedComponentInputMapper>(self, m: M) -> Element {
        rsx! {
            div { class: "text-nowrap text-xl",
                "Reference date: "
                span { class: "font-bold",
                    LoadedComponent { input: m.map(self.ref_date) }
                }
            }
            LoadedComponent { input: m.map(self.heritage_config) }
        }
    }
    fn place_holder() -> Self {
        Self {
            ref_date: UITimestamp::place_holder(),
            heritage_config: UIHeritageConfig::place_holder(),
        }
    }
}
impl FromRef<ArcType<HeritageConfig>> for UICurrentHeritageConfigV1 {
    fn from_ref(heritage_configuration: &ArcType<HeritageConfig>) -> Self {
        match heritage_configuration.version() {
            HeritageConfigVersion::V1 => Self {
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
