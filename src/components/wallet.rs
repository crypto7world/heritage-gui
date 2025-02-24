use dioxus::prelude::*;

use btc_heritage_wallet::{
    bitcoin::{Amount, SignedAmount},
    online_wallet::WalletStatus,
    AnyKeyProvider, AnyOnlineWallet, Wallet,
};

use crate::utils::{amount_to_signed_string, LoadedElement, PlaceHolder};

#[component]
pub fn KeyProviderBadge(wallet: ReadOnlySignal<Option<Wallet>>) -> Element {
    log::debug!("KeyProviderBadge reload");

    let key_provider_badge = use_memo(move || {
        log::debug!("use_memo_key_provider_badge - start compute");
        let key_provider_badge = wallet
            .read()
            .as_ref()
            .map(|wallet| match wallet.key_provider() {
                AnyKeyProvider::None => ("Watch-Only", "badge-secondary"),
                AnyKeyProvider::LocalKey(_) => ("Local Key", "badge-secondary"),
                AnyKeyProvider::Ledger(_) => ("Ledger", "badge-secondary"),
            });
        log::debug!("use_memo_key_provider_badge - finish compute");
        key_provider_badge
    });

    use_drop(|| log::debug!("KeyProviderBadge Dropped"));

    rsx! {
        if let Some((content, color)) = key_provider_badge() {
            div { class: "badge shadow-xl text-nowrap {color}", {content} }
        }
    }
}

#[component]
pub fn OnlineWalletBadge(
    wallet: ReadOnlySignal<Option<Wallet>>,
    wallet_status: ReadOnlySignal<Option<Option<WalletStatus>>>,
) -> Element {
    log::debug!("OnlineWalletBadge reload");

    let online_wallet_badge = use_memo(move || {
        log::debug!("use_memo_online_wallet_badge - start compute");
        let online_wallet_badge =
            wallet
                .read()
                .as_ref()
                .map(|wallet| match wallet.online_wallet() {
                    AnyOnlineWallet::None => ("Sign-Only", "badge-secondary"),
                    AnyOnlineWallet::Service(_) => (
                        "Service",
                        match *wallet_status.read() {
                            Some(Some(_)) => "badge-success",
                            Some(None) => "badge-error",
                            None => "badge-secondary",
                        },
                    ),
                    AnyOnlineWallet::Local(_) => (
                        "Local Node",
                        match *wallet_status.read() {
                            Some(Some(_)) => "badge-success",
                            Some(None) => "badge-error",
                            None => "badge-secondary",
                        },
                    ),
                });
        log::debug!("use_memo_online_wallet_badge - finish compute");
        online_wallet_badge
    });

    use_drop(|| log::debug!("OnlineWalletBadge Dropped"));

    rsx! {
        if let Some((content, color)) = online_wallet_badge() {
            div {
                class: "badge shadow-xl text-nowrap",
                class: if wallet_status.read().is_none() { "skeleton" } else { "{color}" },
                {content}
            }
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum DisplayBtcAmount {
    Amount(SignedAmount),
    #[default]
    None,
}
impl PlaceHolder for DisplayBtcAmount {
    fn place_holder() -> Self {
        Self::Amount(SignedAmount::place_holder())
    }
}
impl From<Option<SignedAmount>> for DisplayBtcAmount {
    fn from(value: Option<SignedAmount>) -> Self {
        match value {
            Some(amount) => DisplayBtcAmount::Amount(amount),
            None => DisplayBtcAmount::None,
        }
    }
}
impl From<SignedAmount> for DisplayBtcAmount {
    fn from(amount: SignedAmount) -> Self {
        DisplayBtcAmount::Amount(amount)
    }
}
impl From<Amount> for DisplayBtcAmount {
    /// Automatically converts the given [Amount] to a [SignedAmount]
    /// and simply [unwrap()] the result of [Amount::to_signed]
    fn from(amount: Amount) -> Self {
        DisplayBtcAmount::Amount(
            amount
                .to_signed()
                .expect("`Normal` amounts cannot be bigger than MAX_MONEY"),
        )
    }
}
#[component]
pub fn BtcAmount(amount: LoadedElement<DisplayBtcAmount>, diff_style: Option<bool>) -> Element {
    let (is_place_holder, amount) = amount.extract();

    let diff_style = diff_style.unwrap_or_default();

    let (is_positive, is_negative, amount_s) = match amount {
        DisplayBtcAmount::Amount(signed_amount) => (
            signed_amount.is_positive() || signed_amount == SignedAmount::ZERO,
            signed_amount.is_negative(),
            amount_to_signed_string(signed_amount),
        ),
        DisplayBtcAmount::None => (false, false, "-".to_owned()),
    };

    let display_text = if !diff_style && is_positive {
        &amount_s[1..]
    } else {
        amount_s.as_str()
    };
    rsx! {
        span {
            class: "text-nowrap inline-block",
            class: if diff_style && is_positive { "text-green-500" },
            class: if diff_style && is_negative { "text-red-500" },
            class: if is_place_holder { "skeleton text-transparent" },
            {display_text}
        }
    }
}
