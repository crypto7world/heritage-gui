use crate::prelude::*;

use crate::components::svg::{Alert, DrawSvg, InfoCircleOutline};

#[component]
pub fn AlertDeleteKeyProvider(
    acknowledge_private_keys: Signal<bool>,
    children: Element,
) -> Element {
    rsx! {
        AlertDeleteAck {
            alert_style: AlertDeleteAckStyle::Error,
            acknowledge: acknowledge_private_keys,
            acknowledgment: rsx! {
                span { class: "font-black uppercase", "I confirm" }
                " that I have "
                span { class: "font-black uppercase", "backed up the mnemonic phrase" }
                "."
            },
            h3 { class: "font-bold", "Private Key Deletion Warning" }
            {children}
        }
    }
}

#[component]
pub fn AlertDeleteLocalWallet(acknowledge_descriptors: Signal<bool>, children: Element) -> Element {
    rsx! {
        AlertDeleteAck {
            alert_style: AlertDeleteAckStyle::Error,
            acknowledge: acknowledge_descriptors,
            acknowledgment: rsx! {
                span { class: "font-black uppercase", "I confirm" }
                " that I have "
                span { class: "font-black uppercase", "backed up the wallet descriptors" }
                "."
            },
            h3 { class: "font-bold", "Local Wallet Deletion Warning" }
            {children}
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum AlertDeleteAckStyle {
    Error,
    Warning,
    Info,
}
#[component]
pub fn AlertDeleteAck(
    #[props(default = AlertDeleteAckStyle::Info)] alert_style: AlertDeleteAckStyle,
    acknowledge: Signal<bool>,
    acknowledgment: Element,
    children: Element,
) -> Element {
    let alert_class = match alert_style {
        AlertDeleteAckStyle::Error => "alert-error",
        AlertDeleteAckStyle::Warning => "alert-warning",
        AlertDeleteAckStyle::Info => "alert-info",
    };
    let bg_class = match alert_style {
        AlertDeleteAckStyle::Error => "bg-error/10",
        AlertDeleteAckStyle::Warning => "bg-warning/10",
        AlertDeleteAckStyle::Info => "bg-info/10",
    };
    let checkbox_class = match alert_style {
        AlertDeleteAckStyle::Error => "checkbox-error",
        AlertDeleteAckStyle::Warning => "checkbox-warning",
        AlertDeleteAckStyle::Info => "checkbox-info",
    };
    let icon = match alert_style {
        AlertDeleteAckStyle::Error => rsx! {
            DrawSvg::<Alert> {}
        },
        AlertDeleteAckStyle::Warning => rsx! {
            DrawSvg::<InfoCircleOutline> {}
        },
        AlertDeleteAckStyle::Info => rsx! {
            DrawSvg::<InfoCircleOutline> {}
        },
    };

    rsx! {
        div { class: "alert {alert_class}",
            {icon}
            div { {children} }
        }
        MaybeHighlight {
            step: OnboardingStep::CheckConfirmStripHeirSeed,
            context_filter: consume_onboarding_context(),
            progress: MaybeHighlightProgressType::Signal(acknowledge.into()),
            label { class: "label justify-start gap-2 {bg_class} p-4 rounded-lg",
                input {
                    r#type: "checkbox",
                    class: "checkbox {checkbox_class}",
                    checked: acknowledge(),
                    onchange: move |evt| *acknowledge.write() = evt.checked(),
                }
                span { class: "font-semibold", {acknowledgment} }
            }
        }
    }
}
