use dioxus::prelude::*;

use btc_heritage_wallet::DatabaseSingleItem;
use futures_util::stream::StreamExt;
use serde::{Deserialize, Serialize};

use crate::{onboarding::Onboarding, prelude::alert_error};

use super::{
    database::{DatabaseCommand, DatabaseReloadEvent},
    event_bus::{subscribe_event, EventBus},
};

#[derive(Debug, Default, PartialEq, Serialize, Deserialize)]
pub enum OnboardingStatus {
    #[default]
    Pending,
    InProgress(Onboarding),
    Completed,
}

impl DatabaseSingleItem for OnboardingStatus {
    fn item_key() -> &'static str {
        "gui_onboarding_status"
    }
}
impl OnboardingStatus {
    // As clone may be "expensive", I don't want to clone by mistake else-where
    // But I still need to clone privately here because of the cache
    // and ownership constraint on tokio::tasks
    fn private_clone(&self) -> Self {
        match self {
            Self::Pending => Self::Pending,
            Self::InProgress(onboarding) => Self::InProgress(onboarding.clone()),
            Self::Completed => Self::Completed,
        }
    }
}

pub static ONBOARDING_STATUS: GlobalSignal<OnboardingStatus> =
    Signal::global(|| OnboardingStatus::default());

/// Commands for the onboarding provider service
#[derive(Debug)]
pub(super) enum OnboardingCommand {
    /// Internal trigger a save into the DB
    Persist { status: OnboardingStatus },
    /// Internal trigger a refresh from the DB
    Refresh,
}

/// Onboarding service coroutine
pub(super) fn use_onboarding_service(
    event_bus: EventBus,
    database_service: Coroutine<DatabaseCommand>,
) -> Coroutine<OnboardingCommand> {
    let service_handle = use_coroutine(
        move |mut rx: UnboundedReceiver<OnboardingCommand>| async move {
            log::info!("onboarding_service (coroutine) - start");

            let mut cached_value = load_status(database_service).await;
            *ONBOARDING_STATUS.write() = cached_value.private_clone();

            // Flag to trash the first persist command that will come from the initial run of the "use_effect"
            let mut discard_next_persit = true;

            while let Some(cmd) = rx.next().await {
                log::debug!("onboarding_service (coroutine) - Processing command {cmd:?}...");

                match cmd {
                    OnboardingCommand::Persist { status } => {
                        if discard_next_persit {
                            log::debug!(
                                "onboarding_service (coroutine) - Ignoring first Persist cmd."
                            );
                            discard_next_persit = false;
                        }
                        if status != cached_value {
                            log::debug!(
                                "onboarding_service (coroutine) - Status changed, persisting..."
                            );
                            match save_status(database_service, status).await {
                                Ok(status) => cached_value = status,
                                Err(msg) => {
                                    log::error!("{msg}");
                                    alert_error(msg);
                                }
                            };
                        } else {
                            log::debug!("onboarding_service (coroutine) - Ignoring Persist cmd: already in database.");
                        }
                    }
                    OnboardingCommand::Refresh => {
                        cached_value = load_status(database_service).await;
                        *ONBOARDING_STATUS.write() = cached_value.private_clone()
                    }
                }

                log::debug!("onboarding_service (coroutine) - Command processed");
            }
        },
    );
    subscribe_event(event_bus, move |_event: DatabaseReloadEvent| {
        service_handle.send(OnboardingCommand::Refresh);
    });
    use_effect(move || {
        {
            let guard = ONBOARDING_STATUS.peek();
            if let OnboardingStatus::InProgress(ref ob) = *guard {
                if ob.finished() {
                    drop(guard);
                    *ONBOARDING_STATUS.write() = OnboardingStatus::Completed;
                }
            }
        }
        service_handle.send(OnboardingCommand::Persist {
            status: ONBOARDING_STATUS.read().private_clone(),
        });
    });
    service_handle
}

async fn load_status(database_service: Coroutine<DatabaseCommand>) -> OnboardingStatus {
    let database = super::helpers::get_database(database_service).await;

    match database
        .blocking_operation(|db| OnboardingStatus::load(&db))
        .await
    {
        Ok(status) => status,
        Err(e) => {
            match e {
                btc_heritage_wallet::errors::DbError::KeyDoesNotExists(_) => (),
                _ => {
                    let msg = format!("Could not load Onboarding Status from database: {e}");
                    log::error!("{msg}");
                    alert_error(msg);
                }
            };
            OnboardingStatus::default()
        }
    }
}

async fn save_status(
    database_service: Coroutine<DatabaseCommand>,
    status: OnboardingStatus,
) -> Result<OnboardingStatus, String> {
    let database = super::helpers::get_database(database_service).await;
    database
        .blocking_operation(move |mut db| status.save(&mut db).map(|_| status))
        .await
        .map_err(|e| e.to_string())
}
