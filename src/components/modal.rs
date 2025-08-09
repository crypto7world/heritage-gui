use crate::prelude::*;

use crate::components::{
    misc::Teleport,
    svg::{Cancel, Close, DrawSvg},
};

#[component]
pub fn Modal(
    is_open: Signal<bool>,
    #[props(default = false)] persistent: bool,
    #[props(default = false)] higher_modal: bool,
    children: Element,
) -> Element {
    let mut classes = use_signal(|| String::new());

    rsx! {
        Teleport {
            input {
                r#type: "checkbox",
                name: "modal-toggle",
                class: "modal-toggle",
                tabindex: "-1",
                checked: is_open(),
            }
            div {
                class: "modal",
                class: if higher_modal { "z-45" } else { "z-40" },
                role: "dialog",
                onclick: move |_| {
                    if persistent {
                        spawn(async move {
                            let orig_len = classes.read().len();
                            *classes.write() += "animate-scalebump";
                            tokio::time::sleep(tokio::time::Duration::from_millis(200)).await;
                            classes.write().truncate(orig_len);
                        });
                    } else {
                        *is_open.write() = false;
                    }
                },
                div {
                    class: "modal-box max-w-max max-h-[calc(100vh-4rem)] p-0 {classes}",
                    onclick: move |event| {
                        event.stop_propagation();
                    },
                    div { class: "p-6 w-fit", {children} }
                }
            }
        }
    }
}

#[component]
pub fn ConfigModal(mut is_open: Signal<bool>, title: &'static str, children: Element) -> Element {
    rsx! {
        Modal { is_open, persistent: true,
            ModalHeader { is_open, title }
            {children}
        }
    }
}

#[component]
pub fn CloseModalButton(mut signal: Signal<bool>) -> Element {
    rsx! {
        button {
            class: "btn btn-outline btn-primary",
            onclick: move |_| *signal.write() = false,
            DrawSvg::<Cancel> {}
            "Cancel"
        }
    }
}

#[component]
pub fn InfoModal(mut is_open: Signal<bool>, title: &'static str, children: Element) -> Element {
    rsx! {
        Modal { is_open, persistent: false,
            ModalHeader { is_open, title }
            {children}
        }
    }
}
#[component]
fn ModalHeader(mut is_open: Signal<bool>, title: &'static str) -> Element {
    rsx! {
        div { class: "flex flex-row justify-between gap-4 mb-4",
            h2 { class: "text-2xl font-bold mb-4", {title} }

            MaybeHighlight {
                step: OnboardingStep::CloseHeirShowMnemonic,
                context_filter: consume_onboarding_context(),
                button {
                    class: "btn btn-circle btn-outline btn-primary btn-sm",
                    onclick: move |_| *is_open.write() = false,
                    DrawSvg::<Close> {}
                }
            }
        }
    }
}
