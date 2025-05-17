use dioxus::prelude::*;

use btc_heritage_wallet::{
    bitcoin::SignedAmount, btc_heritage::bdk_types::BlockTime,
    heritage_service_api_client::TransactionSummary, online_wallet::WalletStatus,
};

use crate::{
    components::{balance::UIBtcAmount, misc::Tooltip, timestamp::UITimestamp},
    loaded::prelude::*,
    utils::{amount_to_signed_string, ArcStr, ArcType},
};

#[derive(Debug, Clone, PartialEq)]
struct UIBlockTime(Option<BlockTime>);
impl LoadedElement for UIBlockTime {
    type Loader = SkeletonLoader;
    #[inline(always)]
    fn element<M: LoadedComponentInputMapper>(self, m: M) -> Element {
        let tooltip_text = if let Some(BlockTime { height, .. }) = &self.0 {
            ArcStr::from(format!("Included in block #{height}"))
        } else {
            ArcStr::from("Not included yet")
        };

        let timestamp = match self.0 {
            Some(BlockTime { timestamp, .. }) => UITimestamp::new_full(timestamp),
            None => UITimestamp::none(),
        };

        rsx! {
            Tooltip { tooltip_text,
                LoadedComponent { input: m.map(timestamp) }
            }
        }
    }
    fn place_holder() -> Self {
        Self(None)
    }
}

#[derive(Debug, Clone, PartialEq)]
struct UITransactionsHistoryRow {
    confirmation_time: UIBlockTime,
    txid: ArcStr,
    amount_tt: ArcStr,
    amount: UIBtcAmount,
    balance_after: Option<UIBtcAmount>,
}
impl LoadedElement for UITransactionsHistoryRow {
    type Loader = TransparentLoader;
    #[inline(always)]
    fn element<M: LoadedComponentInputMapper>(self, m: M) -> Element {
        rsx! {
            tr {
                td {
                    LoadedComponent { input: m.map(self.confirmation_time) }
                }
                td {
                    LoadedComponent { input: m.map(self.txid) }
                }
                td { class: "font-bold",
                    Tooltip { tooltip_text: self.amount_tt,
                        LoadedComponent { input: m.map(self.amount) }
                    }
                }
                td { class: "font-semibold",
                    LoadedComponent::<UIBtcAmount> { input: self.balance_after.into() }
                }
            }
        }
    }
    fn place_holder() -> Self {
        Self {
            confirmation_time: UIBlockTime::place_holder(),
            txid: ArcStr::place_holder(),
            amount_tt: ArcStr::place_holder(),
            amount: UIBtcAmount::place_holder(),
            balance_after: Some(UIBtcAmount::place_holder()),
        }
    }
}

#[component]
pub(super) fn TransactionsHistory() -> Element {
    log::debug!("TransactionsHistory Rendered");

    let wallet_status = use_context::<Resource<Result<WalletStatus, String>>>();
    let wallet_transactions = use_context::<Resource<ArcType<[TransactionSummary]>>>();

    let transaction_history_items = use_memo(move || {
        log::debug!("use_memo_transaction_history_items - start compute");
        let transaction_history_items = wallet_transactions.cloned().map(|wtx| {
            let final_balance = match &*wallet_status.read() {
                Some(Ok(ws)) => Some(SignedAmount::from_sat(
                    ws.balance.total_balance().get_total() as i64,
                )),
                _ => None,
            };
            wtx.iter()
                .scan(final_balance, |balance, tx_sum| {
                    let confirmation_time = UIBlockTime(tx_sum.confirmation_time.clone());
                    let txid = ArcStr::from(tx_sum.txid.to_string());
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

                    let amount_tt = ArcStr::from(format!(
                        "Sent: {} | Received: {} | Fee: {} ({} sat/vB)",
                        &amount_to_signed_string(sent)[1..],
                        &amount_to_signed_string(received)[1..],
                        &amount_to_signed_string(fee)[1..],
                        fee_rate.to_sat_per_vb_floor()
                    ));

                    let balance_after = balance.map(|b| UIBtcAmount::new(Some(b), false));

                    // Update the balance (if any)
                    let amount = received - sent;
                    balance.as_mut().map(|balance| *balance -= amount);

                    let amount = UIBtcAmount::new(Some(amount), true);

                    Some(UITransactionsHistoryRow {
                        confirmation_time,
                        txid,
                        amount_tt,
                        amount,
                        balance_after,
                    })
                })
                .collect::<ArcType<[_]>>()
        });
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
                    LoadedComponent::<ArcType<[UITransactionsHistoryRow]>> { input: transaction_history_items.into() }
                }
            }
        
        }
    }
}
