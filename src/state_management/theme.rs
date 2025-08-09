use dioxus::prelude::*;

use btc_heritage_wallet::DatabaseSingleItem;
use futures_util::stream::StreamExt;
use serde::{Deserialize, Serialize};

use crate::prelude::alert_error;

use super::{
    database::{DatabaseCommand, DatabaseReloadEvent},
    event_bus::{subscribe_event, EventBus},
};

#[derive(Debug, Clone, Copy, PartialEq, Default, Serialize, Deserialize)]
pub enum Theme {
    Light,
    #[default]
    Dark,
}

impl DatabaseSingleItem for Theme {
    fn item_key() -> &'static str {
        "gui_dark_theme"
    }
}

pub static THEME: GlobalSignal<Theme> = Signal::global(|| Theme::Dark);

/// Commands for the blockchain provider service
#[derive(Debug)]
pub(super) enum DarkModeCommand {
    /// Internal trigger a save into the DB
    Persist { theme: Theme },
    /// Internal trigger a refresh from the DB
    Refresh,
}

/// Theme service coroutine
pub(super) fn use_theme_service(
    event_bus: EventBus,
    database_service: Coroutine<DatabaseCommand>,
) -> Coroutine<DarkModeCommand> {
    let service_handle = use_coroutine(
        move |mut rx: UnboundedReceiver<DarkModeCommand>| async move {
            log::info!("darkmode_service (coroutine) - start");

            let mut cached_value = load_theme(database_service).await;
            *THEME.write() = cached_value;

            // Flag to trash the first persist command that will come from the initial run of the "use_effect"
            let mut discard_next_persit = true;

            while let Some(cmd) = rx.next().await {
                log::debug!("darkmode_service (coroutine) - Processing command {cmd:?}...");

                match cmd {
                    DarkModeCommand::Persist { theme } => {
                        if discard_next_persit {
                            log::debug!(
                                "darkmode_service (coroutine) - Ignoring first Persist cmd."
                            );
                            discard_next_persit = false;
                        }
                        if theme != cached_value {
                            log::debug!(
                                "darkmode_service (coroutine) - Status changed, persisting..."
                            );
                            cached_value = theme;

                            match save_theme(database_service, theme).await {
                                Ok(_) => (),
                                Err(msg) => {
                                    log::error!("{msg}");
                                    alert_error(msg);
                                }
                            };
                        } else {
                            log::debug!("darkmode_service (coroutine) - Ignoring Persist cmd: already in database.");
                        }
                    }
                    DarkModeCommand::Refresh => {
                        cached_value = load_theme(database_service).await;
                        *THEME.write() = cached_value
                    }
                }

                log::debug!("onboarding_service (coroutine) - Command processed");
            }
        },
    );
    subscribe_event(event_bus, move |_event: DatabaseReloadEvent| {
        service_handle.send(DarkModeCommand::Refresh);
    });
    use_effect(move || {
        service_handle.send(DarkModeCommand::Persist { theme: THEME() });
    });
    service_handle
}

async fn load_theme(database_service: Coroutine<DatabaseCommand>) -> Theme {
    let database = super::helpers::get_database(database_service).await;

    match database.blocking_operation(|db| Theme::load(&db)).await {
        Ok(theme) => theme,
        Err(_) => Theme::default(),
    }
}

async fn save_theme(
    database_service: Coroutine<DatabaseCommand>,
    theme: Theme,
) -> Result<(), String> {
    let database = super::helpers::get_database(database_service).await;
    database
        .blocking_operation(move |mut db| theme.save(&mut db))
        .await
        .map_err(|e| e.to_string())
}
