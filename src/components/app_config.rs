use crate::prelude::*;

use btc_heritage_wallet::heritage_service_api_client::DeviceAuthorizationResponse;

use crate::{
    components::{
        modal::Modal,
        onboarding::MaybeOnPathHighlight,
        svg::{DrawSvg, Refresh},
    },
    Route,
};

/// AppConfig dropdown component that displays system status and provides access to configuration
#[component]
pub fn AppConfig() -> Element {
    log::debug!("AppConfig reload");

    // Application Config / Database
    let database_path = use_memo(move || {
        let config = state_management::APPLICATION_CONFIG.read();
        btc_heritage_wallet::Database::database_path(&config.datadir, config.network)
    });
    let current_bitcoin_network =
        use_memo(move || state_management::APPLICATION_CONFIG.read().network);

    let connected_user_name = use_memo(|| match &*state_management::SERVICE_STATUS.read() {
        Some(ServiceStatus::Connected(css)) => css.user_id.preferred_username.as_ref().to_owned(),
        _ => String::new(),
    });
    let connected_user_email = use_memo(|| match &*state_management::SERVICE_STATUS.read() {
        Some(ServiceStatus::Connected(css)) => css.user_id.email.as_ref().to_owned(),
        _ => String::new(),
    });

    // Blockchain Provider Status
    let blockchain_provider_height =
        use_memo(
            || match &*state_management::BLOCKCHAIN_PROVIDER_STATUS.read() {
                Some(BlockchainProviderStatus::Connected(h)) => {
                    format!("Block #{h}")
                }
                _ => String::new(),
            },
        );

    use_drop(|| log::debug!("AppConfig Dropped"));

    rsx! {
        AppConfigDropDown {
            head: rsx! {
                MaybeOnPathHighlight { steps: &[OnboardingStep::ClickConnectService, OnboardingStep::ConfigureBlockchainProvider],
                    div { class: "size-full content-center text-center border rounded-box", "Status" }
                }
            },
            div { class: "flex flex-col px-1",
                div { class: "flex flex-col px-3",
                    // Header
                    div { class: "text-lg font-bold mb-2 text-center", "Application Status" }

                    // Bitcoin Network
                    div { class: "text-sm",
                        div { class: "mb-1", "Bitcoin Network:" }
                        div { class: "font-bold", {format!("{:?}", current_bitcoin_network())} }
                    }

                    // Database Status
                    div { class: "text-sm",
                        div { class: "mb-1", "Database Path:" }
                        div { class: "font-bold", {database_path.read().to_string_lossy()} }
                    }

                    hr { class: "mt-4 h-px border-t border-solid border-gray-500" }
                    // Header
                    div { class: "text-lg font-bold mb-2 text-center", "Service Status" }

                    // Heritage Service Status
                    div { class: "text-sm",
                        div { class: "text-lg flex items-center justify-between",
                            if let Some(ServiceStatus::Connected(_)) = *state_management::SERVICE_STATUS.read() {
                                div {
                                    div { class: "text-sm font-thin", "Connected as:" }
                                    div { class: "text-3xl font-black text-center",
                                        {connected_user_name}
                                    }
                                    div { class: "text-sm font-medium text-center",
                                        {connected_user_email}
                                    }
                                }
                            } else {
                                div { "Not Connected" }
                            }
                            ServiceServiceStatus {}
                        }
                    }

                    div { class: "mt-4 w-1/2 mx-auto **:w-full", ServiceConnectButton {} }

                    hr { class: "mt-4 h-px border-t border-solid border-gray-500" }
                    // Header
                    div { class: "text-lg font-bold mb-2 text-center", "Blockchain Provider Status" }

                    // Blockchain Provider Status
                    div { class: "text-sm",
                        div { class: "text-lg flex items-center justify-between",
                            if let Some(BlockchainProviderStatus::Connected(_)) = *state_management::BLOCKCHAIN_PROVIDER_STATUS
                                .read()
                            {
                                div {
                                    div { class: "text-sm font-thin", "Connected at height:" }
                                    div { class: "text-xl font-black text-center",
                                        {blockchain_provider_height}
                                    }
                                }
                            } else {
                                div { "Not Connected" }
                            }
                            BlockchainProviderServiceStatus {}
                        }
                    }

                    hr { class: "mt-4 h-px border-t border-solid border-gray-500" }
                    // Header
                    div { class: "text-lg font-bold mb-2 text-center", "Ledger Status" }

                    // Ledger Status
                    LedgerServiceStatusWithDesc { class: "text-lg flex items-center justify-between" }
                }

                hr { class: "mt-6 mb-4 h-px border-t-2 border-solid border-gray-500" }
                // Configuration button
                div { class: "px-3",
                    MaybeOnPathHighlight { steps: &[OnboardingStep::ConfigureBlockchainProvider],
                        button {
                            class: "btn btn-secondary btn-block",
                            onclick: move |_| {
                                document::eval("document.activeElement.blur();");
                                navigator().push(Route::AppConfigView {});
                            },
                            "Open Configuration"
                        }
                    }
                }
            }
        }
    }
}

// On Windows, using the details/summary method automatically displays a " ▶ "/" ▼ " char on top of the button
// On Linux, using the tabindex/button leads to the dropdown weirdly not displaying when directly clicked but appearing
// if the app window lost and regain focus...
//
// I don't really know (or care) what black magic was written in WebKit for one or the other platform.
// For now I will just use what's working for each platform.
#[cfg(target_os = "windows")]
#[component]
pub fn AppConfigDropDown(head: Element, children: Element) -> Element {
    rsx! {
        div { class: "dropdown dropdown-end",
            button {
                tabindex: "0",
                class: "h-full p-2 content-center text-center cursor-pointer min-w-24",
                {head}
            }
            div {
                tabindex: "0",
                class: "dropdown-content bg-base-100 rounded-b-xl min-w-sm p-4 max-h-[85vh] overflow-y-scroll shadow-lg shadow-base-content/10",
                {children}
            }
        }
    }
}

#[cfg(not(target_os = "windows"))]
#[component]
pub fn AppConfigDropDown(head: Element, children: Element) -> Element {
    let mut mouse_outside = use_signal(|| true);
    let mut dropdown_open = use_signal(|| false);
    let mut dropdown_content = use_signal(|| None);

    rsx! {
        details {
            class: "dropdown dropdown-end",
            open: dropdown_open(),
            onmouseenter: move |_| mouse_outside.set(false),
            onmouseleave: move |_| mouse_outside.set(true),
            summary {
                class: "h-full p-2 content-center text-center cursor-pointer min-w-24",
                onfocusout: move |_| {
                    if mouse_outside() {
                        dropdown_open.set(false);
                    }
                },
                onclick: move |_| dropdown_open.set(!dropdown_open()),
                {head}
            }
            div {
                tabindex: "0",
                class: "dropdown-content bg-base-100 rounded-b-xl min-w-sm p-4 max-h-[85vh] overflow-y-scroll shadow-lg shadow-base-content/10",
                onmounted: move |e| dropdown_content.set(Some(e.data())),
                onfocusout: move |_| async move {
                    if mouse_outside() {
                        dropdown_open.set(false);
                    } else {
                        if let Some(dropdown_content) = dropdown_content() {
                            let _ = dropdown_content.set_focus(true).await;
                        }
                    }
                },
                {children}
            }
        }
    }
}

#[component]
pub fn ServiceConnectButton() -> Element {
    // Heritage Service Status
    let service_client_service = state_management::use_service_client_service();
    let mut dar_content: Signal<Option<(String, String)>> = use_signal(|| None);
    let mut connecting = use_signal(|| false);
    use_effect(move || *connecting.write() = dar_content.read().is_some());

    let connect_button_handler = move |_| async move {
        let read_guard = state_management::SERVICE_STATUS.read();
        if let Some(ref service_status) = *read_guard {
            match service_status {
                ServiceStatus::Connected(_) => {
                    // /!\ IMPORTANT
                    // Before calling the connection process, drop the ReadGuard on SERVICE_STATUS
                    // The crate::state_management::disconnect(...) will try to Write in SERVICE_STATUS at some point
                    drop(read_guard);
                    if let Err(e) = state_management::disconnect(service_client_service).await {
                        alert_error(e);
                    }
                }
                ServiceStatus::Disconnected => {
                    let (tx, rx) = tokio::sync::oneshot::channel();
                    spawn(async move {
                        let dar: DeviceAuthorizationResponse = rx.await.expect("chanel rx closed");
                        let verification_uri_complete =
                            format!("{}?user_code={}", dar.verification_uri, dar.user_code);
                        let human_formated_code =
                            format!("{}-{}", &dar.user_code[..4], &dar.user_code[4..]);

                        _ = open::that_in_background(&verification_uri_complete);

                        *dar_content.write() =
                            Some((verification_uri_complete, human_formated_code));
                    });

                    // /!\ IMPORTANT
                    // Before calling the connection process, drop the ReadGuard on SERVICE_STATUS
                    // The crate::state_management::connect(...) will try to Write in SERVICE_STATUS at some point
                    drop(read_guard);

                    if let Err(e) =
                        state_management::connect(service_client_service, move |dar| async move {
                            tx.send(dar).expect("chanel tx closed");
                            Ok(())
                        })
                        .await
                    {
                        alert_error(e);
                    }
                    *dar_content.write() = None;
                }
            }
        }
    };

    let memo_is_connected = use_memo(move || {
        matches!(
            *state_management::SERVICE_STATUS.read(),
            Some(ServiceStatus::Connected(_))
        )
    });

    rsx! {
        // Heritage Service disconnect button
        MaybeHighlight {
            step: OnboardingStep::ClickConnectService,
            progress: MaybeHighlightProgressType::Signal(memo_is_connected.into()),
            button {
                class: "btn btn-outline",
                onclick: connect_button_handler,
                disabled: state_management::SERVICE_STATUS.read().is_none(),
                if memo_is_connected() {
                    "Disconnect"
                } else {
                    "Connect"
                }
                Modal { is_open: connecting, persistent: true,
                    div { class: "p-4 text-center",
                        h1 { class: "text-5xl font-black", "Connect to the Service" }
                        div { class: "my-4 h-px border-t border-solid border-gray-500" }

                        p { class: "pt-4", "Please browse to the Heritage service website:" }
                        div { class: "text-xl text-secondary font-bold select-all",
                            {dar_content.read().as_ref().map(|v| v.0.as_str()).unwrap_or_default()}
                        }
                        p {
                            "in order to approve the connection (should have been open in your browser)"
                        }
                        p { class: "py-4", "Verify that the code displayed is:" }
                        div { class: "p-2 mb-4 size-fit mx-auto text-6xl text-primary font-black rounded border-solid border-2 border-base-content",
                            {dar_content.read().as_ref().map(|v| v.1.as_str()).unwrap_or_default()}
                        }
                    }
                }
            }
        }
    }
}

#[component]
fn ServiceStatusComp(status: Option<bool>, onclick: EventHandler<MouseEvent>) -> Element {
    rsx! {
        if let Some(status) = status {
            div { class: "group cursor-pointer",
                div { onclick,
                    DrawSvg::<Refresh> { base_class: "group-hover:inline-flex hidden fill-current" }
                }
                div {
                    class: "group-hover:hidden status w-6 h-6 rounded-full",
                    class: if status { "status-success" } else { "status-error" },
                }
            }
        } else {
            span { class: "loading loading-spinner loading-md mt-1" }
        }
    }
}

#[component]
pub fn ServiceServiceStatus() -> Element {
    let service_client_service = state_management::use_service_client_service();
    let service_connected = use_memo(|| {
        state_management::SERVICE_STATUS.lmap(|service_status| match service_status {
            ServiceStatus::Connected(_) => true,
            ServiceStatus::Disconnected => false,
        })
    });
    rsx! {
        ServiceStatusComp {
            status: service_connected(),
            onclick: move |_| state_management::refresh_service_status(service_client_service),
        }
    }
}

#[component]
pub fn BlockchainProviderServiceStatus() -> Element {
    let blockchain_provider_service = state_management::use_blockchain_provider_service();
    let blockchain_provider_connected = use_memo(|| {
        state_management::BLOCKCHAIN_PROVIDER_STATUS
            .read()
            .as_ref()
            .map(
                |blockchain_provider_status| match blockchain_provider_status {
                    BlockchainProviderStatus::Connected(_) => true,
                    BlockchainProviderStatus::Disconnected => false,
                },
            )
    });
    rsx! {
        ServiceStatusComp {
            status: blockchain_provider_connected(),
            onclick: move |_| state_management::refresh_blockchain_provider_status(
                blockchain_provider_service,
            ),
        }
    }
}

#[component]
pub fn LedgerServiceStatus() -> Element {
    let ledger_connected = use_memo(|| state_management::ledger_is_ready().is_some());
    rsx! {
        ServiceStatusComp {
            status: ledger_connected(),
            onclick: move |_| state_management::refresh_ledger_status(),
        }
    }
}

#[component]
pub fn LedgerServiceStatusWithDesc(class: &'static str) -> Element {
    rsx! {
        div { class,
            match state_management::LEDGER_STATUS() {
                Some(LedgerStatus::Ready(ledger_fingerprint)) => {
                    rsx! {
                        div { class: "flex flex-col",
                            div { class: "text-sm font-thin", "Connected with fingerprint:" }
                            div { class: "text-xl font-black text-center", {ledger_fingerprint.to_string()} }
                        }
                    }
                }
                Some(LedgerStatus::WrongNetwork) => {
                    rsx! {
                        div { "Wrong Network" }
                    }
                }
                Some(LedgerStatus::WrongApp) => {
                    rsx! {
                        div { "Wrong App" }
                    }
                }
                Some(LedgerStatus::NotReady) | None => {
                    rsx! {
                        div { "Not Connected" }
                    }
                }
            }
            LedgerServiceStatus {}
        }
    }
}
