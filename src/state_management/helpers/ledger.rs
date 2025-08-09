use btc_heritage_wallet::heritage_service_api_client::Fingerprint;

use super::*;

pub fn refresh_ledger_status() {
    log::debug!("refresh_ledger_status - start");
    spawn(async move {
        let new_status = Some(LedgerStatus::current().await);
        if LEDGER_STATUS() != new_status {
            *LEDGER_STATUS.write() = new_status;
        }
    });
    log::debug!("refresh_ledger_status - finished");
}

/// Checks if the ledger is ready and returns its fingerprint if available.
///
/// This function checks the current ledger status and returns the fingerprint
/// if the ledger is in a ready state.
///
/// # Returns
///
/// Returns `Some(Fingerprint)` if the ledger is ready, containing the ledger's
/// fingerprint. Returns `None` if the ledger is not ready or not available.
pub fn ledger_is_ready() -> Option<Fingerprint> {
    if let Some(LedgerStatus::Ready(fg)) = LEDGER_STATUS() {
        Some(fg)
    } else {
        None
    }
}
