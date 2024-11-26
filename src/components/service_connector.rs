use dioxus::prelude::*;

use crate::{
    clients::{connect, disconnect, get_userid, is_connected, UserId},
    components::Modal,
};

#[component]
pub fn ServiceConnector() -> Element {
    let mut user_id: Signal<Option<UserId>, SyncStorage> = use_signal_sync(|| get_userid());

    let mut connected = use_signal(|| false);
    use_future(move || async move {
        *connected.write() = tokio::task::spawn_blocking(|| is_connected())
            .await
            .unwrap();
    });

    let mut dar_content: Signal<Option<(String, String)>, SyncStorage> = use_signal_sync(|| None);

    let mut connecting = use_signal(|| false);
    use_effect(move || *connecting.write() = dar_content.read().is_some());

    let connect = move |_| {
        spawn(async move {
            *connected.write() = tokio::task::spawn_blocking(move || {
                match connect(|dar| {
                    let verification_uri_complete =
                        format!("{}?user_code={}", dar.verification_uri, dar.user_code);
                    let human_formated_code =
                        format!("{}-{}", &dar.user_code[..4], &dar.user_code[4..]);

                    _ = open::that(&verification_uri_complete);

                    *dar_content.write() = Some((verification_uri_complete, human_formated_code));
                    Ok(())
                }) {
                    Ok(uid) => {
                        user_id.write().replace(uid);
                    }
                    Err(e) => {
                        log::error!("{e}");
                    }
                };
                is_connected()
            })
            .await
            .unwrap();
            *dar_content.write() = None;
        });
    };
    let disconnect = move |_| {
        spawn(async move {
            tokio::task::spawn_blocking(|| disconnect())
                .await
                .unwrap()
                .unwrap();
            *connected.write() = false;
            user_id.write().take();
        });
    };

    let (color, text) = if *connected.read() {
        ("bg-green-700", "Connected")
    } else {
        ("bg-red-700", "Disconnected")
    };

    rsx! {
        div {
            div { class: "dropdown h-full dropdown-end",
                div {
                    tabindex: "0",
                    role: "button",
                    class: "h-full px-2 content-center flex flex-row hover:cursor-pointer",
                    span { class: "h-6 w-6 my-auto inline-block rounded-full {color}" }
                    div { class: "w-28 ml-2 my-auto",
                        div { class: "font-thin text-xs", "Service Status:" }
                        div { "{text}" }
                    }
                }
                ul {
                    tabindex: "0",
                    class: "menu dropdown-content bg-base-100 z-[1] min-w-52 rounded-b-xl p-4 shadow-lg shadow-base-content/10",
                    if *connected.read() {
                        li { class: "text-sm font-thin", "Connected as:" }
                        li { class: "text-3xl font-black text-center",
                            {user_id.read().as_ref().map(|uid|uid.preferred_username.as_ref())}
                        }
                        li { class: "text-sm font-medium text-center",
                            {user_id.read().as_ref().map(|uid|uid.email.as_ref())}
                        }
                        div { class: "my-2 h-px border-t border-solid border-gray-500" }
                        li {
                            button {
                                class: "btn btn-ghost hover:bg-primary/10 uppercase",
                                onclick: disconnect,
                                "Disconnect"
                            }
                        }
                    } else {
                        li {
                            button {
                                class: "btn btn-ghost hover:bg-primary/10 uppercase",
                                onclick: connect,
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
                        {dar_content
                            .read()
                            .as_ref().map(|v|v.0.as_str()).unwrap_or_default()}
                    }
                    p { "in order to approve the connection (should have been open in your browser)" }
                    p { class: "py-4", "Verify that the code displayed is:" }
                    div { class: "p-2 mb-4 size-fit mx-auto text-6xl text-primary font-black rounded border-solid border-2 border-base-content",
                        {dar_content
                            .read()
                            .as_ref().map(|v|v.1.as_str()).unwrap_or_default()}
                    }
                }
            }
        }
    }
}
