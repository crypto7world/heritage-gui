use crate::prelude::*;

use std::collections::{BTreeMap, HashMap, HashSet};

use btc_heritage_wallet::{
    bitcoin::{Address, Amount},
    btc_heritage::{
        heritage_wallet::TransactionSummaryIOTotals, utils::bitcoin_network,
        PartiallySignedTransaction,
    },
    heritage_service_api_client::TransactionSummary,
};

use crate::{
    components::svg::{
        ArrowRight, CheckCircleOutline, DrawSvg,
        SvgSize::{Size4, Size8},
    },
    utils::{amount_to_signed, feerate_sat_per_vb, is_taproot_input_signed, CCStr, CheapClone},
};

use super::{
    balance::UIBtcAmount,
    heritage_configuration::UIExpirationBadge,
    misc::{UIBtcAddr, UITxId},
    timestamp::UITimestamp,
};

#[derive(Debug, Clone, PartialEq)]
enum UITxIOStyle {
    Input,
    ExternalInput,
    OtherInputs,
    SignedInput(Vec<bool>),
    Output,
    ExternalOutput,
    OtherOutputs,
}

#[derive(Debug, Clone, PartialEq)]
struct UITxIO {
    style: UITxIOStyle,
    text: CCStr,
    amounts: Vec<UIBtcAmount>,
}
impl LoadedElement for UITxIO {
    type Loader = TransparentLoader;

    #[inline(always)]
    fn element<M: LoadedComponentInputMapper>(self, m: M) -> Element {
        let (background_class, arrow_color) = match self.style {
            UITxIOStyle::Input | UITxIOStyle::SignedInput(_) => ("bg-error/10", "text-error"),
            UITxIOStyle::Output => ("bg-success/10", "text-success"),
            _ => ("bg-neutral/10", "text-base-content"),
        };

        let font_style = match self.style {
            UITxIOStyle::OtherInputs | UITxIOStyle::OtherOutputs => "",
            // Other style display addresses
            _ => "font-mono",
        };

        let reverse = match self.style {
            UITxIOStyle::Input
            | UITxIOStyle::ExternalInput
            | UITxIOStyle::SignedInput(_)
            | UITxIOStyle::OtherInputs => false,
            UITxIOStyle::Output | UITxIOStyle::ExternalOutput | UITxIOStyle::OtherOutputs => true,
        };

        let signed_inputs = if let UITxIOStyle::SignedInput(signed_inputs) = self.style {
            signed_inputs
        } else {
            vec![]
        };

        rsx! {
            div { class: "{background_class} col-span-3 grid grid-cols-subgrid items-center gap-2 px-2 rounded-box",
                if !reverse {
                    div { class: "row-span-full {font_style} text-right",
                        LoadedComponent { input: m.map(self.text.clone()) }
                    }
                }
                div { class: "col-span-2 grid grid-cols-subgrid items-center",
                    for (idx , amount) in self.amounts.into_iter().enumerate() {
                        if reverse {
                            div {
                                LoadedComponent { input: m.map(amount) }
                            }
                        }
                        div { class: "size-8 relative",
                            div { class: "absolute top-0 left-0 {arrow_color}",
                                DrawSvg::<ArrowRight> { size: Size8 }
                            }
                            if idx < signed_inputs.len() && signed_inputs[idx] {
                                div { class: "absolute top-2 left-2 text-base-content",
                                    DrawSvg::<CheckCircleOutline> { size: Size4 }
                                }
                            }
                        }
                        if !reverse {
                            div { class: "text-right",
                                LoadedComponent { input: m.map(amount) }
                            }
                        }
                    }
                }
                if reverse {

                    MaybeHighlight {
                        step: OnboardingStep::HoverTransactionRecipientAddress,
                        progress: MaybeHighlightProgressType::Hover(2),
                        context_filter: consume_onboarding_context(),
                        div { class: "col-start-3 row-span-full {font_style}",
                            LoadedComponent { input: m.map(self.text) }
                        }
                    }
                }
            }
        }
    }

    fn place_holder() -> Self {
        Self {
            style: UITxIOStyle::Input,
            text: CCStr::place_holder(),
            amounts: vec![UIBtcAmount::place_holder()],
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
enum UITxIOCardStyle {
    Input,
    Output,
}
#[derive(Debug, Clone, PartialEq)]
struct UITxIOCard {
    style: UITxIOCardStyle,
    total_count: usize,
    total_amount: UIBtcAmount,
    ios: CheapClone<[UITxIO]>,
}
impl LoadedElement for UITxIOCard {
    type Loader = TransparentLoader;

    #[inline(always)]
    fn element<M: LoadedComponentInputMapper>(self, m: M) -> Element {
        let (grid_layout, flex_direction, io_name) = match self.style {
            UITxIOCardStyle::Input => (
                "grid-cols-[auto_repeat(2,max-content)]",
                "flex-row",
                "Inputs",
            ),
            UITxIOCardStyle::Output => (
                "grid-cols-[repeat(2,max-content)_auto]",
                "flex-row-reverse",
                "Outputs",
            ),
        };

        rsx! {
            div { class: "card card-border bg-base-100",
                div { class: "card-body",
                    h2 { class: "card-title",
                        "Transaction {io_name}"
                        div { class: "self-center text-xs italic font-normal",
                            match self.style {
                                UITxIOCardStyle::Input => rsx! {
                                    "(Addresses that are "
                                    span { class: "font-black", "spending" }
                                    " coins)"
                                },
                                UITxIOCardStyle::Output => rsx! {
                                    "(Addresses that are "
                                    span { class: "font-black", "receiving" }
                                    " coins)"
                                },
                            }
                        }
                    }
                    div { class: "flex {flex_direction} justify-end gap-8 items-end",
                        div {
                            div { class: "font-semibold text-(--color-base-content)/60",
                                "{io_name} count"
                            }
                            div { class: "text-base text-center",
                                LoadedComponent { input: m.map(self.total_count) }
                            }
                        }
                        div {
                            div { class: "font-semibold text-(--color-base-content)/60",
                                "Total Amount"
                            }
                            div { class: "text-base",
                                LoadedComponent { input: m.map(self.total_amount) }
                            }
                        }
                    }
                    div { class: "grid {grid_layout} gap-y-1",
                        LoadedComponent { input: m.map(self.ios) }
                    }
                }
            }
        }
    }

    fn place_holder() -> Self {
        Self {
            style: UITxIOCardStyle::Input,
            total_count: usize::place_holder(),
            total_amount: UIBtcAmount::place_holder(),
            ios: CheapClone::from([UITxIO::place_holder()]),
        }
    }
}

impl
    From<(
        UITxIOCardStyle,
        TransactionSummaryIOTotals,
        &[TransactionHistoryItemOwnedIO],
    )> for UITxIOCard
{
    fn from(
        (style, io_totals, oios): (
            UITxIOCardStyle,
            TransactionSummaryIOTotals,
            &[TransactionHistoryItemOwnedIO],
        ),
    ) -> Self {
        let TransactionSummaryIOTotals {
            count: total_count,
            amount: total_amount,
        } = io_totals;
        // There is always less or equal owned IOs than the total
        let non_owned_count = total_count - oios.len();
        let other = if non_owned_count > 0 {
            let non_owned_total = total_amount - oios.iter().map(|oio| oio.amount).sum();
            Some(UITxIO {
                style: match style {
                    UITxIOCardStyle::Input => UITxIOStyle::OtherInputs,
                    UITxIOCardStyle::Output => UITxIOStyle::OtherOutputs,
                },
                text: CCStr::from(format!(
                    "{non_owned_count} external {}{}",
                    match style {
                        UITxIOCardStyle::Input => "input",
                        UITxIOCardStyle::Output => "output",
                    },
                    if non_owned_count >= 2 { "s" } else { "" }
                )),
                amounts: vec![UIBtcAmount::from(non_owned_total)],
            })
        } else {
            None
        };
        let owned_by_address = oios.iter().fold(BTreeMap::new(), |mut h, oio| {
            h.entry(oio.address.clone())
                .or_insert(Vec::new())
                .push(oio.amount);
            h
        });

        Self {
            style,
            total_count,
            total_amount: UIBtcAmount::from(total_amount),
            ios: owned_by_address
                .into_iter()
                .map(|(addr, mut amnts)| {
                    amnts.sort();
                    UITxIO {
                        style: match style {
                            UITxIOCardStyle::Input => UITxIOStyle::Input,
                            UITxIOCardStyle::Output => UITxIOStyle::Output,
                        },
                        text: addr,
                        amounts: amnts.into_iter().map(UIBtcAmount::from).collect(),
                    }
                })
                .chain(other)
                .collect(),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct UITxDetails {
    inputs: UITxIOCard,
    outputs: UITxIOCard,
    balance_spent: UIBtcAmount,
    balance_received: UIBtcAmount,
    balance_change: UIBtcAmount,
    fee: UIBtcAmount,
    fee_rate: CCStr,
}
impl LoadedElement for UITxDetails {
    type Loader = TransparentLoader;

    #[inline(always)]
    fn element<M: LoadedComponentInputMapper>(self, m: M) -> Element {
        rsx! {
            div { class: "flex flex-row gap-2",
                LoadedComponent { input: m.map(self.inputs) }
                div { class: "flex flex-col gap-2",
                    div { class: "card card-border bg-base-100",
                        div { class: "card-body",
                            h2 { class: "card-title", "Transaction Fee" }
                            div { class: "flex flex-row justify-center gap-8",
                                div {
                                    div { class: "font-semibold text-xs text-(--color-base-content)/60",
                                        "Fee"
                                    }
                                    div { class: "text-sm",
                                        LoadedComponent { input: m.map(self.fee) }
                                    }
                                }
                                div {
                                    div { class: "font-semibold text-xs text-(--color-base-content)/60",
                                        "Fee rate"
                                    }
                                    div { class: "text-sm",
                                        LoadedComponent { input: m.map(self.fee_rate) }
                                    }
                                }
                            }
                        }
                    }
                    div { class: "card card-border bg-base-100",
                        div { class: "card-body",
                            h2 { class: "card-title", "Wallet Balance Impact" }

                            div { class: "font-bold text-2xl text-center",
                                LoadedComponent { input: m.map(self.balance_change) }
                            }
                            div { class: "flex flex-row gap-2 justify-between",
                                div { class: "basis-1/2 bg-error/10 p-3 rounded-lg min-w-40",
                                    div { class: "text-sm font-semibold text-error",
                                        "Total Spent"
                                    }
                                    div { class: "text-xl font-bold text-center my-2",
                                        LoadedComponent { input: m.map(self.balance_spent) }
                                    }
                                }
                                div { class: "basis-1/2 bg-success/10 p-3 rounded-lg min-w-40 ",
                                    div { class: "text-sm font-semibold text-success",
                                        "Total Received"
                                    }
                                    div { class: "text-xl font-bold text-center my-2",
                                        LoadedComponent { input: m.map(self.balance_received) }
                                    }
                                }
                            }
                        }
                    }
                }
                LoadedComponent { input: m.map(self.outputs) }
            }
        }
    }

    fn place_holder() -> Self {
        let inputs = UITxIOCard::place_holder();
        let mut outputs = inputs.clone();
        outputs.style = UITxIOCardStyle::Output;
        Self {
            inputs,
            outputs,
            balance_spent: UIBtcAmount::place_holder(),
            balance_received: UIBtcAmount::place_holder(),
            balance_change: UIBtcAmount::place_holder(),
            fee: UIBtcAmount::place_holder(),
            fee_rate: CCStr::place_holder(),
        }
    }
}

impl FromRef<TransactionHistoryItem> for UITxDetails {
    fn from_ref(thi: &TransactionHistoryItem) -> Self {
        let balance_spent = UIBtcAmount::from(thi.balance_spent);
        let balance_received = UIBtcAmount::from(thi.balance_received);
        let balance_change = UIBtcAmount::new(Some(thi.balance_change), true);
        let fee = UIBtcAmount::from(thi.fee);
        let fee_rate = CCStr::from(format!("{} sat/vB", feerate_sat_per_vb(thi.fee_rate)));

        let inputs = UITxIOCard::from((
            UITxIOCardStyle::Input,
            thi.inputs_totals,
            thi.owned_inputs.as_ref(),
        ));

        let outputs = UITxIOCard::from((
            UITxIOCardStyle::Output,
            thi.outputs_totals,
            thi.owned_outputs.as_ref(),
        ));

        Self {
            inputs,
            outputs,
            balance_spent,
            balance_received,
            balance_change,
            fee,
            fee_rate,
        }
    }
}

impl FromRef<TransactionSummary> for UITxDetails {
    fn from_ref(tx_sum: &TransactionSummary) -> Self {
        let inputs = UITxIOCard::from((
            UITxIOCardStyle::Input,
            tx_sum.inputs_totals,
            tx_sum
                .owned_inputs
                .iter()
                .map(TransactionHistoryItemOwnedIO::from)
                .collect::<Vec<_>>()
                .as_slice(),
        ));

        let outputs = UITxIOCard::from((
            UITxIOCardStyle::Output,
            tx_sum.outputs_totals,
            tx_sum
                .owned_outputs
                .iter()
                .map(TransactionHistoryItemOwnedIO::from)
                .collect::<Vec<_>>()
                .as_slice(),
        ));

        let balance_spent = tx_sum.owned_inputs.iter().map(|toio| toio.amount).sum();
        let balance_received = tx_sum.owned_outputs.iter().map(|toio| toio.amount).sum();
        let balance_change = amount_to_signed(balance_received) - amount_to_signed(balance_spent);

        let fee = UIBtcAmount::from(tx_sum.fee);
        let fee_rate = CCStr::from(format!("{} sat/vB", feerate_sat_per_vb(tx_sum.fee_rate)));

        Self {
            inputs,
            outputs,
            balance_spent: UIBtcAmount::from(balance_spent),
            balance_received: UIBtcAmount::from(balance_received),
            balance_change: UIBtcAmount::new(Some(balance_change), true),
            fee,
            fee_rate,
        }
    }
}

impl<F: Fn(&Address) -> bool> TryFrom<(&PartiallySignedTransaction, F)> for UITxDetails {
    type Error = String;

    fn try_from(
        (psbt, is_wallet_address): (&PartiallySignedTransaction, F),
    ) -> Result<Self, Self::Error> {
        let tx = &psbt.unsigned_tx;
        let network = bitcoin_network::get();

        // Process inputs
        let mut inputs = HashMap::new();
        let total_input_count = tx.input.len();
        for (tx_input, psbt_input) in tx.input.iter().zip(psbt.inputs.iter()) {
            // Get the previous output to determine the address and amount
            let (address, amount) = if let Some(witness_utxo) = &psbt_input.witness_utxo {
                let address = match Address::from_script(&witness_utxo.script_pubkey, network) {
                    Ok(a) => a,
                    Err(e) => {
                        log::error!("Fail to create address from witness_utxo for input: {psbt_input:?} ({e})");
                        return Err(e.to_string());
                    }
                };
                (address, Amount::from_sat(witness_utxo.value))
            } else if let Some(non_witness_utxo) = &psbt_input.non_witness_utxo {
                let prev_out = &non_witness_utxo.output[tx_input.previous_output.vout as usize];
                let address = match Address::from_script(&prev_out.script_pubkey, network) {
                    Ok(a) => a,
                    Err(e) => {
                        log::error!(
                            "Fail to create address from prev_out for input: {psbt_input:?} ({e})"
                        );
                        return Err(e.to_string());
                    }
                };
                (address, Amount::from_sat(prev_out.value))
            } else {
                // Skip inputs without UTXO information
                log::error!("Skiped input without UTXO info: {psbt_input:?}");
                return Err("Malformed input".to_owned());
            };

            let is_signed = is_taproot_input_signed(psbt_input);

            inputs
                .entry(address)
                .or_insert(Vec::new())
                .push((amount, is_signed));
        }

        let mut owned_input_amount = Amount::ZERO;
        let mut total_input_amount = Amount::ZERO;
        let mut inputs_card_ios = Vec::with_capacity(inputs.len());
        for (address, amount_signed) in inputs.into_iter() {
            let is_owned = is_wallet_address(&address);
            let total_address_amount = amount_signed.iter().map(|(a, _)| *a).sum();
            let (amounts, signed_inputs): (Vec<_>, Vec<_>) = amount_signed
                .into_iter()
                .map(|(a, si)| (UIBtcAmount::from(a), si))
                .unzip();
            if is_owned {
                owned_input_amount += total_address_amount;
            }
            total_input_amount += total_address_amount;
            let style = if is_owned {
                UITxIOStyle::SignedInput(signed_inputs)
            } else {
                UITxIOStyle::ExternalInput
            };
            inputs_card_ios.push(UITxIO {
                style,
                text: CCStr::from(address.to_string()),
                amounts,
            });
        }

        // Process outputs
        let mut outputs = HashMap::new();
        let total_output_count = tx.output.len();
        for tx_output in &tx.output {
            let address = match Address::from_script(&tx_output.script_pubkey, network) {
                Ok(a) => a,
                Err(e) => {
                    log::error!(
                        "Fail to create address from tx_output for output: {tx_output:?} ({e})"
                    );
                    return Err(e.to_string());
                }
            };
            let amount = Amount::from_sat(tx_output.value);
            outputs.entry(address).or_insert(Vec::new()).push(amount);
        }

        let mut owned_output_amount = Amount::ZERO;
        let mut total_output_amount = Amount::ZERO;
        let mut outputs_card_ios = Vec::with_capacity(outputs.len());
        for (address, amounts) in outputs.into_iter() {
            let is_owned = is_wallet_address(&address);
            let total_address_amount = amounts.iter().cloned().sum();
            let amounts = amounts.into_iter().map(|a| UIBtcAmount::from(a)).collect();
            if is_owned {
                owned_output_amount += total_address_amount;
            }
            total_output_amount += total_address_amount;
            let style = if is_owned {
                UITxIOStyle::Output
            } else {
                UITxIOStyle::ExternalOutput
            };
            outputs_card_ios.push(UITxIO {
                style,
                text: CCStr::from(address.to_string()),
                amounts,
            });
        }

        let fee = total_input_amount - total_output_amount;
        let balance_spent = owned_input_amount;
        let balance_received = owned_output_amount;
        let balance_change = amount_to_signed(balance_received) - amount_to_signed(balance_spent);

        // Calculate fee rate (approximate transaction size)
        let tx_weight = tx.weight();
        let fee_rate = fee.to_sat() as f64 / (tx_weight.to_vbytes_ceil() as f64);

        let inputs_card = UITxIOCard {
            style: UITxIOCardStyle::Input,
            total_count: total_input_count,
            total_amount: UIBtcAmount::from(total_input_amount),
            ios: inputs_card_ios.into(),
        };

        let outputs_card = UITxIOCard {
            style: UITxIOCardStyle::Output,
            total_count: total_output_count,
            total_amount: UIBtcAmount::from(total_output_amount),
            ios: outputs_card_ios.into(),
        };

        Ok(Self {
            inputs: inputs_card,
            outputs: outputs_card,
            balance_spent: UIBtcAmount::from(balance_spent),
            balance_received: UIBtcAmount::from(balance_received),
            balance_change: UIBtcAmount::new(Some(balance_change), true),
            fee: UIBtcAmount::from(fee),
            fee_rate: CCStr::from(format!("{:.1} sat/vB", fee_rate)),
        })
    }
}

impl TryFrom<(&PartiallySignedTransaction, &HashSet<Address>)> for UITxDetails {
    type Error = String;
    fn try_from(
        (psbt, wallet_addresses): (&PartiallySignedTransaction, &HashSet<Address>),
    ) -> Result<Self, Self::Error> {
        Self::try_from((psbt, |addr: &Address| wallet_addresses.contains(addr)))
    }
}

impl TryFrom<(&PartiallySignedTransaction, &TransactionSummary)> for UITxDetails {
    type Error = String;
    fn try_from(
        (psbt, tx_sum): (&PartiallySignedTransaction, &TransactionSummary),
    ) -> Result<Self, Self::Error> {
        let wallet_addresses = tx_sum
            .owned_inputs
            .iter()
            .map(|io| &*io.address)
            .chain(tx_sum.owned_outputs.iter().map(|io| &*io.address))
            .cloned()
            .collect::<HashSet<_>>();
        Self::try_from((psbt, &wallet_addresses))
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct UIUtxo {
    outpoint: UITxId,
    confirmation_time: UITimestamp,
    block_height: CCStr,
    address: UIBtcAddr,
    amount: UIBtcAmount,
    heritage_config_expiration: Option<UIExpirationBadge>,
}
impl LoadedElement for UIUtxo {
    type Loader = TransparentLoader;

    #[inline(always)]
    fn element<M: LoadedComponentInputMapper>(self, m: M) -> Element {
        rsx! {
            div { class: "flex flex-row gap-6 items-center",
                div { class: "min-w-36",
                    h3 { class: "text-base font-semibold text-(--color-base-content)/60",
                        "Amount"
                    }
                    div { class: "text-xl",
                        LoadedComponent { input: m.map(self.amount) }
                    }
                }
                div { class: "min-w-xl",
                    div {
                        h4 { class: "font-semibold text-(--color-base-content)/60",
                            "Address"
                        }
                        div { class: "font-mono",
                            LoadedComponent { input: m.map(self.address) }
                        }
                    }
                    div {
                        h4 { class: "font-semibold text-(--color-base-content)/60",
                            "Outpoint"
                        }
                        div { class: "font-mono",
                            LoadedComponent { input: m.map(self.outpoint) }
                        }
                    }
                }
                div { class: "min-w-36",
                    div {
                        h4 { class: "font-semibold text-(--color-base-content)/60",
                            "Confirmation Time"
                        }
                        div {
                            LoadedComponent { input: m.map(self.confirmation_time) }
                        }
                    }
                    div {
                        h4 { class: "font-semibold text-(--color-base-content)/60",
                            "Block Height"
                        }
                        div {
                            LoadedComponent { input: m.map(self.block_height) }
                        }
                    }
                }
            }
        }
    }

    fn place_holder() -> Self {
        Self {
            outpoint: UITxId::place_holder(),
            confirmation_time: UITimestamp::place_holder(),
            block_height: CCStr::place_holder(),
            address: UIBtcAddr::place_holder(),
            amount: UIBtcAmount::place_holder(),
            heritage_config_expiration: None,
        }
    }
}
impl FromRef<UtxoWithInfo> for UIUtxo {
    fn from_ref(utxo_with_info: &UtxoWithInfo) -> Self {
        let outpoint = UITxId::from(utxo_with_info.outpoint.to_string());
        let (confirmation_time, block_height) = utxo_with_info
            .confirmation_time
            .as_ref()
            .map(|bt| {
                (
                    UITimestamp::new_full(bt.timestamp),
                    CCStr::from(format!("{}", bt.height)),
                )
            })
            .unwrap_or_else(|| (UITimestamp::none(), CCStr::from("-")));
        let address = UIBtcAddr::from(utxo_with_info.address.clone());
        let amount = UIBtcAmount::from(utxo_with_info.amount);

        let heritage_config_expiration =
            utxo_with_info
                .heritage_config_expiration
                .map(|expiration_status| {
                    UIExpirationBadge::from((
                        expiration_status,
                        // The is always balance, as the Heritage Config is tied to this UTXO
                        true,
                    ))
                });

        Self {
            outpoint,
            confirmation_time,
            block_height,
            address,
            amount,
            heritage_config_expiration,
        }
    }
}
