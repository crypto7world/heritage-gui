use btc_heritage_wallet::heritage_service_api_client::DeviceAuthorizationResponse;
use dioxus::prelude::*;

use crate::{components::Modal, state_management::CONNECTED_USER};

#[component]
pub fn ServiceConnector() -> Element {
    log::debug!("ServiceConnector reload");

    let connected = use_memo(|| CONNECTED_USER.read().is_some());

    let mut dar_content: Signal<Option<(String, String)>> = use_signal(|| None);
    let mut connecting = use_signal(|| false);
    use_effect(move || *connecting.write() = dar_content.read().is_some());

    let connect_handler = move |_| async move {
        let (tx, rx) = tokio::sync::oneshot::channel();
        spawn(async move {
            let dar: DeviceAuthorizationResponse = rx.await.expect("chanel rx closed");
            let verification_uri_complete =
                format!("{}?user_code={}", dar.verification_uri, dar.user_code);
            let human_formated_code = format!("{}-{}", &dar.user_code[..4], &dar.user_code[4..]);

            _ = open::that_in_background(&verification_uri_complete);

            *dar_content.write() = Some((verification_uri_complete, human_formated_code));
        });
        crate::state_management::connect(move |dar| async move {
            tx.send(dar).expect("chanel tx closed");
            Ok(())
        })
        .await
        .ok();
        *dar_content.write() = None;
    };

    let (color, text) = if connected() {
        ("bg-success", "Connected")
    } else {
        ("bg-error", "Disconnected")
    };

    use_drop(|| log::debug!("ServiceConnector Dropped"));

    rsx! {
        div {
            div { class: "dropdown h-full dropdown-end",
                div {
                    tabindex: "0",
                    role: "button",
                    class: "h-full px-2 content-center flex flex-row cursor-pointer",
                    span { class: "h-6 w-6 my-auto inline-block rounded-full {color}" }
                    div { class: "w-28 ml-2 my-auto",
                        div { class: "font-light text-xs", "Service Status:" }
                        div { {text} }
                    }
                }
                ul {
                    tabindex: "0",
                    class: "menu dropdown-content bg-base-100 z-[1] min-w-52 rounded-b-xl p-4 shadow-lg shadow-base-content/10",
                    if connected() {
                        li { class: "text-sm font-thin", "Connected as:" }
                        li { class: "text-3xl font-black text-center",
                            {CONNECTED_USER.read().as_ref().map(|uid| uid.preferred_username.as_ref())}
                        }
                        li { class: "text-sm font-medium text-center",
                            {CONNECTED_USER.read().as_ref().map(|uid| uid.email.as_ref())}
                        }
                        div { class: "my-2 h-px border-t border-solid border-gray-500" }
                        li {
                            button {
                                class: "btn btn-ghost hover:bg-primary/10 uppercase",
                                onclick: |_| async {
                                    crate::state_management::disconnect().await.ok();
                                },
                                "Disconnect"
                            }
                        }
                    } else {
                        li {
                            button {
                                class: "btn btn-ghost hover:bg-primary/10 uppercase",
                                onclick: connect_handler,
                                "Connect"
                            }
                        }
                    }
                }
            }

            Modal { is_open: connecting, persistent: true,
                div { class: "p-4 text-center",
                    h1 { class: "text-5xl font-black", "Connect to the Service" }
                    div { class: "my-4 h-px border-t border-solid border-gray-500" }

                    p { class: "pt-4", "Please browse to the Heritage service website:" }
                    div { class: "text-xl text-secondary font-bold select-all",
                        {dar_content.read().as_ref().map(|v| v.0.as_str()).unwrap_or_default()}
                    }
                    p { "in order to approve the connection (should have been open in your browser)" }
                    p { class: "py-4", "Verify that the code displayed is:" }
                    div { class: "p-2 mb-4 size-fit mx-auto text-6xl text-primary font-black rounded border-solid border-2 border-base-content",
                        {dar_content.read().as_ref().map(|v| v.1.as_str()).unwrap_or_default()}
                    }
                }
            }
        }
    }
}
