use crate::prelude::*;

use btc_heritage_wallet::heritage_service_api_client::TransactionSummary;

use crate::{
    components::{
        balance::UIBtcAmount, misc::UITxId, timestamp::UITimestamp, transaction::UITxDetails,
    },
    utils::{CCStr, CheapClone},
};

/// Expandable row for transaction history display
#[derive(Debug, Clone, PartialEq)]
struct UITransactionsHistoryExpandableRow {
    txid: UITxId,
    confirmation_time: UITimestamp,
    block_height: CCStr,
    balance_change: UIBtcAmount,
    balance_after: UIBtcAmount,
    details: UITxDetails,
}

impl LoadedElement for UITransactionsHistoryExpandableRow {
    type Loader = TransparentLoader;

    #[inline(always)]
    fn element<M: LoadedComponentInputMapper>(self, m: M) -> Element {
        rsx! {
            div { class: "collapse border-b border-base-content/5 rounded-none last:border-b-0 odd:bg-base-200",
                input { r#type: "checkbox", class: "!py-3 !min-h-0" }
                div { class: "collapse-title min-h-0 p-0",
                    div { class: "grid grid-cols-6 gap-1 px-4 py-3 items-center cursor-pointer",

                        // Confirmation time
                        div { class: "overflow-clip",
                            LoadedComponent { input: m.map(self.confirmation_time) }
                        }
                        // Transaction ID (shortened, 3 cols)
                        div { class: "col-span-3 font-mono text-base overflow-clip",
                            LoadedComponent { input: m.map(self.txid.clone()) }
                        }
                        // Amount
                        div { class: "font-bold text-center",
                            LoadedComponent { input: m.map(self.balance_change) }
                        }
                        // Balance after
                        div { class: "font-semibold",
                            LoadedComponent { input: m.map(self.balance_after) }
                        }
                    }
                }
                div { class: "collapse-content p-0",
                    div { class: "overflow-x-auto w-[calc(100vw-var(--spacing)*16)] bg-base-300",
                        div { class: "flex flex-col gap-2 p-6 bg-base-300 border-t border-base-content/10 w-max",
                            div { class: "flex flex-row gap-8",
                                div {
                                    h4 { class: "font-semibold text-base text-primary",
                                        "Confirmation Time"
                                    }
                                    div { class: "text-base",
                                        LoadedComponent { input: m.map(self.confirmation_time) }
                                    }
                                }
                                div {
                                    h4 { class: "font-semibold text-base text-primary",
                                        "Block Height"
                                    }
                                    div { class: "text-base",
                                        LoadedComponent { input: m.map(self.block_height) }
                                    }
                                }
                                div {
                                    h4 { class: "font-semibold text-base text-primary",
                                        "Transaction ID"
                                    }
                                    div { class: "text-base font-mono",
                                        LoadedComponent { input: m.map(self.txid) }
                                    }
                                }
                            }
                            h4 { class: "font-semibold text-base text-primary", "Transaction Details" }
                            LoadedComponent { input: m.map(self.details) }
                        }
                    }
                }
            }
        }
    }

    fn place_holder() -> Self {
        Self {
            txid: UITxId::place_holder(),
            confirmation_time: UITimestamp::place_holder(),
            block_height: CCStr::place_holder(),
            balance_change: UIBtcAmount::place_holder(),
            balance_after: UIBtcAmount::place_holder(),
            details: UITxDetails::place_holder(),
        }
    }
}
impl LoadedSuccessConversionMarker
    for TypeCouple<TransactionHistoryItem, UITransactionsHistoryExpandableRow>
{
}
impl FromRef<TransactionHistoryItem> for UITransactionsHistoryExpandableRow {
    fn from_ref(thi: &TransactionHistoryItem) -> Self {
        let txid = UITxId::from(thi.txid.to_string());
        let (confirmation_time, block_height) = thi
            .confirmation_time
            .as_ref()
            .map(|bt| {
                (
                    UITimestamp::new_full(bt.timestamp),
                    CCStr::from(format!("{}", bt.height)),
                )
            })
            .unwrap_or_else(|| (UITimestamp::none(), CCStr::from("-")));
        let balance_change = UIBtcAmount::new(Some(thi.balance_change), true);
        let balance_after = UIBtcAmount::from(thi.balance_after);

        Self {
            txid,
            confirmation_time,
            block_height,
            balance_change,
            balance_after,
            details: UITxDetails::from_ref(thi),
        }
    }
}

/// Transaction history component displaying transactions in an expandable grid format
#[component]
pub(super) fn TransactionsHistory() -> Element {
    log::debug!("TransactionsHistory Rendered");

    let wallet_transactions = use_context::<FResource<CheapClone<[TransactionSummary]>>>();

    let transaction_history_items =
        helper_hooks::use_memo_transaction_history_items(wallet_transactions);

    use_drop(|| log::debug!("TransactionsHistory Dropped"));

    rsx! {
        div { class: "max-h-[calc(100vh-var(--spacing)*32)] overflow-y-auto rounded-box border border-base-content/5 shadow-md bg-base-100 my-4",
            // Title
            h2 { class: "text-2xl font-bold p-4", "Transactions History" }

            // Header row
            div { class: "sticky top-0 z-10 bg-base-100 grid grid-cols-6 gap-1 items-center p-4 font-semibold text-base text-(--color-base-content)/60 border-b border-base-content/10",
                div { "Confirmation Time" }
                div { class: "col-span-3", "Transaction ID" }
                div { class: "text-center", "Amount" }
                div { "Balance" }
            }

            // Expandable address rows
            LoadedComponent::<CheapClone<[UITransactionsHistoryExpandableRow]>> { input: transaction_history_items.into() }
        }
    }
}
