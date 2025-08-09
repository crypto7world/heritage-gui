use super::*;

/// Component for creating new transactions
#[component]
pub fn CreateOwnerTx(cant_sign: ReadOnlySignal<bool>) -> Element {
    log::debug!("CreateOwnerTx Rendered");

    let wallet = use_context::<AsyncSignal<Wallet>>();
    let wallet_status = use_context::<FResource<WalletStatus>>();
    let max_spendable_amount = use_memo(move || {
        wallet_status.lrmap(|wallet_status| {
            Amount::from_sat(wallet_status.balance.total_balance().get_spendable())
        })
    });
    use_context_provider(|| max_spendable_amount);

    // Cross-stage communication
    let mut current_stage = use_context::<Signal<SpendStage>>();
    let mut psbt_to_sign = use_context::<Signal<Option<PsbtToSign>>>();
    let mut signed_psbt = use_context::<Signal<Option<SignedPsbt>>>();
    let mut shared_tx_summary = use_context::<Signal<Option<TransactionSummary>>>();
    let psbt_overview = use_context::<FMemo<UITxDetails>>();

    // Form state
    let mut recipients = use_signal(|| vec![RecipientState::default()]);
    use_context_provider(|| recipients);
    let mut show_advanced = use_signal(|| false);

    // Advanced options state
    let fee_policy: Signal<Option<NewTxFeePolicy>> = use_signal(|| None);
    let default_fee_rate = use_memo(move || {
        (match *wallet_status.read() {
            Some(Ok(ref wallet_status)) => wallet_status.last_fee_rate.map(feerate_sat_per_vb),
            _ => None,
        })
        .unwrap_or(1.0)
    });

    let utxo_selection: Signal<Option<NewTxUtxoSelection>> = use_signal(|| None);

    // Transaction creation state
    let mut creating = use_signal(|| false);

    let mut show_export = use_signal(|| false);

    // Validation

    let total_spend = use_memo(move || {
        recipients()
            .iter()
            .filter_map(|r| {
                Amount::from_str_in(
                    &r.amount,
                    match r.unit {
                        RecipientAmountUnit::Sat => Denomination::Satoshi,
                        RecipientAmountUnit::MilliBtc => Denomination::MilliBitcoin,
                        RecipientAmountUnit::Btc => Denomination::Bitcoin,
                    },
                )
                .ok()
            })
            .sum::<Amount>()
    });

    let at_least_one_recipient = use_memo(move || !recipients().is_empty());

    let multiple_drain_to_error =
        use_memo(move || recipients().iter().filter(|r| r.drain_to).count() > 1);

    let max_one_recipient_error =
        use_memo(move || recipients().iter().any(|r| r.drain_to) && recipients().len() > 1);

    let max_spend_error = use_memo(move || {
        total_spend()
            > max_spendable_amount()
                .unwrap_or(Ok(Amount::ZERO))
                .unwrap_or(Amount::ZERO)
    });
    let all_addresses_ok = use_memo(move || {
        recipients()
            .iter()
            .all(|r| match r.address.parse::<Address<NetworkUnchecked>>() {
                Ok(addr) => addr.require_network(bitcoin_network::get()).is_ok(),
                Err(_) => false,
            })
    });

    let form_valid = use_memo(move || {
        at_least_one_recipient()
            && all_addresses_ok()
            && !multiple_drain_to_error()
            && !max_one_recipient_error()
            && !max_spend_error()
    });

    // Create transaction handler
    let create_transaction = move |_| async move {
        *creating.write() = true;

        let recipients = &*recipients.read();

        let spending_config = if recipients.len() == 1 && recipients[0].drain_to {
            NewTxSpendingConfig::DrainTo(NewTxDrainTo {
                drain_to: recipients[0].address.clone(),
            })
        } else {
            NewTxSpendingConfig::Recipients(
                recipients
                    .iter()
                    .filter_map(|r| {
                        Amount::from_str_in(
                            &r.amount,
                            match r.unit {
                                RecipientAmountUnit::Sat => Denomination::Satoshi,
                                RecipientAmountUnit::MilliBtc => Denomination::MilliBitcoin,
                                RecipientAmountUnit::Btc => Denomination::Bitcoin,
                            },
                        )
                        .ok()
                        .map(|amount| NewTxRecipient {
                            address: r.address.clone(),
                            amount: amount.to_sat(),
                        })
                    })
                    .collect(),
            )
        };

        let new_tx = NewTx {
            spending_config,
            fee_policy: fee_policy(),
            utxo_selection: utxo_selection.cloned(),
            disable_rbf: None,
        };

        match wallet
            .with(async |w: &Wallet| w.create_psbt(new_tx).await)
            .await
        {
            Ok((psbt, tx_summary)) => {
                // Share with other stages
                *psbt_to_sign.write() = Some(PsbtToSign(CCStr::from(psbt.to_string())));
                *shared_tx_summary.write() = Some(tx_summary);
                *signed_psbt.write() = None;
                if cant_sign() {
                    *show_export.write() = true;
                } else {
                    *current_stage.write() = SpendStage::Sign;
                }
                log::info!("Transaction created successfully");
            }
            Err(e) => {
                alert_error(format!("Failed to create transaction: {e}"));
                log::error!("Failed to create transaction: {e}");
            }
        }

        *creating.write() = false;
    };

    // Add recipient handler
    let add_recipient = move |_| {
        recipients.write().push(RecipientState::default());
    };

    use_drop(|| log::debug!("CreateOwnerTx Dropped"));

    rsx! {
        div { class: "flex flex-col gap-6",
            // Main form
            div { class: "card shadow-xl",
                div { class: "card-body",
                    h2 { class: "card-title mb-6",
                        div { class: "grid grid-cols-[1fr_1fr_auto] gap-x-4",
                            div { "Transaction spend:" }
                            div { class: if max_spend_error() { "text-error" },
                                AlwaysLoadedComponent::<UIBtcAmount> { input: total_spend().into() }
                                span { class: "text-warning", " *" }
                            }
                            span { class: "text-xs font-light text-warning",
                                "Do not forget to leave room for the transaction fee."
                            }
                            div { "Max spendable:" }
                            div {
                                LoadedComponent::<UIBtcAmount> { input: max_spendable_amount.into() }
                                span { class: "text-info", " *" }
                            }
                            span { class: "text-xs font-light text-info",
                                "This does not include unconfirmed transaction coming from outside of your wallet."
                            }
                        
                        }
                    }

                    // Recipients
                    div { class: "flex flex-col gap-4",
                        for (index , recipient) in recipients.read().iter().enumerate() {
                            OwnerTxRecipientInput {
                                key: "{recipient.uuid}",
                                index,
                                max_one_recipient_error,
                                multiple_drain_to_error,
                                max_spend_error,
                            }
                        }
                    }

                    // Action buttons
                    div { class: "card-actions",
                        button {
                            class: "btn btn-outline btn-sm",
                            onclick: add_recipient,
                            DrawSvg::<Plus> {}
                            "Add Recipient"
                        }
                    }
                }
            }

            // Advanced settings
            div { class: "collapse collapse-arrow shadow-xl",
                input {
                    r#type: "checkbox",
                    checked: show_advanced(),
                    onchange: move |evt| *show_advanced.write() = evt.checked(),
                }
                div { class: "collapse-title text-xl font-medium", "Advanced Settings" }

                div { class: "collapse-content",
                    div { class: "flex flex-col gap-4",
                        FeePolicyInput { fee_policy, default_fee_rate }
                        UtxoSelection { utxo_selection }
                    }
                }
            }

            div { class: "card-actions justify-center mt-6",
                button {
                    class: "btn btn-primary",
                    disabled: !form_valid() || creating(),
                    onclick: create_transaction,
                    if creating() {
                        span { class: "loading loading-spinner loading-sm mr-2" }
                        "Creating..."
                    } else {
                        DrawSvg::<BankPlus> {}
                        "Create Transaction"
                    }
                }
            }
            if let Some(psbt) = psbt_to_sign() {
                div { class: "collapse collapse-arrow bg-base-200 text-base-content mt-4",
                    input {
                        r#type: "checkbox",
                        checked: show_export(),
                        onchange: move |evt| *show_export.write() = evt.checked(),
                    }
                    div { class: "collapse-title font-medium", "Export Unsigned Transaction" }
                    div { class: "collapse-content",
                        div { class: "flex flex-col gap-2",
                            div { class: "text-sm text-(--color-base-content)/60",
                                "Copy this encoded transaction to sign on another device or save for later:"
                            }
                            textarea {
                                class: "textarea textarea-bordered font-mono text-xs w-full",
                                rows: "8",
                                readonly: true,
                                value: psbt.0.as_ref(),
                            }
                            button {
                                class: "btn btn-outline btn-sm",
                                onclick: move |_| {
                                    log::info!("Copy to clipboard functionality will be implemented");
                                },
                                "Copy to Clipboard"
                            }
                        }
                    }
                }
            }

            div { class: "card shadow-xl",
                div { class: "card-body overflow-x-auto",
                    h2 { class: "card-title", "Transaction Overview" }
                    if let Some(psbt_overview) = psbt_overview.cloned() {
                        LoadedComponent { input: psbt_overview.into() }
                    } else {
                        "No Transaction to display yet..."
                    }
                }
            }
        }
    }
}

/// Component for a single recipient input
#[component]
fn OwnerTxRecipientInput(
    index: usize,
    max_one_recipient_error: ReadOnlySignal<bool>,
    multiple_drain_to_error: ReadOnlySignal<bool>,
    max_spend_error: ReadOnlySignal<bool>,
) -> Element {
    log::debug!("OwnerTxRecipientInput {index} Rendered");

    let mut recipients = use_context::<Signal<Vec<RecipientState>>>();
    let max_spendable_amount = use_context::<FMemo<Amount>>();

    let recipient = use_memo(move || {
        log::debug!("OwnerTxRecipientInput {index} - Compute recipient");
        recipients.read()[index].clone()
    });

    let address_error = use_memo(move || {
        log::debug!("OwnerTxRecipientInput {index} - Compute address_error");
        match recipient
            .read()
            .address
            .parse::<Address<NetworkUnchecked>>()
        {
            Ok(addr) => match addr.require_network(bitcoin_network::get()) {
                Ok(_) => None,
                Err(e) => Some(e.to_string()),
            },
            Err(e) if !recipient.read().address.is_empty() => Some(format!("Address invalid: {e}")),
            _ => None,
        }
    });
    let multiple_drain_to_error = use_memo(move || {
        log::debug!("OwnerTxRecipientInput {index} - Compute multiple_drain_to_error");
        if multiple_drain_to_error() && recipient.read().drain_to {
            Some("Only one recipient can have \"Send All\" checked")
        } else {
            None
        }
    });
    let max_one_recipient_error = use_memo(move || {
        log::debug!("OwnerTxRecipientInput {index} - Compute max_one_recipient_error");
        if max_one_recipient_error() {
            Some(if recipient.read().drain_to {
                (
                    "Recipient with \"Send All\" should be the only one",
                    "text-warning",
                )
            } else {
                ("Another recipient has \"Send All\" checked", "text-error")
            })
        } else {
            None
        }
    });

    let max_spend_error = use_memo(move || {
        log::debug!("OwnerTxRecipientInput {index} - Compute max_spend_error");
        if max_spend_error() {
            Some("This transaction is trying to spend more than the available balance")
        } else {
            None
        }
    });

    let send_all_check = move |evt: Event<FormData>| {
        let recipient = &mut recipients.write()[index];
        recipient.drain_to = evt.checked();
        if recipient.drain_to {
            if let Some(Ok(ref max_spendable_amount)) = *max_spendable_amount.read() {
                (recipient.amount, recipient.unit) =
                    match denomination_for_amount(*max_spendable_amount) {
                        Denomination::Bitcoin => (
                            format!("{}", max_spendable_amount.display_in(Denomination::Bitcoin),),
                            RecipientAmountUnit::Btc,
                        ),
                        Denomination::MilliBitcoin => (
                            format!(
                                "{}",
                                max_spendable_amount.display_in(Denomination::MilliBitcoin),
                            ),
                            RecipientAmountUnit::MilliBtc,
                        ),
                        Denomination::Satoshi => (
                            format!("{}", max_spendable_amount.display_in(Denomination::Satoshi),),
                            RecipientAmountUnit::Sat,
                        ),
                        _ => {
                            unreachable!("denomination_for_amount never return another denom",)
                        }
                    };
            }
        }
    };

    use_drop(move || log::debug!("OwnerTxRecipientInput {index} Dropped"));

    rsx! {
        div { class: "flex flex-col gap-2 p-4 border border-base-300 rounded-lg",
            div { class: "flex justify-between items-center mb-2",
                span { class: "font-semibold text-xl", "Recipient #{index + 1}" }
                button {
                    class: "btn btn-circle btn-outline btn-primary btn-xs",
                    onclick: move |_| {
                        recipients.write().remove(index);
                    },
                    disabled: recipients.read().len() <= 1,
                    DrawSvg::<Close> { size: Size3 }
                }
            }
            div { class: "flex flex-row gap-2 flex-wrap",
                fieldset { class: "fieldset w-lg",
                    legend { class: "fieldset-legend", "Bitcoin Address" }
                    input {
                        r#type: "url",
                        class: "input w-full",
                        value: "{recipient.read().address}",
                        placeholder: "Enter address...",
                        oninput: move |evt| recipients.write()[index].address = evt.value().to_string(),
                    }
                    if let Some(ref address_error) = *address_error.read() {
                        div { class: "label text-error", "{address_error}" }
                    }
                    if let Some(ref multiple_drain_to_error) = *multiple_drain_to_error.read() {
                        div { class: "label text-error", {multiple_drain_to_error} }
                    }
                    if let Some((ref max_one_recipient_error, ref text_color)) = *max_one_recipient_error
                        .read()
                    {
                        div { class: "label {text_color}", {max_one_recipient_error} }
                    }
                }
                fieldset { class: "fieldset",
                    legend { class: "fieldset-legend", "Amount" }
                    div { class: "flex flex-row gap-2",
                        div { class: "join",
                            input {
                                r#type: "number",
                                class: "join-item input w-40",
                                placeholder: "Amount",
                                disabled: recipient.read().drain_to,
                                value: "{recipient.read().amount}",
                                oninput: move |evt| recipients.write()[index].amount = evt.value().to_string(),
                            }
                            select {
                                class: "join-item select w-24",
                                disabled: recipient.read().drain_to,
                                value: recipient.read().unit.to_string(),
                                onchange: move |evt| recipients.write()[index].unit = evt.value().parse().unwrap(),
                                option { value: "sat", "sat" }
                                option { value: "mBTC", "mBTC" }
                                option { value: "BTC", "BTC" }
                            }
                        }
                        label { class: "label",
                            input {
                                r#type: "checkbox",
                                class: "checkbox",
                                checked: recipient.read().drain_to,
                                onchange: send_all_check,
                            }
                            "Send All"
                        }
                    }
                    if let Some(ref max_spend_error) = *max_spend_error.read() {
                        div { class: "label text-error", {max_spend_error} }
                    }
                }
            }
        }
    }
}
