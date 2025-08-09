use crate::prelude::*;

use btc_heritage_wallet::KeyProvider;

use crate::{
    components::{
        svg::{CheckCircleOutline, DrawSvg, Signature},
        transaction::UITxDetails,
    },
    utils::{is_psbt_fully_signed, CCStr},
};

use super::SpendStage;

#[doc = "Properties for the [`SignTx`] component."]
#[allow(missing_docs)]
#[derive(Props, Clone, PartialEq)]
#[allow(non_camel_case_types)]
pub(super) struct SignTxProps {
    pub(super) cant_broadcast: ReadOnlySignal<bool>,
}
#[doc = " Component for signing transactions"]
#[doc = "# Props\n*For details, see the [props struct definition](SignTxProps).*"]
#[doc = "- [`cant_broadcast`](SignTxProps::cant_broadcast) : `ReadOnlySignal<bool>`"]
#[allow(non_snake_case)]
pub(super) fn SignTx<S: KeyProvider + 'static>(
    SignTxProps { cant_broadcast }: SignTxProps,
) -> Element {
    log::debug!("SignTx Rendered");

    let signer = use_context::<AsyncSignal<S>>();

    // Cross-stage communication
    let mut current_stage = use_context::<Signal<SpendStage>>();
    let mut psbt_to_sign = use_context::<Signal<Option<super::PsbtToSign>>>();
    let psbt_to_sign_status = use_context::<Memo<super::PsbtToSignStatus>>();
    let mut signed_psbt = use_context::<Signal<Option<super::SignedPsbt>>>();
    let signed_psbt_status = use_context::<Memo<super::SignedPsbtStatus>>();
    let psbt_overview = use_context::<FMemo<UITxDetails>>();

    // Form state
    let mut show_import = use_signal(|| false);
    use_effect(move || {
        if matches!(current_stage(), SpendStage::Sign) {
            *show_import.write() = psbt_to_sign.peek().is_none();
        }
    });

    let mut show_export = use_signal(|| false);

    let can_sign =
        use_memo(move || matches!(*psbt_to_sign_status.read(), super::PsbtToSignStatus::Ok));

    let has_signed =
        use_memo(move || matches!(*signed_psbt_status.read(), super::SignedPsbtStatus::Ok));

    let cant_sign_error = use_memo(move || match psbt_to_sign_status() {
        super::PsbtToSignStatus::Invalid(ccstr) => Some(ccstr),
        super::PsbtToSignStatus::AlreadySigned => {
            Some(CCStr::from("The transaction is already fully signed"))
        }
        super::PsbtToSignStatus::Ok | super::PsbtToSignStatus::Absent => None,
    });
    // Signing state
    let mut signing = use_signal(|| false);

    // Sign transaction handler
    let sign_transaction = move |_| async move {
        if let Some(ref psbt) = psbt_to_sign() {
            let mut psbt_to_sign = match psbt.0.parse() {
                Ok(psbt) => psbt,
                Err(e) => {
                    alert_error(format!("Failed to parse transaction: {e}"));
                    log::error!("Failed to parse transaction: {e}");
                    return;
                }
            };

            *signing.write() = true;
            match signer
                .with(async |s: &S| s.sign_psbt(&mut psbt_to_sign).await)
                .await
            {
                Ok(signed_count) if signed_count > 0 => {
                    *signed_psbt.write() =
                        Some(super::SignedPsbt(CCStr::from(psbt_to_sign.to_string())));
                    alert_success(format!(
                        "Transaction signed successfully. Inputs signed: {signed_count}"
                    ));
                    if is_psbt_fully_signed(&psbt_to_sign) {
                        if cant_broadcast() {
                            *show_export.write() = true;
                        } else {
                            *current_stage.write() = SpendStage::Broadcast;
                        }
                        alert_success("Transaction is ready to broadcast");
                    } else {
                        alert_warn("Transaction is not fully signed and cannot be broadcasted yet");
                    }
                    log::info!("Transaction signed successfully. Inputs signed: {signed_count}");
                }
                Ok(_) => {
                    alert_warn(format!("No inputs owned by this wallet. Nothing signed."));
                    log::info!("No inputs owned by this wallet. Nothing signed.");
                }
                Err(e) => {
                    alert_error(format!("Failed to sign transaction: {e}"));
                    log::error!("Failed to sign transaction: {e}");
                }
            }

            *signing.write() = false;
        }
    };

    use_drop(|| log::debug!("SignTx Dropped"));

    rsx! {
        div { class: "flex flex-col gap-6",
            // Import section
            div { class: "collapse collapse-arrow bg-base-200 text-base-content mt-4",
                input {
                    r#type: "checkbox",
                    checked: show_import(),
                    onchange: move |evt| *show_import.write() = evt.checked(),
                }
                div { class: "collapse-title font-medium", "Import Unsigned Transaction" }
                div { class: "collapse-content",
                    div { class: "flex flex-col gap-2",
                        div { class: "text-sm text-(--color-base-content)/60",
                            "Paste the encoded unsigned transaction (PSBT) to sign:"
                        }
                        textarea {
                            class: "textarea textarea-bordered font-mono text-xs w-full",
                            rows: "8",
                            placeholder: "Paste PSBT here...",
                            readonly: has_signed(),
                            value: if let Some(psbt) = psbt_to_sign() { "{psbt.0}" } else { "" },
                            oninput: move |evt| {
                                let val = evt.value();
                                if val.is_empty() {
                                    *psbt_to_sign.write() = None;
                                } else {
                                    *psbt_to_sign.write() = Some(super::PsbtToSign(CCStr::from(val)));
                                }
                            },
                        }
                        if let Some(msg) = cant_sign_error() {
                            div { class: "text-sm text-error", {msg} }
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

            div { class: "card-actions justify-center mt-6",
                MaybeHighlight {
                    step: OnboardingStep::ClickInheritanceSignTransaction,
                    progress: MaybeHighlightProgressType::Signal(has_signed.into()),
                    context_filter: consume_onboarding_context(),
                    button {
                        class: "btn btn-primary",
                        disabled: !can_sign() || signing() || has_signed(),
                        onclick: sign_transaction,
                        if signing() {
                            span { class: "loading loading-spinner loading-sm mr-2" }
                            "Signing..."
                        } else if has_signed() {
                            DrawSvg::<CheckCircleOutline> {}
                            "Sign Complete"
                        } else {
                            DrawSvg::<Signature> {}
                            "Sign Transaction"
                        }
                    }
                }
            }

            if let Some(psbt) = signed_psbt() {
                div { class: "collapse collapse-arrow bg-base-200 text-base-content mt-4",
                    input {
                        r#type: "checkbox",
                        checked: show_export(),
                        onchange: move |evt| *show_export.write() = evt.checked(),
                    }
                    div { class: "collapse-title font-medium", "Export Signed Transaction" }
                    div { class: "collapse-content",
                        div { class: "flex flex-col gap-2",
                            div { class: "text-sm text-(--color-base-content)/60",
                                "Copy this encoded transaction to broadcast on another device or save for later:"
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
        }
    }
}
