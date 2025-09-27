mod blockchain;
mod clipboard;
mod database;
mod event_bus;
mod helpers;
mod ledger;
mod onboarding;
mod service;
mod theme;

pub fn use_init_services() {
    log::debug!("init_services - start");
    let event_bus_service = event_bus::use_event_bus_service();
    let database_service = database::use_database_service(event_bus_service);
    let _ = service::use_service_client_service(event_bus_service, database_service);
    let _ = blockchain::use_blockchain_provider_service(event_bus_service, database_service);
    let _ = onboarding::use_onboarding_service(event_bus_service, database_service);
    let _ = theme::use_theme_service(event_bus_service, database_service);
    let _ = clipboard::use_clipboard_service();
    ledger::use_ledger_status_service();
    log::debug!("init_services - finished");
}

pub mod prelude {
    pub use super::blockchain::BlockchainProviderStatus;
    pub use super::database::ApplicationConfig;
    pub use super::ledger::LedgerStatus;
    pub use super::onboarding::OnboardingStatus;
    pub use super::service::ServiceStatus;
    pub use super::theme::Theme;

    pub mod state_management {
        pub use super::super::blockchain::BLOCKCHAIN_PROVIDER_STATUS;
        pub use super::super::database::APPLICATION_CONFIG;
        pub use super::super::helpers::*;
        pub use super::super::ledger::LEDGER_STATUS;
        pub use super::super::onboarding::ONBOARDING_STATUS;
        pub use super::super::service::SERVICE_STATUS;
        pub use super::super::theme::THEME;
    }
}
