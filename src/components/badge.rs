use crate::prelude::*;

use btc_heritage_wallet::AnyKeyProvider;

use crate::{components::misc::TextTooltip, utils::CCStr};

#[derive(Debug, Clone, Copy, PartialEq)]
#[allow(dead_code)]
pub enum UIBadgeStyle {
    InDatabase,
    HeritageService,
    LocalNode,
    LocalKey,
    Ledger,
    NoComponent,
    Custom(&'static str),
}
impl UIBadgeStyle {
    fn classes(self) -> &'static str {
        match self {
            UIBadgeStyle::InDatabase => "bg-pink-500 text-white",
            UIBadgeStyle::HeritageService => "border-2 border-white bg-[#333333] outline outline-[#333333] text-[#e33f35] font-black",
            UIBadgeStyle::LocalNode => "bg-orange-400 text-secondary-content",
            UIBadgeStyle::LocalKey => "bg-[#2e86ab] text-white",
            UIBadgeStyle::Ledger =>  unreachable!("classes() is never called on UIBadgeStyle::Ledger"),
            UIBadgeStyle::NoComponent => "border-base-content",
            UIBadgeStyle::Custom(classes) => classes,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct UIBadge {
    pub text: &'static str,
    pub badge_style: UIBadgeStyle,
    pub tooltip: &'static str,
    pub status: ExternalDependencyStatus,
}
impl LoadedElement for UIBadge {
    type Loader = SkeletonLoader;
    #[inline(always)]
    fn element<M: LoadedComponentInputMapper>(self, _m: M) -> Element {
        let base = rsx! {
            TextTooltip { tooltip_text: CCStr::from(self.tooltip),
                match self.badge_style {
                    UIBadgeStyle::Ledger => rsx! {
                        div { class: "relative outline outline-black",
                            div { class: "badge rounded-none px-1 h-fit shadow-xl text-nowrap font-mono uppercase \
                                                                                                                                                                                                                                                                                                                                                                                font-semibold text-white bg-black border-3 border-white box-content",
                                {self.text}
                            }
                            div { class: "absolute w-full h-1/2 top-1/4 left-0 border-l-4 border-r-4 border-black" }
                        
                            div { class: "absolute h-full w-2/3 top-0 left-1/6 border-t-4 border-b-4 border-black" }
                        }
                    },
                    _ => rsx! {
                        div { class: "badge shadow-xl text-nowrap {self.badge_style.classes()}", {self.text} }
                    },
                }
            }
        };
        match self.status {
            ExternalDependencyStatus::Available
            | ExternalDependencyStatus::Unavailable
            | ExternalDependencyStatus::NeedUserAction => rsx! {
                div { class: "indicator",
                    span { class: "indicator-item status {self.status.color_class()} status-lg" }
                    {base}
                }
            },
            ExternalDependencyStatus::None => base,
        }
    }

    fn place_holder() -> Self {
        Self {
            text: <&'static str>::place_holder(),
            badge_style: UIBadgeStyle::NoComponent,
            tooltip: <&'static str>::place_holder(),
            status: ExternalDependencyStatus::None,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum HeritageProviderType {
    None,
    Service,
    LocalWallet,
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
    NeedUserAction,
    None,
}
impl ExternalDependencyStatus {
    fn color_class(self) -> &'static str {
        match self {
            Self::Available => "status-success",
            Self::Unavailable => "status-error",
            Self::NeedUserAction => "status-warning",
            Self::None => {
                unreachable!("color_class() is never called on ExternalDependencyStatus::None")
            }
        }
    }
}

impl LoadedSuccessConversionMarker
    for TypeCouple<(KeyProviderType, ExternalDependencyStatus), UIBadge>
{
}
impl FromRef<(KeyProviderType, ExternalDependencyStatus)> for UIBadge {
    fn from_ref(&(kpt, eds): &(KeyProviderType, ExternalDependencyStatus)) -> Self {
        match kpt {
            KeyProviderType::None => Self {
                text: "Watch Only",
                badge_style: UIBadgeStyle::NoComponent,
                tooltip: "Cannot sign transactions but may still generate them",
                status: eds,
            },
            KeyProviderType::LocalKey => Self {
                text: "Local Key",
                badge_style: UIBadgeStyle::LocalKey,
                tooltip: "Can sign transactions with a local private key",
                status: eds,
            },
            KeyProviderType::Ledger => Self {
                text: "Ledger",
                badge_style: UIBadgeStyle::Ledger,
                tooltip: "Can sign transactions using your Ledger device",
                status: eds,
            },
        }
    }
}

impl LoadedSuccessConversionMarker
    for TypeCouple<(OnlineWalletType, ExternalDependencyStatus), UIBadge>
{
}
impl FromRef<(OnlineWalletType, ExternalDependencyStatus)> for UIBadge {
    fn from_ref(&(owt, eds): &(OnlineWalletType, ExternalDependencyStatus)) -> Self {
        match owt {
            OnlineWalletType::None => Self {
                text: "Sign Only",
                badge_style: UIBadgeStyle::NoComponent,
                tooltip: "Have no access to the blockchain but may still sign transaction",
                status: eds,
            },
            OnlineWalletType::Service => Self {
                text: "Service",
                badge_style: UIBadgeStyle::HeritageService,
                tooltip: "Rely on the Heritage Service for blockchain data access",
                status: eds,
            },
            OnlineWalletType::Local => Self {
                text: "Local Node",
                badge_style: UIBadgeStyle::LocalNode,
                tooltip: "Rely on the local database for data access and a Blockchain provider for synchronization",
                status: eds,
            },
        }
    }
}

impl LoadedSuccessConversionMarker
    for TypeCouple<(HeritageProviderType, ExternalDependencyStatus), UIBadge>
{
}
impl FromRef<(HeritageProviderType, ExternalDependencyStatus)> for UIBadge {
    fn from_ref(&(ept, eds): &(HeritageProviderType, ExternalDependencyStatus)) -> Self {
        match ept {
            HeritageProviderType::None => Self {
                text: "Sign Only",
                badge_style: UIBadgeStyle::NoComponent,
                tooltip: "Cannot find inheritances but may still sign transaction",
                status: eds,
            },
            HeritageProviderType::Service => Self {
                text: "Service",
                badge_style: UIBadgeStyle::HeritageService,
                tooltip: "Rely on the Heritage Service to find inheritances",
                status: eds,
            },
            HeritageProviderType::LocalWallet => Self {
                text: "Local Node",
                badge_style: UIBadgeStyle::LocalNode,
                tooltip: "Rely on the local database to find inheritances and a Blockchain provider for synchronization",
                status: eds,
            },
        }
    }
}

const HEIR_DB_BADGE: UIBadge = UIBadge {
    text: "App",
    badge_style: UIBadgeStyle::InDatabase,
    tooltip: "Heir is registered in this Heritage app",
    status: ExternalDependencyStatus::None,
};
const HEIR_SERVICE_BADGE: UIBadge = UIBadge {
    text: "Service",
    badge_style: UIBadgeStyle::HeritageService,
    tooltip: "Heir is registered in the Heritage service",
    status: ExternalDependencyStatus::None,
};
const HEIR_LOCAL_SEED_BADGE: UIBadge = UIBadge {
    text: "Seed",
    badge_style: UIBadgeStyle::LocalKey,
    tooltip: "You have the private seed mnemonic for this heir",
    status: ExternalDependencyStatus::None,
};

#[derive(Debug, Clone, PartialEq)]
pub struct UIHeirBadges(Vec<UIBadge>);
impl LoadedElement for UIHeirBadges {
    type Loader = TransparentLoader;
    #[inline(always)]
    fn element<M: LoadedComponentInputMapper>(self, m: M) -> Element {
        rsx! {
            LoadedComponent::<Vec<UIBadge>> { input: m.map(self.0) }
        }
    }

    fn place_holder() -> Self {
        Self(vec![UIBadge::place_holder(); 2].into())
    }
}
impl LoadedSuccessConversionMarker for TypeCouple<CompositeHeir, UIHeirBadges> {}
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
        if service_heir.as_ref().is_some_and(|inner| inner.is_some()) {
            badges.push(HEIR_SERVICE_BADGE);
        }
        UIHeirBadges(badges)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct UIInheritanceBadges(Vec<UIBadge>);
impl LoadedElement for UIInheritanceBadges {
    type Loader = TransparentLoader;
    #[inline(always)]
    fn element<M: LoadedComponentInputMapper>(self, m: M) -> Element {
        rsx! {
            LoadedComponent::<Vec<UIBadge>> { input: m.map(self.0) }
        }
    }

    fn place_holder() -> Self {
        Self(vec![UIBadge::place_holder(); 2].into())
    }
}
