use crate::prelude::*;

pub mod create_form;
mod v1;

use v1::UIHeritageConfigV1;

use btc_heritage_wallet::btc_heritage::{HeritageConfig, HeritageConfigVersion};

use crate::{
    components::badge::{ExternalDependencyStatus, UIBadge, UIBadgeStyle, UIHeirBadges},
    utils::CCStr,
};

#[derive(Debug, Clone, PartialEq)]
struct UIKnownHeir {
    name: CCStr,
    email: Option<CCStr>,
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
            name: CCStr::place_holder(),
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
            .lmap(|service_heir| {
                service_heir
                    .lmap(|service_heir| CCStr::from(service_heir.main_contact.email.as_str()))
            })
            .flatten();

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
        rsx! {
            LoadedComponent {
                input: m.map(
                    match self {
                        UIHeritageConfig::V1(uiheritage_config_v1) => uiheritage_config_v1,
                    },
                ),
            }
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

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct UIExpirationBadge(UIBadge);
impl UIExpirationBadge {
    pub fn tooltip(self) -> &'static str {
        self.0.tooltip
    }
}
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
            ExpirationStatus::Current => UIBadgeStyle::Custom("badge-success"),
            ExpirationStatus::Outdated | ExpirationStatus::ExpireSoon if with_balance => {
                UIBadgeStyle::Custom("badge-warning")
            }
            ExpirationStatus::Expired if with_balance => UIBadgeStyle::Custom("badge-error"),
            _ => UIBadgeStyle::Custom("badge-soft"),
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
            badge_style: color_class,
            tooltip,
            status: ExternalDependencyStatus::None,
        })
    }
}
