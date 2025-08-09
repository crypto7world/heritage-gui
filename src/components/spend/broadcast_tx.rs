use crate::prelude::*;

use btc_heritage_wallet::Broadcaster;

use super::SpendStage;

use crate::{
    components::{
        svg::{AlertOutline, Broadcast, CheckBold, DrawSvg, InfoCircle},
        transaction::UITxDetails,
    },
    utils::CCStr,
};

/// Component for broadcasting transactions
#[component]
pub(super) fn BroadcastTx<B: Broadcaster + 'static>() -> Element {
    log::debug!("BroadcastTx Rendered");

    let broadcaster = use_context::<AsyncSignal<B>>();

    // Cross-stage communication
    let current_stage = use_context::<Signal<SpendStage>>();
    let mut signed_psbt = use_context::<Signal<Option<super::SignedPsbt>>>();
    let signed_psbt_status = use_context::<Memo<super::SignedPsbtStatus>>();
    let psbt_overview = use_context::<FMemo<UITxDetails>>();

    // Form state

    let mut show_import = use_signal(|| false);
    use_effect(move || {
        if matches!(current_stage(), SpendStage::Broadcast) {
            *show_import.write() = signed_psbt.peek().is_none();
        }
    });

    // Broadcasting state
    let mut broadcasting = use_signal(|| false);
    let mut broadcast_txid = use_signal(|| None);
    let can_broadcast =
        use_memo(move || matches!(*signed_psbt_status.read(), super::SignedPsbtStatus::Ok));
    let has_broadcast = use_memo(move || broadcast_txid.read().is_some());

    let cant_broadcast_error = use_memo(move || match signed_psbt_status() {
        super::SignedPsbtStatus::Invalid(ccstr) => Some(ccstr),
        super::SignedPsbtStatus::NotSigned => Some(CCStr::from(
            "This transaction is valid but not fully signed",
        )),
        super::SignedPsbtStatus::Ok | super::SignedPsbtStatus::Absent => None,
    });

    // Broadcast transaction handler
    let broadcast_transaction = move |_| async move {
        if let Some(ref psbt) = signed_psbt() {
            let psbt_to_broadcast = match psbt.0.parse() {
                Ok(psbt) => psbt,
                Err(e) => {
                    alert_error(format!("Failed to parse transaction: {e}"));
                    log::error!("Failed to parse transaction: {e}");
                    return;
                }
            };

            *broadcasting.write() = true;

            match broadcaster
                .with(async |b: &B| b.broadcast(psbt_to_broadcast).await)
                .await
            {
                Ok(txid) => {
                    broadcast_txid.write().replace(txid);
                    alert_success("Transaction broadcast successfully: {txid}");
                    log::info!("Transaction broadcast successfully: {txid}");
                }
                Err(e) => {
                    alert_error(format!("Failed to broadcast transaction: {e}"));
                    log::error!("Failed to broadcast transaction: {e}");
                }
            }

            *broadcasting.write() = false;
        }
    };

    // Retrieve the OnboardingContextItem if it exist

    use_drop(|| log::debug!("BroadcastTx Dropped"));

    rsx! {
        div { class: "flex flex-col gap-6",
            // Import section
            div { class: "collapse collapse-arrow bg-base-200 text-base-content mt-4",
                input {
                    r#type: "checkbox",
                    checked: show_import(),
                    onchange: move |evt| *show_import.write() = evt.checked(),
                }
                div { class: "collapse-title font-medium", "Import Signed Transaction" }
                div { class: "collapse-content",
                    div { class: "flex flex-col gap-2",
                        div { class: "text-sm text-(--color-base-content)/60",
                            "Paste the encoded signed transaction (PSBT) to broadcast:"
                        }
                        textarea {
                            class: "textarea textarea-bordered font-mono text-xs w-full",
                            rows: "8",
                            placeholder: "Paste PSBT here...",
                            readonly: has_broadcast(),
                            value: if let Some(psbt) = signed_psbt() { "{psbt.0}" } else { "" },
                            oninput: move |evt| {
                                let val = evt.value();
                                if val.is_empty() {
                                    *signed_psbt.write() = None;
                                } else {
                                    *signed_psbt.write() = Some(super::SignedPsbt(CCStr::from(val)));
                                }
                            },
                        }
                        if let Some(msg) = cant_broadcast_error() {
                            div { class: "text-sm text-error", {msg} }
                        }
                    }
                }
            }


            // Transaction display and broadcasting
            if can_broadcast() && !has_broadcast() {
                div { class: "alert alert-warning mt-4",
                    DrawSvg::<AlertOutline> {}
                    div {
                        div { class: "font-medium", "Warning:" }
                        div { class: "text-sm mt-1",
                            "Broadcasting this transaction will submit it to the Bitcoin network. This action cannot be undone. Please verify all details are correct before proceeding."
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

            // Broadcasting section
            div { class: "card-actions justify-center mt-6",
                MaybeHighlight {
                    step: OnboardingStep::ClickInheritanceBroadcastTransaction,
                    progress: MaybeHighlightProgressType::Signal(has_broadcast.into()),
                    context_filter: consume_onboarding_context(),
                    button {
                        class: "btn btn-primary",
                        disabled: !can_broadcast() || broadcasting() || has_broadcast(),
                        onclick: broadcast_transaction,
                        if broadcasting() {
                            span { class: "loading loading-spinner loading-sm mr-2" }
                            "Broadcasting..."
                        } else if has_broadcast() {
                            "Broadcast Complete"
                        } else {
                            DrawSvg::<Broadcast> {}
                            "Broadcast Transaction"
                        }
                    }
                }
            }


            if let Some(txid) = broadcast_txid() {
                div { class: "card bg-success text-success-content shadow-xl",
                    div { class: "card-body",
                        h2 { class: "card-title",
                            DrawSvg::<CheckBold> {}
                            "Transaction Broadcast Successfully"
                        }

                        div { class: "mt-4 p-4 bg-base-100 text-base-content rounded-lg",
                            div { class: "text-lg font-semibold mb-2", "Transaction ID" }
                            div { class: "font-mono text-sm break-all select-all p-2 bg-base-200 rounded border",
                                "{txid}"
                            }

                            div { class: "text-sm text-gray-600 mt-2",
                                "Your transaction has been submitted to the Bitcoin network. It may take some time to be confirmed depending on network conditions and the fee paid."
                            }
                        }

                        div { class: "alert alert-info mt-4",
                            DrawSvg::<InfoCircle> {}
                            div {
                                div { class: "font-medium", "Next Steps:" }
                                ul { class: "text-sm mt-1 list-disc list-inside",
                                    li {
                                        "Monitor the transaction status in your wallet's transaction history"
                                    }
                                    li {
                                        "Wait for network confirmations (typically 1-6 confirmations recommended)"
                                    }
                                    li {
                                        "The recipient will see the funds once the transaction is confirmed"
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
