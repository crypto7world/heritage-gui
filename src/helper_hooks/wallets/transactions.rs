use crate::prelude::*;

use std::collections::HashMap;

use btc_heritage_wallet::{
    bitcoin::{Address, Amount, FeeRate, OutPoint, SignedAmount, Txid},
    btc_heritage::{bdk_types::BlockTime, heritage_wallet::TransactionSummaryIOTotals},
    heritage_service_api_client::{TransactionSummary, TransactionSummaryOwnedIO},
    DatabaseItem, OnlineWallet, Wallet,
};

use crate::utils::{amount_to_signed, CCStr, CheapClone};

pub fn use_resource_wallet_transactions(
    wallet: AsyncSignal<Wallet>,
) -> FResource<CheapClone<[TransactionSummary]>> {
    use_resource(move || async move {
        log::debug!("use_resource_wallet_transactions - start");

        super::subscribe_service_status_if_service_wallet(&wallet);

        let wallet_txs = wallet
            .with(async |wallet| {
                let wallet_name = wallet.name().to_owned();
                wallet
                    .list_transactions()
                    .await
                    .map_err(|e| {
                        let error = format!(
                            "Error retrieving the wallet transactions of wallet {}: {e}",
                            wallet_name
                        );
                        log::error!("{error}");
                        CCStr::from(error)
                    })
                    .map(Into::into)
            })
            .await;
        log::debug!("use_resource_wallet_transactions - loaded");

        wallet_txs
    })
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct TxIOInner {
    pub outpoint: OutPoint,
    pub amount: Amount,
}
impl From<&TransactionSummaryOwnedIO> for TxIOInner {
    fn from(io: &TransactionSummaryOwnedIO) -> Self {
        Self {
            outpoint: io.outpoint,
            amount: io.amount,
        }
    }
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TxIO {
    Incoming(TxIOInner),
    Outgoing(TxIOInner),
}
impl core::ops::Deref for TxIO {
    type Target = TxIOInner;

    fn deref(&self) -> &Self::Target {
        match self {
            TxIO::Incoming(tx_ioinner) => tx_ioinner,
            TxIO::Outgoing(tx_ioinner) => tx_ioinner,
        }
    }
}
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TransactionStats {
    pub id: Txid,
    pub block_time: Option<BlockTime>,
    pub in_out: CheapClone<[TxIO]>,
}
impl TransactionStats {
    fn new_by_address(tx_sum: &TransactionSummary) -> HashMap<Address, Self> {
        let id = tx_sum.txid;
        let incoming = tx_sum
            .owned_outputs
            .iter()
            .map(|io| ((*io.address).clone(), TxIO::Incoming(TxIOInner::from(io))));
        let outgoing = tx_sum
            .owned_inputs
            .iter()
            .map(|io| ((*io.address).clone(), TxIO::Outgoing(TxIOInner::from(io))));
        incoming
            .chain(outgoing)
            .fold(HashMap::new(), |mut h, (addr, txio)| {
                h.entry(addr)
                    .or_insert_with(|| Vec::with_capacity(2))
                    .push(txio);
                h
            })
            .into_iter()
            .map(|(addr, txios)| {
                (
                    addr,
                    Self {
                        id,
                        block_time: tx_sum.confirmation_time.clone(),
                        in_out: txios.into(),
                    },
                )
            })
            .collect()
    }
}

pub fn use_memo_tx_stats_by_address(
    wallet_transactions: FResource<CheapClone<[TransactionSummary]>>,
) -> FMemo<HashMap<Address, CheapClone<[TransactionStats]>>> {
    use_memo(move || {
        log::debug!("use_memo_tx_stats_by_address - start compute");

        let tx_stats_by_address = wallet_transactions.lrmap(|wallet_transactions| {
            wallet_transactions
                .iter()
                .map(TransactionStats::new_by_address)
                .flatten()
                .fold(HashMap::new(), |mut h, (addr, tx_status)| {
                    h.entry(addr).or_insert_with(|| Vec::new()).push(tx_status);
                    h
                })
                .into_iter()
                .map(|(addr, tx_stats)| (addr, tx_stats.into()))
                .collect()
        });

        log::debug!("use_memo_tx_stats_by_address - finish compute");
        tx_stats_by_address
    })
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TransactionHistoryItemOwnedIO {
    pub address: CCStr,
    pub amount: Amount,
}
impl From<&TransactionSummaryOwnedIO> for TransactionHistoryItemOwnedIO {
    fn from(txsum_oio: &TransactionSummaryOwnedIO) -> Self {
        Self {
            address: CCStr::from(txsum_oio.address.to_string()),
            amount: txsum_oio.amount,
        }
    }
}
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TransactionHistoryItem {
    pub txid: Txid,
    pub confirmation_time: Option<BlockTime>,
    pub balance_spent: Amount,
    pub balance_received: Amount,
    pub balance_change: SignedAmount,
    pub balance_after: Amount,
    pub inputs_totals: TransactionSummaryIOTotals,
    pub owned_inputs: CheapClone<[TransactionHistoryItemOwnedIO]>,
    pub outputs_totals: TransactionSummaryIOTotals,
    pub owned_outputs: CheapClone<[TransactionHistoryItemOwnedIO]>,
    pub fee: Amount,
    pub fee_rate: FeeRate,
}

pub fn use_memo_transaction_history_items(
    wallet_transactions: FResource<CheapClone<[TransactionSummary]>>,
) -> FMemo<CheapClone<[TransactionHistoryItem]>> {
    use_memo(move || {
        log::debug!("use_memo_transaction_history_items - start compute");

        let transaction_history_items = wallet_transactions.lrmap(|wtx| {
            let transaction_history_items = wtx
                .iter()
                .rev()
                .scan(Amount::ZERO, |current_balance, tx_sum| {
                    let txid = tx_sum.txid;
                    let confirmation_time = tx_sum.confirmation_time.clone();
                    let balance_spent = tx_sum.owned_inputs.iter().map(|toio| toio.amount).sum();
                    let balance_received =
                        tx_sum.owned_outputs.iter().map(|toio| toio.amount).sum();
                    let balance_change =
                        amount_to_signed(balance_received) - amount_to_signed(balance_spent);

                    // Update the current_balance for the next iteration (if any)
                    *current_balance = *current_balance + balance_received - balance_spent;
                    let balance_after = current_balance.clone();

                    let inputs_totals = tx_sum.inputs_totals;
                    let outputs_totals = tx_sum.outputs_totals;
                    let fee = tx_sum.fee;
                    let fee_rate = tx_sum.fee_rate;
                    let owned_inputs = tx_sum
                        .owned_inputs
                        .iter()
                        .map(TransactionHistoryItemOwnedIO::from)
                        .collect();
                    let owned_outputs = tx_sum
                        .owned_outputs
                        .iter()
                        .map(TransactionHistoryItemOwnedIO::from)
                        .collect();

                    Some(TransactionHistoryItem {
                        txid,
                        confirmation_time,
                        balance_spent,
                        balance_received,
                        balance_change,
                        balance_after,
                        inputs_totals,
                        owned_inputs,
                        outputs_totals,
                        owned_outputs,
                        fee,
                        fee_rate,
                    })
                })
                .collect::<Vec<_>>();
            transaction_history_items.into_iter().rev().collect()
        });

        log::debug!("use_memo_transaction_history_items - finish compute");
        transaction_history_items
    })
}
