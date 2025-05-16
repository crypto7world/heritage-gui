use dioxus::prelude::*;

use btc_heritage_wallet::{
    bitcoin::{Amount, SignedAmount},
    online_wallet::WalletStatus,
    AnyKeyProvider, AnyOnlineWallet, Wallet,
};

use crate::{
    components::misc::{Badge, SkeletonBadgeType, WalletBadgeType},
    utils::amount_to_signed_string,
};

use super::loaded::{FromRef, LoadedComponent, LoadedElement};
