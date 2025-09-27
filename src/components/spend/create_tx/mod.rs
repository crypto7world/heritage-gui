use crate::prelude::*;

use std::collections::{HashMap, HashSet};

use btc_heritage_wallet::{
    bitcoin::{address::NetworkUnchecked, Address, Amount, Denomination, OutPoint},
    btc_heritage::utils::bitcoin_network,
    heritage_service_api_client::{
        NewTx, NewTxDrainTo, NewTxFeePolicy, NewTxRecipient, NewTxSpendingConfig,
        NewTxUtxoSelection, TransactionSummary,
    },
    online_wallet::WalletStatus,
    HeirWallet, HeritageProvider, OnlineWallet, Wallet,
};

use crate::{
    components::{
        balance::UIBtcAmount,
        svg::{BankPlus, Close, DrawSvg, Plus, SvgSize::Size3},
        transaction::{UITxDetails, UIUtxo},
    },
    utils::{denomination_for_amount, feerate_sat_per_vb, CCStr, CheapClone},
};

use super::{ExportEncodedTransaction, PsbtToSign, SignedPsbt, SpendStage};

mod heir;
mod owner;

pub(super) use heir::CreateHeirTx;
pub(super) use owner::CreateOwnerTx;

#[derive(Debug, Clone, Default, Copy, PartialEq)]
enum RecipientAmountUnit {
    Sat,
    #[default]
    MilliBtc,
    Btc,
}
impl core::fmt::Display for RecipientAmountUnit {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            Self::Sat => "sat",
            Self::MilliBtc => "mBTC",
            Self::Btc => "BTC",
        })
    }
}
impl core::str::FromStr for RecipientAmountUnit {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "sat" => Ok(RecipientAmountUnit::Sat),
            "mBTC" => Ok(RecipientAmountUnit::MilliBtc),
            "BTC" => Ok(RecipientAmountUnit::Btc),
            _ => Err(()),
        }
    }
}
/// State for a single recipient
#[derive(Debug, Clone, PartialEq)]
struct RecipientState {
    uuid: uuid::Uuid,
    address: String,
    amount: String,
    unit: RecipientAmountUnit,
    drain_to: bool,
}
impl Default for RecipientState {
    fn default() -> Self {
        Self {
            uuid: uuid::Uuid::new_v4(),
            address: String::new(),
            amount: String::new(),
            unit: RecipientAmountUnit::default(),
            drain_to: false,
        }
    }
}

/// Advanced settings components
#[component]
fn FeePolicyInput(
    fee_policy: Signal<Option<NewTxFeePolicy>>,
    default_fee_rate: ReadOnlySignal<f32>,
) -> Element {
    let mut field_label = use_signal(|| "sat");
    let mut field_placeholder = use_signal(|| "Fee in satoshis");
    let field_explanation = use_memo(move || match fee_policy() {
        Some(NewTxFeePolicy::Absolute { .. }) => {
            "The TX fee will be exactly the amount you specify"
        }
        Some(NewTxFeePolicy::Rate { .. }) => {
            "The TX fee will depend on the TX size, \
        with a rate that you specify"
        }
        None => {
            "The TX fee will depend on the TX size, \
        with the rate retrieved from the Bitcoin blockchain at the last sync"
        }
    });
    let field_disabled = use_memo(move || matches!(fee_policy(), None));
    let field_value = use_memo(move || match fee_policy() {
        Some(NewTxFeePolicy::Absolute { amount }) => amount.to_string(),
        Some(NewTxFeePolicy::Rate { rate }) => rate.to_string(),
        None => default_fee_rate().to_string(),
    });

    rsx! {
        // Fee policy
        fieldset { class: "fieldset",
            legend { class: "fieldset-legend", "Fee Policy" }
            div { class: "flex flex-col gap-4",
                div { class: "flex flex-row gap-6",
                    label { class: "label",
                        input {
                            r#type: "radio",
                            name: "fee_policy",
                            class: "radio",
                            checked: matches!(fee_policy(), None),
                            onchange: move |_| {
                                *fee_policy.write() = None;
                                *field_label.write() = "sat/vB";
                            },
                        }
                        "Automatic"
                    }
                    label { class: "label",
                        input {
                            r#type: "radio",
                            name: "fee_policy",
                            class: "radio",
                            checked: matches!(fee_policy(), Some(NewTxFeePolicy::Rate { .. })),
                            onchange: move |_| {
                                *fee_policy.write() = Some(NewTxFeePolicy::Rate {
                                    rate: default_fee_rate(),
                                });
                                *field_label.write() = "sat/vB";
                                *field_placeholder.write() = "Fee rate (sats/vB)";
                            },
                        }
                        "Fee Rate"
                    }
                    label { class: "label",
                        input {
                            r#type: "radio",
                            name: "fee_policy",
                            class: "radio",
                            checked: matches!(fee_policy(), Some(NewTxFeePolicy::Absolute { .. })),
                            onchange: move |_| {
                                *fee_policy.write() = Some(NewTxFeePolicy::Absolute {
                                    amount: 1,
                                });
                                *field_label.write() = "sat";
                                *field_placeholder.write() = "Fee in satoshis";
                            },
                        }
                        "Fee Amount"
                    }
                }
                label { class: "input input-sm w-fit",
                    input {
                        r#type: "number",
                        class: "w-28",
                        disabled: field_disabled(),
                        placeholder: field_placeholder(),
                        value: field_value(),
                        oninput: move |evt| {
                            match &mut *fee_policy.write() {
                                Some(NewTxFeePolicy::Absolute { amount }) => {
                                    *amount = evt.value().parse().unwrap_or(1);
                                }
                                Some(NewTxFeePolicy::Rate { rate }) => {
                                    *rate = evt.value().parse().unwrap_or(1.0);
                                }
                                None => todo!(),
                            }
                        },
                    }
                    span { class: "label", {field_label()} }
                }
            }
            div { class: "label", {field_explanation()} }
        }
    }
}

#[component]
fn UtxoSelection(utxo_selection: Signal<Option<NewTxUtxoSelection>>) -> Element {
    let utxos_with_info = use_context::<FMemo<CheapClone<[UtxoWithInfo]>>>();

    let mut selection_mode = use_signal(|| "auto");
    let field_explanation = use_memo(move || match selection_mode() {
        "auto" => {
            "UTXO will be picked automatically to match the amount spend. \
        Note that UTXO related to previous Heritage Configurations will all be picked"
        }
        "inc_exc" => "Same as auto, with respect to explicit inclusions/exclusions you specify",
        "manual" => "The TX will use only the UTXO you specify",
        _ => unreachable!("no other value is possible"),
    });

    let mut utxo_inc_exc_status: Signal<HashMap<OutPoint, bool>> = use_signal(|| HashMap::new());
    let mut utxo_selected_status: Signal<HashSet<OutPoint>> = use_signal(|| HashSet::new());

    use_effect(move || match selection_mode() {
        "auto" => {
            *utxo_selection.write() = None;
        }
        "inc_exc" => {
            let hashmap = &*utxo_inc_exc_status.read();
            let (inc, exc): (Vec<_>, Vec<_>) = hashmap.iter().partition(|(_, inc)| **inc);
            *utxo_selection.write() = Some(if exc.is_empty() {
                NewTxUtxoSelection::Include {
                    include: inc.into_iter().map(|(out, _)| *out).collect(),
                }
            } else if inc.is_empty() {
                NewTxUtxoSelection::Exclude {
                    exclude: exc.into_iter().map(|(out, _)| *out).collect(),
                }
            } else {
                NewTxUtxoSelection::IncludeExclude {
                    include: inc.into_iter().map(|(out, _)| *out).collect(),
                    exclude: exc.into_iter().map(|(out, _)| *out).collect(),
                }
            });
        }
        "manual" => {
            *utxo_selection.write() = Some(NewTxUtxoSelection::UseOnly {
                use_only: utxo_selected_status.read().iter().cloned().collect(),
            });
        }
        _ => unreachable!("no other value is possible"),
    });

    rsx! {
        // UTXO selection placeholder
        fieldset { class: "fieldset",
            legend { class: "fieldset-legend", "UTXO Selection" }
            div { class: "flex flex-row gap-6",
                label { class: "label",
                    input {
                        r#type: "radio",
                        name: "utxo_selection",
                        class: "radio",
                        checked: matches!(selection_mode(), "auto"),
                        onchange: move |_| { *selection_mode.write() = "auto" },
                    }
                    "Automatic"
                }
                label { class: "label",
                    input {
                        r#type: "radio",
                        name: "utxo_selection",
                        class: "radio",
                        checked: matches!(selection_mode(), "inc_exc"),
                        onchange: move |_| { *selection_mode.write() = "inc_exc" },
                    }
                    "Include/Exclude"
                }
                label { class: "label",
                    input {
                        r#type: "radio",
                        name: "utxo_selection",
                        class: "radio",
                        checked: matches!(selection_mode(), "manual"),
                        onchange: move |_| { *selection_mode.write() = "manual" },
                    }
                    "Manual selection"
                }
            }
            div { class: "label", {field_explanation()} }
            if !matches!(selection_mode(), "auto") {
                table { class: "table table-zebra",
                    thead {
                        tr {
                            th { class: "w-60",
                                match selection_mode() {
                                    "inc_exc" => "Constraints",
                                    "manual" => "Select",
                                    _ => unreachable!("no other value is possible"),
                                }
                            }
                            th { "UTXO" }
                        }
                    }
                    tbody {
                        if let Some(Ok(utxos_with_info)) = utxos_with_info() {
                            for utxo_with_info in utxos_with_info.iter() {
                                tr { key: "{utxo_with_info.outpoint}",
                                    td {
                                        match selection_mode() {
                                            "inc_exc" => rsx! {
                                                div { class: "flex flex-row gap-6",
                                                    label { class: "label",
                                                        input {
                                                            r#type: "checkbox",
                                                            name: "inc_exc",
                                                            class: "checkbox",
                                                            checked: utxo_inc_exc_status.read().get(&utxo_with_info.outpoint).is_some_and(|b| *b),
                                                            onchange: {
                                                                let outpoint = utxo_with_info.outpoint;
                                                                move |evt: Event<FormData>| {
                                                                    if evt.checked() {
                                                                        utxo_inc_exc_status.write().insert(outpoint, true);
                                                                    } else {
                                                                        utxo_inc_exc_status.write().remove(&outpoint);
                                                                    }
                                                                }
                                                            },
                                                        }
                                                        "Include"
                                                    }
                                                    label { class: "label",
                                                        input {
                                                            r#type: "checkbox",
                                                            name: "inc_exc",
                                                            class: "checkbox",
                                                            checked: utxo_inc_exc_status.read().get(&utxo_with_info.outpoint).is_some_and(|b| !*b),
                                                            onchange: {
                                                                let outpoint = utxo_with_info.outpoint;
                                                                move |evt: Event<FormData>| {
                                                                    if evt.checked() {
                                                                        utxo_inc_exc_status.write().insert(outpoint, false);
                                                                    } else {
                                                                        utxo_inc_exc_status.write().remove(&outpoint);
                                                                    }
                                                                }
                                                            },
                                                        }
                                                        "Exclude"
                                                    }
                                                }
                                            },
                                            "manual" => rsx! {
                                                label { class: "label",
                                                    input {
                                                        r#type: "checkbox",
                                                        name: "manual_select",
                                                        class: "checkbox",
                                                        checked: utxo_selected_status.read().contains(&utxo_with_info.outpoint),
                                                        onchange: {
                                                            let outpoint = utxo_with_info.outpoint;
                                                            move |evt: Event<FormData>| {
                                                                if evt.checked() {
                                                                    utxo_selected_status.write().insert(outpoint);
                                                                } else {
                                                                    utxo_selected_status.write().remove(&outpoint);
                                                                }
                                                            }
                                                        },
                                                    }
                                                    "Include"
                                                }
                                            },
                                            _ => unreachable!("no other value is possible"),
                                        }
                                    }
                                    td {
                                        AlwaysLoadedComponent::<UIUtxo> { input: utxo_with_info.ref_into() }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}
