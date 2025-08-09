use super::*;

/// Component for creating new transactions
#[component]
pub fn CreateHeirTx(heritage_id: CCStr, cant_sign: ReadOnlySignal<bool>) -> Element {
    log::debug!("CreateHeirTx Rendered");

    let heirwallet = use_context::<AsyncSignal<HeirWallet>>();
    let max_spendable_amount = use_context::<FMemo<Amount>>();

    // Cross-stage communication
    let mut current_stage = use_context::<Signal<SpendStage>>();
    let mut psbt_to_sign = use_context::<Signal<Option<PsbtToSign>>>();
    let mut signed_psbt = use_context::<Signal<Option<SignedPsbt>>>();
    let mut shared_tx_summary = use_context::<Signal<Option<TransactionSummary>>>();
    let psbt_overview = use_context::<FMemo<UITxDetails>>();

    // Form state
    let mut recipient = use_signal(|| {
        let mut recipient = RecipientState::default();
        recipient.drain_to = true;
        recipient
    });
    use_effect(move || {
        if let Some(Ok(ref max_spendable_amount)) = *max_spendable_amount.read() {
            let r = &mut *recipient.write();
            (r.amount, r.unit) = match denomination_for_amount(*max_spendable_amount) {
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
    });

    let recipient_address = use_memo(move || {
        let recipient_address = &recipient.read().address;
        match recipient_address.parse::<Address<NetworkUnchecked>>() {
            Ok(addr) => addr
                .require_network(bitcoin_network::get())
                .map(Some)
                .map_err(|e| e.to_string()),
            Err(e) if !recipient_address.is_empty() => Err(format!("Address invalid: {e}")),
            _ => Ok(None),
        }
    });

    let mut show_advanced = use_signal(|| false);

    // Advanced options state
    let fee_policy: Signal<Option<NewTxFeePolicy>> = use_signal(|| None);
    let default_fee_rate = use_signal(|| 1.0);

    // Transaction creation state
    let mut creating = use_signal(|| false);

    let mut show_export = use_signal(|| false);

    // Validation
    let form_valid = use_memo(move || recipient_address.read().as_ref().is_ok_and(Option::is_some));

    // Create transaction handler
    let create_transaction = {
        move |_| {
            let heritage_id = heritage_id.clone();
            async move {
                *creating.write() = true;

                let address = recipient_address.cloned().unwrap().unwrap();

                match heirwallet
                    .with(async |hw: &HeirWallet| {
                        hw.create_psbt(heritage_id.as_ref(), address).await
                    })
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
            }
        }
    };

    // Retrieve the OnboardingContextItem if it exist
    let has_created = use_memo(move || psbt_to_sign.read().is_some());

    use_drop(|| log::debug!("CreateTx Dropped"));

    rsx! {
        div { class: "flex flex-col gap-6",
            // Main form
            div { class: "card shadow-xl",
                div { class: "card-body",
                    h2 { class: "card-title",
                        div { class: "grid grid-cols-2 gap-x-4",

                            div { "Transaction spend:" }
                            div {
                                LoadedComponent::<UIBtcAmount> { input: max_spendable_amount.into() }
                            }
                        }
                    }

                    // Recipients
                    div { class: "flex flex-col gap-4",
                        HeirTxRecipientInput { recipient }
                    }
                }
            }

            // Advanced settings
            //
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
                    }
                }
            }

            div { class: "card-actions justify-center mt-6",
                MaybeHighlight {
                    step: OnboardingStep::ClickInheritanceCreateTransaction,
                    progress: MaybeHighlightProgressType::Signal(has_created.into()),
                    context_filter: consume_onboarding_context(),
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
fn HeirTxRecipientInput(recipient: Signal<RecipientState>) -> Element {
    log::debug!("HeirTxRecipientInput Rendered");

    let address_error = use_memo(move || {
        log::debug!("HeirTxRecipientInput - Compute address_error");
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

    // Retrieve the OnboardingContextItem if it exist
    let valid_address =
        use_memo(move || !recipient.read().address.is_empty() && address_error.read().is_none());

    use_drop(move || log::debug!("HeirTxRecipientInput Dropped"));

    rsx! {
        div { class: "flex flex-col gap-2 p-4 border border-base-300 rounded-lg",
            div { class: "font-semibold text-xl mb-2", "Send inheritance to" }
            div { class: "flex flex-row gap-2 flex-wrap",

                MaybeHighlight {
                    step: OnboardingStep::InputInheritanceSpendAddress,
                    progress: MaybeHighlightProgressType::Signal(valid_address.into()),
                    context_filter: consume_onboarding_context(),
                    fieldset { class: "fieldset w-lg",
                        legend { class: "fieldset-legend", "Bitcoin Address" }
                        input {
                            r#type: "url",
                            class: "input w-full",
                            value: "{recipient.read().address}",
                            placeholder: "Enter address...",
                            oninput: move |evt| recipient.write().address = evt.value().to_string(),
                        }
                        if let Some(ref address_error) = *address_error.read() {
                            div { class: "label text-error", "{address_error}" }
                        }
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
                                disabled: true,
                                value: "{recipient.read().amount}",
                            }
                            select {
                                class: "join-item select w-24",
                                disabled: true,
                                value: recipient.read().unit.to_string(),
                                option { value: "sat", "sat" }
                                option { value: "mBTC", "mBTC" }
                                option { value: "BTC", "BTC" }
                            }
                        }
                        label { class: "label",
                            input {
                                r#type: "checkbox",
                                class: "checkbox",
                                disabled: true,
                                checked: recipient.read().drain_to,
                            }
                            "Send All"
                        }
                    }
                }
            }
        }
    }
}
