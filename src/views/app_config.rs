use crate::prelude::*;

use std::sync::Arc;

use btc_heritage_wallet::{
    heritage_service_api_client::HeritageServiceConfig,
    online_wallet::{AuthConfig, BlockchainProviderConfig},
};

use crate::{
    components::{
        app_config::{
            BlockchainProviderServiceStatus, LedgerServiceStatus, ServiceConnectButton,
            ServiceServiceStatus,
        },
        svg::{AlertOutline, DrawSvg, InfoCircleOutline},
    },
    utils::CCStr,
};

/// Application configuration view component
#[component]
pub fn AppConfigView() -> Element {
    log::debug!("AppConfigView reload");

    use_drop(|| log::debug!("AppConfigView Dropped"));

    rsx! {
        super::TitledView {
            title: CCStr::from("Application Configuration"),
            subtitle: CCStr::from("Manage external provider connections and settings."),
            div { class: "container mx-auto px-8 space-y-8",
                // Bitcoin Network Section (Read-only)
                ApplicationConfigSection {}

                // Application onboarding section
                OnboardingConfigSection {}

                // Heritage Service Configuration Section
                HeritageServiceConfigSection {}

                // Blockchain Provider Configuration Section
                BlockchainProviderConfigSection {}

                // Ledger Configuration Section
                LedgerConfigSection {}
            }
        }
    }
}

/// Application configuration section
#[component]
fn ApplicationConfigSection() -> Element {
    let database_service = state_management::use_database_service();

    let mut new_datadir_str = use_signal(String::new);
    let new_datadir = use_memo(move || std::path::PathBuf::from(new_datadir_str.read().as_str()));
    let mut new_network_str = use_signal(|| "bitcoin".to_owned());
    let new_network =
        use_memo(move || new_network_str.read().parse().expect("options are correct"));
    let mut updating = use_signal(|| false);

    let mut update_form_from_config = move |config: &ApplicationConfig| {
        *new_datadir_str.write() = config.datadir.to_string_lossy().to_string();
        *new_network_str.write() = config.network.to_string();
    };

    // Initialize the input with current value
    use_effect(move || update_form_from_config(&*state_management::APPLICATION_CONFIG.read()));

    let current_network = use_memo(move || {
        state_management::APPLICATION_CONFIG
            .read()
            .network
            .to_string()
    });

    let current_dbpath = use_memo(move || {
        state_management::APPLICATION_CONFIG
            .read()
            .datadir
            .to_string_lossy()
            .to_string()
    });

    let has_changes = use_memo(move || {
        let current_config = state_management::APPLICATION_CONFIG.read();
        !new_datadir_str.read().is_empty() && new_datadir.read().as_path() != current_config.datadir
            || new_network() != current_config.network
    });

    let is_default = use_memo(move || {
        let default_config = ApplicationConfig::default();
        new_datadir.read().as_path() == default_config.datadir
            && new_network() == default_config.network
    });

    let database_file_path = use_memo(move || {
        let db_path = btc_heritage_wallet::Database::database_path(
            new_datadir.read().as_path(),
            new_network(),
        );
        db_path.to_string_lossy().to_string()
    });

    let update_handler = move |_| async move {
        *updating.write() = true;
        let new_config = ApplicationConfig {
            network: new_network(),
            datadir: new_datadir(),
        };

        match state_management::update_application_config(database_service, new_config).await {
            Ok(()) => {
                log::info!("Database directory updated successfully");
                alert_success("Database directory updated successfully");
            }
            Err(e) => {
                log::error!("Failed to update database directory: {e}");
                alert_error(format!("Failed to update database directory: {e}"));
            }
        }
        *updating.write() = false;
    };

    let reset_default_handler = move |_| {
        let default_config = ApplicationConfig::default();
        update_form_from_config(&default_config)
    };

    let reset_current_handler =
        move |_| update_form_from_config(&*state_management::APPLICATION_CONFIG.read());

    rsx! {
        div { class: "card bg-base-200 shadow-xl",
            div { class: "card-body",
                h2 { class: "card-title", "Application Configuration" }
                p { class: "text-sm text-gray-600 mb-4",
                    "Configure the location where wallet data is stored and the Bitcoin network. Changes take effect immediately and will close the current database."
                }

                div { class: "flex flex-row gap-4",
                    fieldset { class: "fieldset w-32",
                        legend { class: "fieldset-legend", "Bitcoin Network" }
                        select {
                            class: "select select-bordered",
                            value: new_network_str(),
                            disabled: *updating.read(),
                            onchange: move |event| *new_network_str.write() = event.value(),
                            option { value: "bitcoin", "Bitcoin" }
                            option { value: "testnet", "Testnet" }
                        }
                        div { class: "label", "Current: {current_network.read()}" }
                    }
                    fieldset { class: "fieldset grow",
                        legend { class: "fieldset-legend", "Data Directory" }
                        input {
                            r#type: "text",
                            class: "input",
                            value: "{new_datadir_str}",
                            disabled: *updating.read(),
                            oninput: move |event| *new_datadir_str.write() = event.value(),
                            placeholder: "Enter database directory path...",
                        }
                        div { class: "label", "Current: {current_dbpath.read()}" }
                    }
                }

                if has_changes() {
                    div { class: "alert alert-warning mt-4",
                        div { class: "flex items-center",
                            DrawSvg::<AlertOutline> {}
                            div {
                                div { class: "font-medium", "Warning:" }
                                div { class: "text-sm mt-1",
                                    "Changing the database directory or the network will close the current database and open the new location. Make sure the path is accessible and has appropriate permissions."
                                }
                                div { class: "text-sm mt-1",
                                    "New database path: "
                                    span { class: "text-base font-bold font-mono",
                                        {database_file_path}
                                    }
                                }
                            }
                        }
                    }
                }

                div { class: "card-actions justify-end mt-6",
                    button {
                        class: "btn btn-outline",
                        disabled: *updating.read() || is_default(),
                        onclick: reset_default_handler,
                        "Reset to Defaults"
                    }
                    button {
                        class: "btn btn-outline",
                        disabled: *updating.read() || !has_changes(),
                        onclick: reset_current_handler,
                        "Reset to Current"
                    }
                    button {
                        class: "btn btn-primary",
                        disabled: updating() || !has_changes(),
                        onclick: update_handler,
                        if updating() {
                            span { class: "loading loading-spinner loading-sm mr-2" }
                            "Updating..."
                        } else {
                            "Update Configuration"
                        }
                    }
                }
            }
        }
    }
}

/// Heritage Service configuration section
#[component]
fn HeritageServiceConfigSection() -> Element {
    let service_client_service = state_management::use_service_client_service();
    let mut resource_service_client_config = use_resource(move || async move {
        state_management::get_service_config(service_client_service).await
    });

    let mut service_api_url = use_signal(String::new);
    let mut auth_url = use_signal(String::new);
    let mut auth_client_id = use_signal(String::new);
    let mut updating = use_signal(|| false);

    let mut update_form_from_config = move |config: &HeritageServiceConfig| {
        *service_api_url.write() = config.service_api_url.to_string();
        *auth_url.write() = config.auth_url.to_string();
        *auth_client_id.write() = config.auth_client_id.to_string();
    };

    // Initialize inputs with current values
    use_effect(move || {
        if let Some(config) = resource_service_client_config.read().as_ref() {
            update_form_from_config(config)
        }
    });

    let has_changes = use_memo(move || {
        if let Some(config) = resource_service_client_config.read().as_ref() {
            !service_api_url.read().is_empty()
                && service_api_url.read().as_str() != config.service_api_url.as_ref()
                || !auth_url.read().is_empty()
                    && auth_url.read().as_str() != config.auth_url.as_ref()
                || !auth_client_id.read().is_empty()
                    && auth_client_id.read().as_str() != config.auth_client_id.as_ref()
        } else {
            false
        }
    });

    let is_default = use_memo(move || {
        let default_config = HeritageServiceConfig::default();
        service_api_url.read().as_str() == default_config.service_api_url.as_ref()
            && auth_url.read().as_str() == default_config.auth_url.as_ref()
            && auth_client_id.read().as_str() == default_config.auth_client_id.as_ref()
    });

    let update_handler = move |_| async move {
        *updating.write() = true;
        let new_config = HeritageServiceConfig {
            service_api_url: Arc::from(service_api_url.read().as_str()),
            auth_url: Arc::from(auth_url.read().as_str()),
            auth_client_id: Arc::from(auth_client_id.read().as_str()),
        };

        state_management::update_service_config(service_client_service, new_config);
        log::info!("Heritage Service configuration updated successfully");
        alert_success("Heritage Service configuration updated successfully");
        resource_service_client_config.restart();
        *updating.write() = false;
    };

    let reset_default_handler = move |_| {
        let default_config = HeritageServiceConfig::default();
        update_form_from_config(&default_config)
    };

    let reset_current_handler = move |_| {
        if let Some(config) = resource_service_client_config.read().as_ref() {
            update_form_from_config(config)
        }
    };

    rsx! {
        div { class: "card bg-base-200 shadow-xl",
            div { class: "card-body",
                h2 { class: "card-title",
                    "Heritage Service Configuration"
                    ServiceServiceStatus {}
                }
                p { class: "text-sm text-gray-600 mb-4",
                    "Configure connection to the Heritage service for wallet synchronization and inheritance management."
                }

                div { class: "flex flex-row flex-wrap gap-4",
                    fieldset { class: "w-lg fieldset border-base-content rounded-box border p-4",
                        legend { class: "fieldset-legend", "Service API" }
                        fieldset { class: "fieldset w-full",
                            legend { class: "fieldset-legend", "URL" }
                            input {
                                r#type: "url",
                                class: "input w-full",
                                value: service_api_url(),
                                disabled: *updating.read(),
                                oninput: move |event| *service_api_url.write() = event.value(),
                                placeholder: "https://api.btcherit.com/v1",
                            }
                            if let Some(config) = resource_service_client_config.read().as_ref() {
                                div { class: "label", "Current: {config.service_api_url}" }
                            }
                        }
                    }
                    fieldset { class: "w-lg fieldset border-base-content rounded-box border p-4",
                        legend { class: "fieldset-legend", "Authentication" }

                        fieldset { class: "fieldset w-full",
                            legend { class: "fieldset-legend", "URL" }
                            input {
                                r#type: "url",
                                class: "input w-full",
                                value: auth_url(),
                                disabled: *updating.read(),
                                oninput: move |event| *auth_url.write() = event.value(),
                                placeholder: "https://device.crypto7.world/token",
                            }
                            if let Some(config) = resource_service_client_config.read().as_ref() {
                                div { class: "label", "Current: {config.auth_url}" }
                            }
                        }

                        fieldset { class: "fieldset w-full",
                            legend { class: "fieldset-legend", "Client ID" }
                            input {
                                r#type: "text",
                                class: "input w-full",
                                value: auth_client_id(),
                                disabled: *updating.read(),
                                oninput: move |event| *auth_client_id.write() = event.value(),
                                placeholder: "cda6031ca00d09d66c2b632448eb8fef",
                            }
                            if let Some(config) = resource_service_client_config.read().as_ref() {
                                div { class: "label", "Current: {config.auth_client_id}" }
                            }
                        }
                    }
                }
                div { class: "card-actions justify-end mt-6",
                    ServiceConnectButton {}
                    div { class: "basis-1/12" }
                    button {
                        class: "btn btn-outline",
                        disabled: *updating.read() || is_default(),
                        onclick: reset_default_handler,
                        "Reset to Defaults"
                    }
                    button {
                        class: "btn btn-outline",
                        disabled: *updating.read() || !has_changes()
                            || resource_service_client_config.read().is_none(),
                        onclick: reset_current_handler,
                        "Reset to Current"
                    }
                    button {
                        class: "btn btn-primary",
                        disabled: updating() || !has_changes(),
                        onclick: update_handler,
                        if updating() {
                            span { class: "loading loading-spinner loading-sm mr-2" }
                            "Updating..."
                        } else {
                            "Update Configuration"
                        }
                    }
                }
            }
        }
    }
}

/// Blockchain provider configuration section
#[component]
fn BlockchainProviderConfigSection() -> Element {
    let blockchain_provider_service = state_management::use_blockchain_provider_service();
    let mut resource_blockchain_provider_config = use_resource(move || async move {
        state_management::get_blockchain_provider_config(blockchain_provider_service).await
    });

    let mut provider_type = use_signal(|| "electrum".to_string());
    let current_provider_type = use_memo(move || {
        resource_blockchain_provider_config
            .read()
            .as_ref()
            .map(|config| match config {
                BlockchainProviderConfig::BitcoinCore { .. } => "Bitcoin Core RPC",
                BlockchainProviderConfig::Electrum { .. } => "Electrum",
            })
    });

    let mut electrum_url = use_signal(String::new);
    let mut bitcoincore_url = use_signal(String::new);

    let mut auth_type = use_signal(|| "cookie".to_string());
    let current_auth_type = use_memo(move || {
        if let Some(BlockchainProviderConfig::BitcoinCore { auth, .. }) =
            resource_blockchain_provider_config.read().as_ref()
        {
            Some(match auth {
                AuthConfig::Cookie { .. } => "Cookie File",
                AuthConfig::UserPass { .. } => "Username/Password",
            })
        } else {
            None
        }
    });
    let mut cookie_path = use_signal(String::new);
    let mut username = use_signal(String::new);
    let mut password = use_signal(String::new);

    let mut updating = use_signal(|| false);

    let mut update_form_from_config = move |config: &BlockchainProviderConfig| match config {
        BlockchainProviderConfig::Electrum { url } => {
            *provider_type.write() = "electrum".to_string();
            *electrum_url.write() = url.to_string();
        }
        BlockchainProviderConfig::BitcoinCore { url, auth } => {
            *provider_type.write() = "bitcoincore".to_string();
            *bitcoincore_url.write() = url.to_string();
            match auth {
                AuthConfig::Cookie { file } => {
                    *auth_type.write() = "cookie".to_string();
                    *cookie_path.write() = file.to_string();
                }
                AuthConfig::UserPass {
                    username: u,
                    password: p,
                } => {
                    *auth_type.write() = "userpass".to_string();
                    *username.write() = u.to_string();
                    *password.write() = p.to_string();
                }
            }
        }
    };

    // Initialize inputs with current values
    use_effect(move || {
        if let Some(config) = resource_blockchain_provider_config.read().as_ref() {
            update_form_from_config(config);
        }
    });

    let is_valid = use_memo(move || match provider_type().as_ref() {
        "electrum" => !electrum_url.read().is_empty(),
        "bitcoincore" => {
            !bitcoincore_url.read().is_empty()
                && match auth_type.read().as_str() {
                    "cookie" => !cookie_path.read().is_empty(),
                    "userpass" => !username.read().is_empty() && !password.read().is_empty(),
                    _ => false,
                }
        }
        _ => false,
    });

    let has_changes = use_memo(move || {
        if let Some(config) = resource_blockchain_provider_config.read().as_ref() {
            is_valid()
                && match config {
                    BlockchainProviderConfig::Electrum { url } => {
                        provider_type.read().as_str() != "electrum"
                            || electrum_url.read().as_str() != url.as_ref()
                    }
                    BlockchainProviderConfig::BitcoinCore { url, auth } => {
                        provider_type.read().as_str() != "bitcoincore"
                            || bitcoincore_url.read().as_str() != url.as_ref()
                            || match auth {
                                AuthConfig::Cookie { file } => {
                                    auth_type.read().as_str() != "cookie"
                                        || cookie_path.read().as_str() != file.as_ref()
                                }
                                AuthConfig::UserPass {
                                    username: u,
                                    password: p,
                                } => {
                                    auth_type.read().as_str() != "userpass"
                                        || username.read().as_str() != u.as_ref()
                                        || password.read().as_str() != p.as_ref()
                                }
                            }
                    }
                }
        } else {
            false
        }
    });

    let update_handler = move |_| async move {
        *updating.write() = true;

        let new_config = match provider_type.read().as_str() {
            "electrum" => BlockchainProviderConfig::Electrum {
                url: Arc::from(electrum_url.read().as_ref()),
            },
            "bitcoincore" => {
                let auth = match auth_type.read().as_str() {
                    "cookie" => AuthConfig::Cookie {
                        file: Arc::from(cookie_path.read().as_ref()),
                    },
                    "userpass" => AuthConfig::UserPass {
                        username: Arc::from(username.read().as_ref()),
                        password: Arc::from(password.read().as_ref()),
                    },
                    _ => AuthConfig::Cookie {
                        file: Arc::from(cookie_path.read().as_ref()),
                    },
                };
                BlockchainProviderConfig::BitcoinCore {
                    url: Arc::from(bitcoincore_url.read().as_ref()),
                    auth,
                }
            }
            _ => BlockchainProviderConfig::Electrum {
                url: Arc::from(electrum_url.read().as_ref()),
            },
        };
        state_management::update_blockchain_provider_config(
            blockchain_provider_service,
            new_config,
        );
        log::info!("Blockchain provider configuration updated successfully");
        alert_success("Blockchain provider configuration updated successfully");
        resource_blockchain_provider_config.restart();
        *updating.write() = false;
    };

    let is_default = use_memo(move || {
        let default_config = BlockchainProviderConfig::default();
        match &default_config {
            BlockchainProviderConfig::Electrum { url } => {
                provider_type.read().as_str() == "electrum"
                    && electrum_url.read().as_str() == url.as_ref()
            }
            BlockchainProviderConfig::BitcoinCore { url, auth } => {
                provider_type.read().as_str() == "bitcoincore"
                    && bitcoincore_url.read().as_str() == url.as_ref()
                    && match auth {
                        AuthConfig::Cookie { file } => {
                            auth_type.read().as_str() == "cookie"
                                && cookie_path.read().as_str() == file.as_ref()
                        }
                        AuthConfig::UserPass {
                            username: u,
                            password: p,
                        } => {
                            auth_type.read().as_str() == "userpass"
                                && username.read().as_str() == u.as_ref()
                                && password.read().as_str() == p.as_ref()
                        }
                    }
            }
        }
    });
    let reset_default_handler = move |_| {
        let default_config = BlockchainProviderConfig::default();
        update_form_from_config(&default_config);
    };

    let reset_current_handler = move |_| {
        if let Some(config) = resource_blockchain_provider_config.read().as_ref() {
            update_form_from_config(config);
        }
    };

    let blockchain_provider_connected = use_memo(|| {
        state_management::BLOCKCHAIN_PROVIDER_STATUS
            .lmap(
                |blockchain_provider_status| match blockchain_provider_status {
                    BlockchainProviderStatus::Connected(_) => true,
                    BlockchainProviderStatus::Disconnected => false,
                },
            )
            .unwrap_or(false)
    });

    rsx! {
        div { class: "card bg-base-200 shadow-xl",
            div { class: "card-body",

                MaybeHighlight {
                    step: OnboardingStep::ConfigureBlockchainProvider,
                    progress: MaybeHighlightProgressType::Signal(blockchain_provider_connected.into()),
                    h2 { class: "card-title",
                        "Blockchain Provider Configuration"
                        BlockchainProviderServiceStatus {}
                    }
                }
                p { class: "text-sm text-gray-600 mb-4",
                    "Configure connection to a Bitcoin node or Electrum server for blockchain synchronization and transaction broadcasting."
                }

                div { class: "flex flex-col gap-4",

                    fieldset { class: "fieldset w-48",
                        legend { class: "fieldset-legend", "Provider Type" }
                        select {
                            class: "select select-bordered",
                            value: "{provider_type}",
                            disabled: *updating.read(),
                            onchange: move |event| *provider_type.write() = event.value(),
                            option { value: "electrum", "Electrum Server" }
                            option { value: "bitcoincore", "Bitcoin Core RPC" }
                        }
                        if let Some(current_provider_type) = current_provider_type() {
                            div { class: "label", "Current: {current_provider_type}" }
                        }
                    }


                    // Electrum configuration
                    if provider_type.read().as_str() == "electrum" {
                        fieldset { class: "w-lg fieldset border-base-content rounded-box border p-4",
                            legend { class: "fieldset-legend", "Electrum Server" }
                            fieldset { class: "fieldset w-full",
                                legend { class: "fieldset-legend", "URL" }
                                input {
                                    r#type: "text",
                                    class: "input",
                                    value: electrum_url(),
                                    disabled: *updating.read(),
                                    oninput: move |event| *electrum_url.write() = event.value(),
                                    placeholder: "ssl://electrum.blockstream.info:50002",
                                }
                                if let Some(BlockchainProviderConfig::Electrum { url }) = resource_blockchain_provider_config
                                    .read()
                                    .as_ref()
                                {
                                    div { class: "label", "Current: {url}" }
                                }
                            }
                        }
                    }

                    // Bitcoin Core configuration
                    if provider_type.read().as_str() == "bitcoincore" {
                        div { class: "flex flex-row flex-wrap gap-4",
                            fieldset { class: "fieldset border-base-content rounded-box border p-4 w-fit",
                                legend { class: "fieldset-legend", "Bitcoin Core RPC" }

                                fieldset { class: "fieldset w-lg",
                                    legend { class: "fieldset-legend", "URL" }
                                    input {
                                        r#type: "url",
                                        class: "input w-full",
                                        value: bitcoincore_url(),
                                        disabled: *updating.read(),
                                        oninput: move |event| *bitcoincore_url.write() = event.value(),
                                        placeholder: "http://localhost:8332",
                                    }
                                    if let Some(BlockchainProviderConfig::BitcoinCore { url, .. }) = resource_blockchain_provider_config
                                        .read()
                                        .as_ref()
                                    {
                                        div { class: "label", "Current: {url}" }
                                    }
                                }
                            }
                            fieldset { class: "fieldset border-base-content rounded-box border p-4 w-lg",

                                legend { class: "fieldset-legend", "Authentication" }

                                fieldset { class: "fieldset w-48",
                                    legend { class: "fieldset-legend", "Method" }
                                    select {
                                        class: "select select-bordered",
                                        value: "{auth_type}",
                                        disabled: *updating.read(),
                                        onchange: move |event| *auth_type.write() = event.value(),
                                        option { value: "cookie", "Cookie File" }
                                        option { value: "userpass", "Username/Password" }
                                    }
                                    if let Some(current_auth_type) = current_auth_type() {
                                        div { class: "label", "Current: {current_auth_type}" }
                                    }
                                }
                                if auth_type.read().as_str() == "cookie" {
                                    fieldset { class: "fieldset",
                                        legend { class: "fieldset-legend", "Cookie File Path" }
                                        input {
                                            r#type: "text",
                                            class: "input",
                                            value: "{cookie_path}",
                                            disabled: *updating.read(),
                                            oninput: move |event| *cookie_path.write() = event.value(),
                                            placeholder: "/home/user/.bitcoin/.cookie",
                                        }
                                        if let Some(
                                            BlockchainProviderConfig::BitcoinCore { auth: AuthConfig::Cookie { file }, .. },
                                        ) = resource_blockchain_provider_config.read().as_ref()
                                        {
                                            div { class: "label", "Current: {file}" }
                                        }
                                    }
                                }

                                if auth_type.read().as_str() == "userpass" {
                                    div { class: "flex flex-row gap-4",
                                        fieldset { class: "fieldset grow",
                                            legend { class: "fieldset-legend", "Username" }
                                            input {
                                                r#type: "text",
                                                class: "input",
                                                value: "{username}",
                                                disabled: *updating.read(),
                                                oninput: move |event| *username.write() = event.value(),
                                                placeholder: "bitcoinrpc",
                                            }
                                            if let Some(
                                                BlockchainProviderConfig::BitcoinCore {
                                                    auth: AuthConfig::UserPass { username: u, .. },
                                                    ..
                                                },
                                            ) = resource_blockchain_provider_config.read().as_ref()
                                            {
                                                div { class: "label", "Current: {u}" }
                                            }
                                        }
                                        fieldset { class: "fieldset grow",
                                            legend { class: "fieldset-legend", "Password" }
                                            input {
                                                r#type: "password",
                                                class: "input",
                                                value: "{password}",
                                                disabled: *updating.read(),
                                                oninput: move |event| *password.write() = event.value(),
                                                placeholder: "Enter RPC password...",
                                            }
                                            if let Some(
                                                BlockchainProviderConfig::BitcoinCore { auth: AuthConfig::UserPass { .. }, .. },
                                            ) = resource_blockchain_provider_config.read().as_ref()
                                            {
                                                div { class: "label",
                                                    "Current: ••••••••"
                                                }
                                            }
                                        }
                                    }
                                }
                            
                            }
                        }
                    }
                }

                div { class: "card-actions justify-end mt-6",
                    button {
                        class: "btn btn-outline",
                        disabled: *updating.read() || is_default(),
                        onclick: reset_default_handler,
                        "Reset to Defaults"
                    }
                    button {
                        class: "btn btn-outline",
                        disabled: *updating.read() || !has_changes()
                            || resource_blockchain_provider_config.read().is_none(),
                        onclick: reset_current_handler,
                        "Reset to Current"
                    }

                    MaybeHighlight {
                        step: OnboardingStep::ConfigureBlockchainProvider,
                        progress: MaybeHighlightProgressType::Signal(blockchain_provider_connected.into()),
                        button {
                            class: "btn btn-primary",
                            disabled: updating() || !has_changes(),
                            onclick: update_handler,
                            if updating() {
                                span { class: "loading loading-spinner loading-sm mr-2" }
                                "Updating..."
                            } else {
                                "Update Configuration"
                            }
                        }
                    }
                }
            }
        }
    }
}

/// Ledger configuration section
#[component]
fn LedgerConfigSection() -> Element {
    rsx! {
        div { class: "card bg-base-200 shadow-xl",
            div { class: "card-body",
                h2 { class: "card-title",
                    "Ledger Hardware Wallet"
                    LedgerServiceStatus {}
                }
                p { class: "text-sm text-gray-600 mb-4",
                    "Ledger hardware wallet integration for secure transaction signing."
                }


                div { class: "alert alert-info mt-4",
                    div { class: "flex items-center",
                        DrawSvg::<InfoCircleOutline> {}
                        div {
                            div { class: "font-medium", "Hardware Wallet Setup:" }
                            div { class: "text-sm mt-1",
                                "To use a Ledger device, connect it via USB, unlock it, and open the Bitcoin app. The device must be in the correct network mode (Bitcoin vs Testnet)."
                            }
                        }
                    }
                }
                div { class: "flex flex-col gap-4",
                    div { class: "form-control",
                        label { class: "label font-medium", "Device Status:" }
                        div { class: "p-4 bg-base-100 rounded-lg",
                            match state_management::LEDGER_STATUS() {
                                Some(LedgerStatus::NotReady) => {
                                    rsx! {
                                        div { class: "flex items-center space-x-2",
                                            div { class: "status status-xl status-error rounded-full" }
                                            span { "No Ledger device detected or Bitcoin app not launched" }
                                        }
                                        div { class: "text-sm text-gray-500 mt-2",
                                            "Connect your Ledger device via USB and open the Bitcoin app."
                                        }
                                    }
                                }
                                Some(LedgerStatus::WrongNetwork) => {
                                    rsx! {
                                        div { class: "flex items-center space-x-2",
                                            div { class: "status status-xl status-error rounded-full" }
                                            span { "Bitcoin app launched with the wrong network" }
                                        }
                                        div { class: "text-sm text-gray-500 mt-2", "Launch the Bitcoin app for the correct Network." }
                                    }
                                }
                                Some(LedgerStatus::WrongApp) => {
                                    rsx! {
                                        div { class: "flex items-center space-x-2",
                                            div { class: "status status-xl status-error rounded-full" }
                                            span { "Ledger device detected but Bitcoin app not launched" }
                                        }
                                        div { class: "text-sm text-gray-500 mt-2", "Launch the Bitcoin app for the correct Network." }
                                    }
                                }
                                Some(LedgerStatus::Ready(fingerprint)) => {
                                    rsx! {
                                        div { class: "space-y-2",
                                            div { class: "flex items-center space-x-2",
                                                div { class: "status status-xl status-success rounded-full" }
                                                span { "Ledger device ready" }
                                            }
                                            div { class: "text-sm text-gray-500", "Device fingerprint: {fingerprint}" }
                                        }
                                    }
                                }
                                None => rsx! {
                                    div { class: "flex items-center space-x-2",
                                        span { class: "loading loading-spinner loading-sm" }
                                        span { "Checking device status..." }
                                    }
                                },
                            }
                        }
                    }
                }
            }
        }
    }
}

/// Onboarding configuration section
#[component]
fn OnboardingConfigSection() -> Element {
    rsx! {
        div { class: "card bg-base-200 shadow-xl",
            div { class: "card-body",
                h2 { class: "card-title", "Application Onboarding" }
                p { class: "text-sm text-gray-600 mb-4",
                    "The onboarding guiding your first steps into the Heritage wallet application."
                }

                div { class: "flex flex-col gap-4",
                    label { class: "label font-medium", "Onboarding Status:" }
                    div { class: "p-4 bg-base-100 rounded-lg",
                        match &*state_management::ONBOARDING_STATUS.read() {
                            OnboardingStatus::Pending => rsx! {
                                "Pending"
                                div { class: "text-sm text-gray-500 mt-2",
                                    "The onboarding process will launch next time the application is started."
                                }
                            },
                            OnboardingStatus::InProgress(_) => rsx! {
                                "In Progress"
                                div { class: "text-sm text-gray-500 mt-2",
                                    "The onboarding process is currently guiding you in the app."
                                }
                            },
                            OnboardingStatus::Completed => rsx! {
                                "Completed"
                                div { class: "text-sm text-gray-500 mt-2", "The onboarding process has been completed." }
                            },
                        }
                    }
                }

                div { class: "card-actions justify-end mt-6",
                    button {
                        class: "btn btn-outline",
                        disabled: matches!(&*state_management::ONBOARDING_STATUS.read(), OnboardingStatus::Pending),
                        onclick: move |_| {
                            *state_management::ONBOARDING_STATUS.write() = OnboardingStatus::Pending;
                        },
                        "Reset to pending"
                    }
                    button {
                        class: "btn btn-outline",
                        disabled: matches!(&*state_management::ONBOARDING_STATUS.read(), OnboardingStatus::Completed),
                        onclick: move |_| {
                            *state_management::ONBOARDING_STATUS.write() = OnboardingStatus::Completed;
                        },
                        "Mark Completed"
                    }
                }
            }
        }
    }
}
