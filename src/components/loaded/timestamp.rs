use btc_heritage_wallet::{
    heritage_service_api_client::HeritageWalletMeta, online_wallet::WalletStatus,
};
use dioxus::prelude::*;

use crate::utils::{timestamp_to_date_string, timestamp_to_string};

use super::{ComponentMapper, FromRef, LoadedElement};

#[derive(Debug, Clone, Copy, PartialEq)]
enum UITimestampInner {
    Ts(u64),
    None,
    Never,
}
impl From<u64> for UITimestampInner {
    fn from(value: u64) -> Self {
        UITimestampInner::Ts(value)
    }
}
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
enum UITimestampStyle {
    #[default]
    Full,
    DateOnly,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct UITimestamp {
    ts: UITimestampInner,
    style: UITimestampStyle,
}
impl UITimestamp {
    pub fn new_full(ts: u64) -> Self {
        Self {
            ts: UITimestampInner::Ts(ts),
            style: UITimestampStyle::Full,
        }
    }
    pub fn new_date_only(ts: u64) -> Self {
        Self {
            ts: UITimestampInner::Ts(ts),
            style: UITimestampStyle::DateOnly,
        }
    }
    pub fn never() -> Self {
        Self {
            ts: UITimestampInner::Never,
            style: UITimestampStyle::default(),
        }
    }
    pub fn none() -> Self {
        Self {
            ts: UITimestampInner::None,
            style: UITimestampStyle::default(),
        }
    }
}
impl LoadedElement for UITimestamp {
    #[inline(always)]
    fn element<CM: ComponentMapper>(self, _mapper: CM) -> Element {
        let last_synced_s = match self.ts {
            UITimestampInner::Ts(timestamp) => match self.style {
                UITimestampStyle::Full => timestamp_to_string(timestamp),
                UITimestampStyle::DateOnly => timestamp_to_date_string(timestamp),
            },
            UITimestampInner::None => "-".to_owned(),
            UITimestampInner::Never => "Never".to_owned(),
        };
        rsx! {
            span { class: "text-nowrap inline-block", {last_synced_s} }
        }
    }

    fn place_holder() -> Self {
        Self {
            ts: UITimestampInner::Ts(0),
            style: UITimestampStyle::default(),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct LastSyncSpan(UITimestamp);
impl FromRef<WalletStatus> for LastSyncSpan {
    fn from_ref(wallet_status: &WalletStatus) -> Self {
        Self(UITimestamp::new_full(wallet_status.last_sync_ts))
    }
}
impl FromRef<HeritageWalletMeta> for LastSyncSpan {
    fn from_ref(wallet_meta: &HeritageWalletMeta) -> Self {
        Self(UITimestamp::new_full(wallet_meta.last_sync_ts))
    }
}
impl LoadedElement for LastSyncSpan {
    #[inline(always)]
    fn element<CM: ComponentMapper>(self, mapper: CM) -> Element {
        self.0.element(mapper)
    }

    fn place_holder() -> Self {
        Self(UITimestamp::place_holder())
    }
}
