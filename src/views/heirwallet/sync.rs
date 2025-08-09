use crate::prelude::*;

use btc_heritage_wallet::{
    btc_heritage::utils::timestamp_now, AnyHeritageProvider, HeirWallet, OnlineWallet,
};

use crate::{
    components::{
        svg::{DrawSvg, SvgSize::Full, Sync},
        timestamp::UITimestamp,
    },
    utils::CCStr,
};

/// A component that displays wallet synchronization status and provides sync functionality
///
/// For Heritage Service wallets, implements a 1-minute cooldown after successful sync
/// to respect the service's global sync lock.
#[component]
pub fn HeirWalletLocalSync() -> Element {
    log::debug!("HeirWalletSync Rendered");

    let blockchain_provider_service = state_management::use_blockchain_provider_service();

    let mut heirwallet = use_context::<AsyncSignal<HeirWallet>>();

    let heirwallet_local_lastsync = use_context::<FResource<Option<u64>>>();

    let ui_heirwallet_local_lastsync = use_memo(move || match *heirwallet_local_lastsync.read() {
        Some(Ok(Some(ts))) if ts == 0 => Some(UITimestamp::never()),
        Some(Ok(Some(ts))) => Some(UITimestamp::new_full(ts)),
        _ => None,
    });
    let can_sync = use_memo(
        move || match state_management::BLOCKCHAIN_PROVIDER_STATUS() {
            Some(BlockchainProviderStatus::Connected(_)) => true,
            _ => false,
        },
    );
    let mut syncing = use_signal(|| false);
    let click_sync = move |_| async move {
        *syncing.write() = true;
        let blockchain_factory =
            match state_management::blockchain_factory(blockchain_provider_service).await {
                Ok(bcf) => bcf,
                Err(e) => {
                    log::error!("{e}");
                    alert_error(e);
                    return;
                }
            };

        match heirwallet
            .with_mut(async |hw: &mut HeirWallet| {
                let AnyHeritageProvider::LocalWallet(lw) = hw.heritage_provider_mut() else {
                    return Err(CCStr::from("Cannot sync a non-local Heritage Provider"));
                };
                let lhw = lw.local_heritage_wallet_mut();
                lhw.init_blockchain_factory(blockchain_factory);
                lhw.sync().await.map_err(|e| CCStr::from(e.to_string()))
            })
            .await
        {
            Ok(()) => {
                log::info!("Heir Wallet synced with blockchain");
                alert_success("Heir Wallet synced with blockchain");
            }
            Err(e) => {
                log::error!("{e}");
                alert_error(e);
            }
        }

        *syncing.write() = false;
    };

    let recently_synced = use_memo(move || {
        // Verify it has been synced less than an hour ago
        heirwallet_local_lastsync
            .lrmap_ok(|ots| ots.is_some_and(|ts| timestamp_now() < ts + 3600))
            .unwrap_or(false)
    });

    use_drop(|| log::debug!("HeirWalletSync Dropped"));

    rsx! {
        if let Some(ui_heirwallet_local_lastsync) = ui_heirwallet_local_lastsync() {
            MaybeHighlight {
                step: OnboardingStep::SynchronizeLocalHeritage,
                progress: MaybeHighlightProgressType::Signal(recently_synced.into()),
                div { class: "h-full flex gap-2 items-center",
                    button {
                        class: "btn btn-circle btn-outline btn-primary btn-lg p-2",
                        onclick: click_sync,
                        disabled: !can_sync() || syncing(),
                        DrawSvg::<Sync> { size: Full }
                    }
                    div { class: "h-fit",
                        div { class: "text-base font-light", "Last synced:" }
                        div { class: "text-lg font-semibold text-nowrap",
                            AlwaysLoadedComponent { input: ui_heirwallet_local_lastsync }
                        }
                    }
                }
            }
        }
    }
}
