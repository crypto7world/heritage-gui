use btc_heritage_wallet::{online_wallet::WalletStatus, AnyKeyProvider, AnyOnlineWallet, Wallet};
use dioxus::prelude::*;

#[component]
pub fn KeyProviderBadge(wallet: ReadOnlySignal<Option<Wallet>>) -> Element {
    log::debug!("KeyProviderBadge reload");

    let key_provider_badge = use_memo(move || {
        log::debug!("key_provider_badge - memo compute");
        wallet
            .read()
            .as_ref()
            .map(|wallet| match wallet.key_provider() {
                AnyKeyProvider::None => ("Watch-Only", "badge-secondary"),
                AnyKeyProvider::LocalKey(_) => ("Local Key", "badge-secondary"),
                AnyKeyProvider::Ledger(_) => ("Ledger", "badge-secondary"),
            })
    });

    use_drop(|| log::debug!("KeyProviderBadge Dropped"));

    rsx! {
        if let Some((content, color)) = key_provider_badge() {
            div { class: "badge shadow-xl {color}", {content} }
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
        log::debug!("online_wallet_badge - memo compute");
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
            })
    });

    use_drop(|| log::debug!("OnlineWalletBadge Dropped"));

    rsx! {
        if let Some((content, color)) = online_wallet_badge() {
            div {
                class: "badge shadow-xl",
                class: if wallet_status.read().is_none() { "skeleton" } else { "{color}" },
                {content}
            }
        }
    }
}
