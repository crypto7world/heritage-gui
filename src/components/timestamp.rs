use crate::prelude::*;

use btc_heritage_wallet::{
    heritage_service_api_client::HeritageWalletMeta, online_wallet::WalletStatus,
};

use crate::utils::{timestamp_to_date_string, timestamp_to_string};

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
    type Loader = SkeletonLoader;
    #[inline(always)]
    fn element<M: LoadedComponentInputMapper>(self, _m: M) -> Element {
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
impl LoadedSuccessConversionMarker for TypeCouple<WalletStatus, LastSyncSpan> {}
impl FromRef<WalletStatus> for LastSyncSpan {
    fn from_ref(wallet_status: &WalletStatus) -> Self {
        Self(UITimestamp::new_full(wallet_status.last_sync_ts))
    }
}
impl LoadedSuccessConversionMarker for TypeCouple<HeritageWalletMeta, LastSyncSpan> {}
impl FromRef<HeritageWalletMeta> for LastSyncSpan {
    fn from_ref(wallet_meta: &HeritageWalletMeta) -> Self {
        Self(UITimestamp::new_full(wallet_meta.last_sync_ts))
    }
}
impl LoadedElement for LastSyncSpan {
    type Loader = SkeletonLoader;
    #[inline(always)]
    fn element<M: LoadedComponentInputMapper>(self, m: M) -> Element {
        self.0.element(m)
    }

    fn place_holder() -> Self {
        Self(UITimestamp::place_holder())
    }
}
