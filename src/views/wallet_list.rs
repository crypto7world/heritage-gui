use std::ops::Deref;

use dioxus::prelude::*;

use super::TitledView;
use crate::{
    clients::{database, database_mut, service_client, UserId},
    gui::Route,
    utils::{amount_to_string, log_error, timestamp_to_string},
};
use btc_heritage_wallet::{
    bitcoin::Amount, online_wallet::WalletStatus, AnyKeyProvider, AnyOnlineWallet,
    BoundFingerprint, DatabaseItem, OnlineWallet, Wallet,
};

#[component]
pub fn WalletListView() -> Element {
    rsx! {
        TitledView {
            title: "Wallets",
            subtitle: "Heritage wallets with simple Heritage configurations instead of complex Bitcoin scripts.",
            WalletList {}
        }
    }
}

#[component]
fn WalletList() -> Element {
    let mut db = database_mut();

    let wallet_names = Wallet::list_names(&db)
        .map_err(log_error)
        .unwrap_or_default();

    rsx! {
        div { class: "max-w-80 md:container mx-auto grid grid-cols-1 md:grid-cols-2 xl:grid-cols-3 2xl:grid-cols-4 gap-12",
            for wallet_name in wallet_names {
                WalletItem { key: "{wallet_name}", wallet_name }
            }
            for i in 0..30 {

                div { class: "card card-compact aspect-square border", "Pouet {i}" }
            }
        }
    }
}

#[component]
fn WalletItem(wallet_name: String) -> Element {
    let navigator = use_navigator();
    let user_id = use_context::<Signal<Option<UserId>, SyncStorage>>();

    let wallet = Wallet::load(database().deref(), &wallet_name)
        .expect("wallet should exist and I have nothing smart to do with this error anyway");

    let key_provider_badge_value = match wallet.key_provider() {
        AnyKeyProvider::None => "Watch-Only",
        AnyKeyProvider::LocalKey(_) => "Local Key",
        AnyKeyProvider::Ledger(_) => "Ledger",
    };
    let key_provider_badge_color = match wallet.key_provider() {
        AnyKeyProvider::None => "badge-secondary",
        AnyKeyProvider::LocalKey(_) => "badge-secondary",
        AnyKeyProvider::Ledger(_) => "badge-secondary",
    };

    let online_wallet_badge_value = match wallet.online_wallet() {
        AnyOnlineWallet::None => "Sign-Only",
        AnyOnlineWallet::Service(_) => "Service",
        AnyOnlineWallet::Local(_) => "Local Node",
    };

    let wallet_status = match wallet.online_wallet() {
        AnyOnlineWallet::None => Some(Err::<WalletStatus, ()>(())),
        AnyOnlineWallet::Service(service_binding) => {
            let service_binding = service_binding.clone();
            use_resource(move || {
                // Bind to user ID
                let _ = user_id.read();
                let mut service_binding = service_binding.clone();
                async {
                    tokio::task::spawn_blocking(move || {
                        if !service_binding.has_service_client() {
                            service_binding.init_service_client_unchecked(service_client());
                        }
                        service_binding.get_wallet_status().map_err(|e| {
                            log::warn!("{e}");
                            ()
                        })
                    })
                    .await
                    .unwrap()
                }
            })
            .cloned()
        }
        AnyOnlineWallet::Local(local_heritage_wallet) => {
            Some(local_heritage_wallet.get_wallet_status().map_err(|e| {
                log::warn!("{e}");
                ()
            }))
        }
    };

    let online_wallet_badge_color = match wallet.online_wallet() {
        AnyOnlineWallet::None => "badge-secondary",
        AnyOnlineWallet::Service(_) => {
            if user_id.read().is_none() {
                "badge-error"
            } else {
                match wallet_status {
                    Some(Ok(_)) => "badge-success",
                    Some(Err(_)) => "badge-error",
                    None => "badge-secondary",
                }
            }
        }
        AnyOnlineWallet::Local(_) => "badge-secondary",
    };

    let is_loading = wallet_status.is_none();

    let last_sync = wallet_status.as_ref().map(|rws| match rws {
        Ok(ws) => timestamp_to_string(ws.last_sync_ts),
        Err(_) => "-".to_owned(),
    });

    let balance = wallet_status.as_ref().map(|rws| match rws {
        Ok(ws) => amount_to_string(Amount::from_sat(ws.balance.total_balance().get_total())),
        Err(_) => "-".to_owned(),
    });

    let cur_balance = wallet_status.as_ref().map(|rws| match rws {
        Ok(ws) => amount_to_string(Amount::from_sat(ws.balance.uptodate_balance().get_total())),
        Err(_) => "-".to_owned(),
    });
    let obs_balance = wallet_status.as_ref().map(|rws| match rws {
        Ok(ws) => amount_to_string(Amount::from_sat(ws.balance.obsolete_balance().get_total())),
        Err(_) => "-".to_owned(),
    });

    let fingerprint = wallet
        .fingerprint()
        .map(|fg| fg.to_string())
        .unwrap_or_else(|e| {
            log::warn!("{e}");
            "-".to_owned()
        });

    rsx! {
        div {
            class: "card card-compact aspect-square border",
            onclick: move |_| {
                navigator
                    .push(Route::WalletView {
                        wallet_name: wallet.name().to_owned(),
                    });
            },
            div { class: "card-body",
                div {
                    div { class: "card-title text-3xl font-black", "{wallet.name()}" }
                    div { class: "text-sm font-light", {fingerprint} }
                }
                div { class: "grow" }

                if is_loading {
                    div { class: "skeleton h-20 w-20" }
                } else {
                    div { class: "text-sm text-left", "Balance" }
                    div { class: "text-4xl font-black text-center", {balance} }
                    div { class: "text-medium font-light text-center",
                        "Current: "
                        {cur_balance}
                        " | Obsolete: "
                        {obs_balance}
                    }
                    div { class: "text-sm font-light text-left",
                        "Last Sync: "
                        {last_sync}
                    }
                }

                div { class: "grow" }
                div { class: "mx-auto flex flex-row gap-6",
                    div { class: "badge {key_provider_badge_color}", {key_provider_badge_value} }
                    div { class: "badge {online_wallet_badge_color}", {online_wallet_badge_value} }
                }
            }
        }
    }
}
