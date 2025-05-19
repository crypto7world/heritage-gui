use dioxus::prelude::*;

use btc_heritage_wallet::AnyKeyProvider;

use crate::{
    components::misc::Tooltip, helper_hooks::CompositeHeir, loaded::prelude::*, utils::ArcStr,
};

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct UIBadge {
    pub text: &'static str,
    pub color_class: &'static str,
    pub tooltip: &'static str,
}
impl LoadedElement for UIBadge {
    type Loader = SkeletonLoader;
    #[inline(always)]
    fn element<M: LoadedComponentInputMapper>(self, _m: M) -> Element {
        rsx! {
            Tooltip { tooltip_text: ArcStr::from(self.tooltip),
                div { class: "badge shadow-xl text-nowrap {self.color_class}", {self.text} }
            }
        }
    }

    fn place_holder() -> Self {
        Self {
            text: <&'static str>::place_holder(),
            color_class: <&'static str>::place_holder(),
            tooltip: <&'static str>::place_holder(),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum OnlineWalletType {
    None,
    Service,
    Local,
}
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum KeyProviderType {
    None,
    LocalKey,
    Ledger,
}
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ExternalDependencyStatus {
    Available,
    Unavailable,
}
impl ExternalDependencyStatus {
    fn color_class(self) -> &'static str {
        match self {
            Self::Available => "badge-success",
            Self::Unavailable => "badge-error",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct UIKeyProviderBadge(UIBadge);
impl LoadedElement for UIKeyProviderBadge {
    type Loader = SkeletonLoader;
    #[inline(always)]
    fn element<M: LoadedComponentInputMapper>(self, m: M) -> Element {
        self.0.element(m)
    }

    fn place_holder() -> Self {
        Self(UIBadge::place_holder())
    }
}
impl FromRef<(KeyProviderType, ExternalDependencyStatus)> for UIKeyProviderBadge {
    fn from_ref(&(kpt, eds): &(KeyProviderType, ExternalDependencyStatus)) -> Self {
        let text = match kpt {
            KeyProviderType::None => "Watch Only",
            KeyProviderType::LocalKey => "Local Key",
            KeyProviderType::Ledger => "Ledger",
        };
        let color_class = match kpt {
            KeyProviderType::None | KeyProviderType::LocalKey => "badge-secondary",
            _ => eds.color_class(),
        };
        let tooltip = match kpt {
            KeyProviderType::None => "This wallet cannot sign transactions but may still access the blockchain and generate them",
            KeyProviderType::LocalKey => "This wallet can sign transactions",
            KeyProviderType::Ledger => "This wallet can sign transactions using your Ledger",
        };
        UIKeyProviderBadge(UIBadge {
            text,
            color_class,
            tooltip,
        })
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct UIOnlineWalletBadge(UIBadge);
impl LoadedElement for UIOnlineWalletBadge {
    type Loader = SkeletonLoader;
    #[inline(always)]
    fn element<M: LoadedComponentInputMapper>(self, m: M) -> Element {
        self.0.element(m)
    }

    fn place_holder() -> Self {
        Self(UIBadge::place_holder())
    }
}
impl FromRef<(OnlineWalletType, ExternalDependencyStatus)> for UIOnlineWalletBadge {
    fn from_ref(&(owt, eds): &(OnlineWalletType, ExternalDependencyStatus)) -> Self {
        let text = match owt {
            OnlineWalletType::None => "Sign Only",
            OnlineWalletType::Service => "Service",
            OnlineWalletType::Local => "Local Node",
        };
        let color_class = match owt {
            OnlineWalletType::None => "badge-secondary",
            _ => eds.color_class(),
        };
        let tooltip = match owt {
            OnlineWalletType::None => "This wallet have no access to the blockchain but may still sign transaction offline",
            OnlineWalletType::Service => "Wallet rely on the Heritage Service for blockchain operations",
            OnlineWalletType::Local =>"Wallet rely on a local node for blockchain operations",
        };
        UIOnlineWalletBadge(UIBadge {
            text,
            color_class,
            tooltip,
        })
    }
}

const HEIR_DB_BADGE: UIBadge = UIBadge {
    text: "App",
    color_class: "badge-info",
    tooltip: "Heir is registered in this Heritage app",
};
const HEIR_SERVICE_BADGE: UIBadge = UIBadge {
    text: "Service",
    color_class: "badge-success",
    tooltip: "Heir is registered in the Heritage service",
};
const HEIR_LOCAL_SEED_BADGE: UIBadge = UIBadge {
    text: "Seed",
    color_class: "badge-secondary",
    tooltip: "You have the private seed for this heir",
};

#[derive(Debug, Clone, PartialEq)]
pub struct UIHeirBadges(Vec<UIBadge>);
impl LoadedElement for UIHeirBadges {
    type Loader = TransparentLoader;
    #[inline(always)]
    fn element<M: LoadedComponentInputMapper>(self, m: M) -> Element {
        rsx! {
            for badge in self.0 {
                LoadedComponent { input: m.map(badge) }
            }
        }
    }

    fn place_holder() -> Self {
        Self(vec![UIBadge::place_holder(); 2].into())
    }
}
impl FromRef<CompositeHeir> for UIHeirBadges {
    fn from_ref(heir: &CompositeHeir) -> Self {
        let CompositeHeir {
            db_heir,
            service_heir,
            ..
        } = heir;
        let mut badges = vec![];
        if let Some(db_heir) = db_heir {
            match db_heir.key_provider() {
                AnyKeyProvider::None => (),
                AnyKeyProvider::LocalKey(_) | AnyKeyProvider::Ledger(_) => {
                    badges.push(HEIR_LOCAL_SEED_BADGE)
                }
            }
        }
        if db_heir.is_some() {
            badges.push(HEIR_DB_BADGE);
        }
        if service_heir.is_some() {
            badges.push(HEIR_SERVICE_BADGE);
        }
        UIHeirBadges(badges)
    }
}
