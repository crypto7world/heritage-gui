pub mod create_form;
mod v1;

use dioxus::prelude::*;

use v1::UIHeritageConfigV1;

use btc_heritage_wallet::btc_heritage::{HeritageConfig, HeritageConfigVersion};

use crate::{
    components::badge::UIHeirBadges, helper_hooks::CompositeHeir, loaded::prelude::*, utils::ArcStr,
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
pub enum UIHeritageConfig {
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
