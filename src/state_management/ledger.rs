use dioxus::prelude::*;

use btc_heritage_wallet::{btc_heritage::utils::bitcoin_network, errors::Error};

/// Status of the Ledger hardware wallet
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LedgerStatus {
    /// Device ready
    Ready(btc_heritage_wallet::heritage_service_api_client::Fingerprint),
    /// Device on Bitcoin application for the wrong network
    WrongNetwork,
    /// Device on the wrong application
    WrongApp,
    /// No device detected
    NotReady,
}
impl LedgerStatus {
    pub(super) async fn current() -> Self {
        match btc_heritage_wallet::ledger_client().await {
            Some((ledger_client, fg)) => match ledger_client.network().await {
                Ok(ledger_network) => {
                    if ledger_network == bitcoin_network::get() {
                        LedgerStatus::Ready(fg)
                    } else {
                        log::warn!("The Ledger Bitcoin application is opened on the wrong network");
                        LedgerStatus::WrongNetwork
                    }
                }
                Err(Error::WrongLedgerApplication) => {
                    log::warn!("The Ledger is not opened on the Bitcoin application");
                    LedgerStatus::WrongApp
                }
                Err(e) => {
                    log::error!("{e}");
                    LedgerStatus::NotReady
                }
            },
            None => LedgerStatus::NotReady,
        }
    }
}

pub static LEDGER_STATUS: GlobalSignal<Option<LedgerStatus>> = Signal::global(|| None);

pub(super) fn use_ledger_status_service() {
    use_future(async move || loop {
        log::debug!("ledger_status_service: Refreshing...");
        let new_status = Some(LedgerStatus::current().await);

        if LEDGER_STATUS() != new_status {
            *LEDGER_STATUS.write() = new_status;
        }
        tokio::time::sleep(std::time::Duration::from_secs(10)).await
    });
}
