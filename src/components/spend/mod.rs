use crate::{components::quick_actions::UnlockLocalKey, prelude::*};

mod broadcast_tx;
mod create_tx;
mod sign_tx;

use std::collections::HashSet;

use btc_heritage_wallet::{
    bitcoin::Address, btc_heritage::PartiallySignedTransaction,
    heritage_service_api_client::TransactionSummary, Broadcaster, KeyProvider,
};

use crate::{
    components::{
        svg::{Alert, ChevronRight, DrawSvg, One, SvgSize::Size8, Three, Two},
        transaction::UITxDetails,
    },
    utils::{is_psbt_fully_signed, log_error_ccstr, CCStr},
};

use super::quick_actions::LocalKeyUnlocker;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SpendTabsType {
    Owner,
    Heir(CCStr),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum SpendStage {
    Create,
    Sign,
    Broadcast,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct PsbtToSign(CCStr);
#[derive(Debug, Clone, PartialEq, Eq)]
enum PsbtToSignStatus {
    Absent,
    Invalid(CCStr),
    AlreadySigned,
    Ok,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct SignedPsbt(CCStr);
#[derive(Debug, Clone, PartialEq, Eq)]
enum SignedPsbtStatus {
    Absent,
    Invalid(CCStr),
    NotSigned,
    Ok,
}

#[doc = "Properties for the [`SpendTabs`] component."]
#[allow(missing_docs)]
#[derive(Props, Clone, PartialEq)]
#[allow(non_camel_case_types)]
pub struct SpendTabsProps {
    pub spendtabs_type: SpendTabsType,
    pub cannot_create_reason: ReadOnlySignal<Option<&'static str>>,
    pub cannot_sign_reason: ReadOnlySignal<Option<(&'static str, bool)>>,
    pub cannot_broadcast_reason: ReadOnlySignal<Option<&'static str>>,
    pub addresses_set: ReadOnlySignal<LResult<HashSet<Address>>>,
}
#[doc = "# Props\n*For details, see the [props struct definition](SpendTabsProps).*"]
#[doc = "- [`spendtabs_type`](SpendTabsProps::spendtabs_type) : `SpendTabsType`"]
#[doc = "- [`cannot_create_reason`](SpendTabsProps::cannot_create_reason) : `ReadOnlySignal<Option<&'staticstr>>`"]
#[doc = "- [`cannot_sign_reason`](SpendTabsProps::cannot_sign_reason) : `ReadOnlySignal<Option<&'staticstr>>`"]
#[doc = "- [`cannot_broadcast_reason`](SpendTabsProps::cannot_broadcast_reason) : `ReadOnlySignal<Option<&'staticstr>>`"]
#[doc = "- [`addresses_set`](SpendTabsProps::addresses_set) : `ReadOnlySignal<Option<HashSet<Address>>>`"]
#[allow(non_snake_case)]
pub fn SpendTabs<T: KeyProvider + Broadcaster + LocalKeyUnlocker + 'static>(
    SpendTabsProps {
        spendtabs_type,
        cannot_create_reason,
        cannot_sign_reason,
        cannot_broadcast_reason,
        addresses_set,
    }: SpendTabsProps,
) -> Element {
    log::debug!("SpendTabs Rendered");

    let cant_create = use_memo(move || cannot_create_reason.read().is_some());

    let cant_sign = use_memo(move || cannot_sign_reason.read().is_some());

    let cant_broadcast = use_memo(move || cannot_broadcast_reason.read().is_some());

    // Shared PSBT data between stages
    let current_stage = use_signal(|| SpendStage::Create);
    let psbt_to_sign: Signal<Option<PsbtToSign>> = use_signal(|| None);
    let psbt_to_sign_status =
        use_memo(
            move || match psbt_to_sign.lmap(|s| s.0.parse::<PartiallySignedTransaction>()) {
                Some(Ok(psbt)) if is_psbt_fully_signed(&psbt) => PsbtToSignStatus::AlreadySigned,
                Some(Ok(_)) => PsbtToSignStatus::Ok,
                Some(Err(e)) => PsbtToSignStatus::Invalid(CCStr::from(e.to_string())),
                None => PsbtToSignStatus::Absent,
            },
        );

    let signed_psbt: Signal<Option<SignedPsbt>> = use_signal(|| None);
    let signed_psbt_status =
        use_memo(
            move || match signed_psbt.lmap(|s| s.0.parse::<PartiallySignedTransaction>()) {
                Some(Ok(psbt)) if is_psbt_fully_signed(&psbt) => SignedPsbtStatus::Ok,
                Some(Ok(_)) => SignedPsbtStatus::NotSigned,
                Some(Err(e)) => SignedPsbtStatus::Invalid(CCStr::from(e.to_string())),
                None => SignedPsbtStatus::Absent,
            },
        );

    let shared_tx_summary: Signal<Option<TransactionSummary>> = use_signal(|| None);

    // Provide shared state to all tabs
    use_context_provider(|| current_stage);
    use_context_provider(|| psbt_to_sign);
    use_context_provider(|| psbt_to_sign_status);
    use_context_provider(|| signed_psbt);
    use_context_provider(|| signed_psbt_status);
    use_context_provider(|| shared_tx_summary);

    // Compute the PSBT overview
    let psbt_overview = use_memo(move || {
        let psbt: Option<CCStr> = match current_stage() {
            SpendStage::Create => psbt_to_sign.lmap(|s| s.0.clone()),
            SpendStage::Sign if signed_psbt.read().is_some() => signed_psbt.lmap(|s| s.0.clone()),
            SpendStage::Sign => psbt_to_sign.lmap(|s| s.0.clone()),
            SpendStage::Broadcast => signed_psbt.lmap(|s| s.0.clone()),
        };
        let Some(psbt) = psbt else {
            return None;
        };

        let psbt = match psbt.parse::<PartiallySignedTransaction>() {
            Ok(psbt) => psbt,
            Err(e) => {
                return Some(Err(log_error_ccstr(e)));
            }
        };
        if let Some(ref tx_sum) = *shared_tx_summary.read() {
            return Some(UITxDetails::try_from((&psbt, tx_sum)).map_err(log_error_ccstr));
        }
        if let Some(Ok(ref addr_set)) = *addresses_set.read() {
            return Some(UITxDetails::try_from((&psbt, addr_set)).map_err(log_error_ccstr));
        }

        Some(Err(CCStr::from(
            "We have the PSBT but nothing can provide the addresses ownership information",
        )))
    });
    use_context_provider(|| psbt_overview);

    use_drop(|| log::debug!("SpendTabs Dropped"));

    rsx! {
        div { class: "flex flex-col gap-6 border border-base-300 rounded-box",
            div { role: "tablist", class: "tabs tabs-xl tabs-border",
                SendTabLabel {
                    current_stage,
                    stage: SpendStage::Create,
                    disabled: cant_create(),
                    DrawSvg::<One> { size: Size8 }
                    "Create TX"
                }
                SendTabContent {
                    match cannot_create_reason() {
                        Some(reason) => rsx! {
                            {reason}
                        },
                        None => {
                            match spendtabs_type {
                                SpendTabsType::Owner => rsx! {
                                    create_tx::CreateOwnerTx { cant_sign }
                                },
                                SpendTabsType::Heir(heritage_id) => rsx! {
                                    create_tx::CreateHeirTx { heritage_id, cant_sign }
                                },
                            }
                        }
                    }
                }

                SendTabLabel {
                    current_stage,
                    stage: SpendStage::Sign,
                    disabled: cant_sign(),
                    DrawSvg::<Two> { size: Size8 }
                    "Sign TX"
                }
                SendTabContent {
                    match cannot_sign_reason() {
                        Some((reason, need_unlock)) => rsx! {
                            {reason}
                            if need_unlock {
                                div { class: "m-4", UnlockLocalKey::<T> {} }
                            }
                        },
                        None => rsx! {
                            sign_tx::SignTx::<T> { cant_broadcast }
                        },
                    }
                }

                SendTabLabel {
                    current_stage,
                    stage: SpendStage::Broadcast,
                    disabled: cant_broadcast(),
                    DrawSvg::<Three> { size: Size8 }
                    "Broadcast TX"
                }
                SendTabContent {
                    match cannot_broadcast_reason() {
                        Some(reason) => rsx! {
                            {reason}
                        },
                        None => rsx! {
                            broadcast_tx::BroadcastTx::<T> {}
                        },
                    }
                }
            }
        }
    }
}

#[component]
fn SendTabLabel(
    mut current_stage: Signal<SpendStage>,
    stage: SpendStage,
    children: Element,
    disabled: bool,
) -> Element {
    rsx! {
        label { class: "tab font-bold uppercase text-xl gap-2",
            input {
                r#type: "radio",
                name: "send_tabs",
                checked: current_stage() == stage,
                oninput: move |_| current_stage.set(stage),
            }
            {children}
            if disabled {
                DrawSvg::<Alert> { base_class: "fill-error" }
            }
            DrawSvg::<ChevronRight> { size: Size8 }
        }
    }
}
#[component]
fn SendTabContent(children: Element) -> Element {
    rsx! {
        div { class: "tab-content p-6", {children} }
    }
}
