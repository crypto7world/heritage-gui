use dioxus::prelude::*;

use std::{str::FromStr, sync::Arc};

use btc_heritage_wallet::{
    bitcoin::{self, FeeRate, SignedAmount},
    btc_heritage::bdk_types::BlockTime,
    heritage_service_api_client::TransactionSummary,
    online_wallet::WalletStatus,
};

use crate::{
    components::{
        misc::{Date, DisplayTimestamp, Tooltip},
        wallet::{BtcAmount, DisplayBtcAmount},
    },
    utils::{amount_to_signed_string, LoadedElement, PlaceHolder, RcStr},
};

#[component]
pub(super) fn TransactionsHistory() -> Element {
    log::debug!("TransactionsHistory Rendered");

    let wallet_status = use_context::<Resource<Option<WalletStatus>>>();
    let wallet_transactions = use_context::<Resource<Arc<[TransactionSummary]>>>();

    let transaction_history_items = use_memo(move || {
        log::debug!("use_memo_transaction_history_items - start compute");
        let transaction_history_items = if let Some(wallet_transactions) =
            wallet_transactions.cloned()
        {
            let final_balance = match &*wallet_status.read() {
                Some(Some(ws)) => Some(SignedAmount::from_sat(
                    ws.balance.total_balance().get_total() as i64,
                )),
                _ => None,
            };
            wallet_transactions
                .iter()
                .scan(final_balance, |balance, tx_sum| {
                    let confirmation_time = LoadedElement::Loaded(tx_sum.confirmation_time.clone());
                    let txid = LoadedElement::Loaded(tx_sum.txid);
                    let sent = tx_sum
                        .owned_inputs
                        .iter()
                        .fold(SignedAmount::ZERO, |acc, tio| {
                            acc + tio
                                .amount
                                .to_signed()
                                .expect("TX IO cannot be bigger than MAX_MONEY")
                        });

                    let received =
                        tx_sum
                            .owned_outputs
                            .iter()
                            .fold(SignedAmount::ZERO, |acc, tio| {
                                acc + tio
                                    .amount
                                    .to_signed()
                                    .expect("TX IO cannot be bigger than MAX_MONEY")
                            });

                    let fee = tx_sum
                        .fee
                        .to_signed()
                        .expect("Fee cannot be bigger than MAX_MONEY");
                    let fee_rate = tx_sum.fee_rate;
                    let balance_after = LoadedElement::Loaded((*balance).into());

                    // Update the balance (if any)
                    let amount = received - sent;
                    balance.as_mut().map(|balance| *balance -= amount);

                    let amount = LoadedElement::Loaded(amount.into());

                    Some(TransactionsHistoryItemProps {
                        confirmation_time,
                        txid,
                        sent,
                        received,
                        fee,
                        fee_rate,
                        amount,
                        balance_after,
                    })
                })
                .collect()
        } else {
            vec![
                TransactionsHistoryItemProps {
                    confirmation_time: LoadedElement::Loading,
                    txid: LoadedElement::Loading,
                    sent: SignedAmount::place_holder(),
                    received: SignedAmount::place_holder(),
                    fee: SignedAmount::place_holder(),
                    fee_rate: FeeRate::ZERO,
                    amount: LoadedElement::Loading,
                    balance_after: LoadedElement::Loading,
                };
                5
            ]
        };
        log::debug!("use_memo_transaction_history_items - finish compute");
        transaction_history_items
    });

    use_drop(|| log::debug!("TransactionsHistory Dropped"));

    rsx! {
        div { class: "overflow-x-auto rounded-box border border-base-content/5 bg-base-100 m-4",
            h2 { class: "text-h2 font-bold p-4", "Transaction History" }
            table { class: "table table-zebra",
                thead {
                    tr {
                        th { "Confirmation time" }
                        th { "Transaction ID" }
                        th { "Amount" }
                        th { "Balance" }
                    }
                }
                tbody {
                    for transaction_history_item in transaction_history_items() {
                        TransactionsHistoryItem { ..transaction_history_item }
                    }
                }
            }
        
        }
    }
}

#[component]
fn TransactionsHistoryItem(
    confirmation_time: LoadedElement<Option<BlockTime>>,
    txid: LoadedElement<bitcoin::Txid>,
    sent: SignedAmount,
    received: SignedAmount,
    fee: SignedAmount,
    fee_rate: FeeRate,
    amount: LoadedElement<DisplayBtcAmount>,
    balance_after: LoadedElement<DisplayBtcAmount>,
) -> Element {
    let amount_tt = format!(
        "Sent: {} | Received: {} | Fee: {} ({} sat/vB)",
        &amount_to_signed_string(sent)[1..],
        &amount_to_signed_string(received)[1..],
        &amount_to_signed_string(fee)[1..],
        fee_rate.to_sat_per_vb_floor()
    )
    .into();

    rsx! {
        tr {
            td {
                ConfirmationTime { confirmation_time }
            }
            td {
                Txid { txid }
            }
            td { class: "font-bold",
                Tooltip { tooltip_text: amount_tt,
                    BtcAmount { amount, diff_style: true }
                }
            }
            td { class: "font-semibold",
                BtcAmount { amount: balance_after }
            }
        
        }
    }
}

#[component]
fn ConfirmationTime(confirmation_time: LoadedElement<Option<BlockTime>>) -> Element {
    let tooltip_text =
        if let LoadedElement::Loaded(Some(BlockTime { height, .. })) = &confirmation_time {
            RcStr::from(format!("Included in block #{height}"))
        } else {
            RcStr::from_str("Not included yet").unwrap()
        };

    let timestamp = confirmation_time.map(|opt| match opt {
        Some(BlockTime { timestamp, .. }) => DisplayTimestamp::Ts(timestamp),
        None => DisplayTimestamp::None,
    });

    rsx! {
        Tooltip { tooltip_text,
            Date { timestamp }
        }
    }
}

#[component]
fn Txid(txid: LoadedElement<bitcoin::Txid>) -> Element {
    let (is_place_holder, txid) = txid.extract();

    let txid_s = txid.to_string();

    rsx! {
        span {
            class: "text-nowrap inline-block uppercase font-mono",
            class: if is_place_holder { "skeleton text-transparent" },
            {txid_s}
        }
    }
}
