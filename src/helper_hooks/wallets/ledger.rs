use crate::prelude::*;

use std::collections::BTreeMap;

use btc_heritage_wallet::{
    btc_heritage::{AccountXPubId, HeritageWalletBackup},
    AnyKeyProvider, LedgerPolicy, Wallet,
};

use crate::utils::log_error;

pub fn use_memo_ledger_registered_policies(
    wallet: AsyncSignal<Wallet>,
) -> Memo<Option<BTreeMap<AccountXPubId, LedgerPolicy>>> {
    use_memo(move || {
        log::debug!("use_memo_ledger_registered_policies - start compute");

        let ledger_registered_policies = wallet
            .lmap(|wallet| {
                if let AnyKeyProvider::Ledger(ledger_key) = wallet.key_provider() {
                    Some(
                        ledger_key
                            .list_registered_policies()
                            .into_iter()
                            .map(|(index, policy, _, _)| (index, policy))
                            .collect(),
                    )
                } else {
                    None
                }
            })
            .flatten();

        log::debug!("use_memo_ledger_registered_policies - finish compute");
        ledger_registered_policies
    })
}

pub fn use_memo_wallet_uses_ledger(wallet: AsyncSignal<Wallet>) -> Memo<bool> {
    use_memo(move || {
        log::debug!("use_memo_wallet_uses_ledger - start compute");

        let uses_ledger = wallet
            .lmap(|wallet| matches!(wallet.key_provider(), AnyKeyProvider::Ledger(_)))
            .is_some_and(|b| b);

        log::debug!("use_memo_wallet_uses_ledger - finish compute");
        uses_ledger
    })
}

pub fn use_memo_ledger_unregistered_policies(
    ledger_registered_policies: Memo<Option<BTreeMap<AccountXPubId, LedgerPolicy>>>,
    backup: FMemo<HeritageWalletBackup>,
) -> FMemo<BTreeMap<AccountXPubId, LedgerPolicy>> {
    use_memo(move || {
        log::debug!("use_memo_ledger_unregistered_policies - start compute");

        let ledger_unregistered_policies = match (
            ledger_registered_policies.read().as_ref(),
            backup.read().as_ref(),
        ) {
            (Some(ledger_registered_policies), Some(backup)) => Some(
                backup
                    .as_ref()
                    .map(|bkp| {
                        bkp.iter()
                            .filter_map(|swbkp| {
                                LedgerPolicy::try_from(swbkp)
                                    .map_err(log_error)
                                    .ok()
                                    .map(|p| (p.get_account_id(), p))
                                    .filter(|(id, policy)| {
                                        !ledger_registered_policies.get(id).is_some_and(
                                            |existing_policy| existing_policy == policy,
                                        )
                                    })
                            })
                            .collect()
                    })
                    .map_err(Clone::clone),
            ),
            _ => None,
        };

        log::debug!("use_memo_ledger_unregistered_policies - finish compute");
        ledger_unregistered_policies
    })
}
