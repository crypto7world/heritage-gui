use crate::prelude::*;

use std::time::Duration;

use btc_heritage_wallet::{online_wallet::WalletStatus, OnlineWallet, Wallet};

use crate::components::{
    badge::{ExternalDependencyStatus, OnlineWalletType},
    svg::{DrawSvg, SvgSize::Full, Sync},
    timestamp::LastSyncSpan,
};

/// A component that displays wallet synchronization status and provides sync functionality
///
/// For Heritage Service wallets, implements a 1-minute cooldown after successful sync
/// to respect the service's global sync lock.
#[component]
pub fn WalletSync() -> Element {
    log::debug!("WalletSync Rendered");

    let mut wallet = use_context::<AsyncSignal<Wallet>>();
    let wallet_status = use_context::<FResource<WalletStatus>>();

    let online_status = helper_hooks::use_memo_wallet_online_status(wallet);

    let mut syncing = use_signal(|| false);

    let mut online_sync_cooldown_active = use_signal(|| false);
    let sync_available = use_memo(move || match online_status() {
        Some((OnlineWalletType::Service, ExternalDependencyStatus::Available)) => {
            !online_sync_cooldown_active()
        }
        Some((OnlineWalletType::Local, ExternalDependencyStatus::Available)) => true,
        _ => false,
    });

    let click_sync = move |_| async move {
        *syncing.write() = true;
        match wallet
            .with_mut(async |wallet: &mut Wallet| wallet.sync().await)
            .await
        {
            Ok(_) => {
                log::info!("Successfully synced wallet");
                alert_info("Wallet Synced.");
            }
            Err(e) => {
                alert_error(format!("Failed to sync: {e}"));
                log::error!("Failed to sync: {e}");
            }
        }
        *syncing.write() = false;
        *online_sync_cooldown_active.write() = true;
        tokio::time::sleep(Duration::from_secs(60)).await;
        *online_sync_cooldown_active.write() = false;
    };

    use_drop(|| log::debug!("WalletSync Dropped"));

    rsx! {
        div { class: "h-full flex gap-2 items-center",
            button {
                class: "btn btn-circle btn-outline btn-primary btn-lg p-2",
                onclick: click_sync,
                disabled: !sync_available() || syncing(),
                DrawSvg::<Sync> { size: Full }
            }
            div { class: "h-fit",
                div { class: "text-base font-light", "Last synced:" }
                div { class: "text-base font-bold",
                    LoadedComponent::<LastSyncSpan> { input: wallet_status.into() }
                }
            }
        }
    }
}
