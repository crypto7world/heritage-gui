use crate::prelude::*;

use btc_heritage_wallet::{
    btc_heritage::{
        bitcoincore_rpc::jsonrpc::serde_json, utils::timestamp_now, HeritageWalletBackup,
    },
    AnyKeyProvider, HeirWallet, KeyProvider, Wallet,
};

use crate::{
    components::{
        copy::CopyTextarea,
        modal::InfoModal,
        svg::{
            Alert, Cancel, DrawSvg, FileDownload, InfoCircleOutline, Seed, SvgSize::Size4, Unlock,
        },
    },
    utils::{log_error, log_error_ccstr, timestamp_to_file_string, CCStr},
};

#[cfg(feature = "desktop")]
use crate::components::inputs::FileInput;

#[cfg(feature = "desktop")]
use std::fs;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ShowKeyProviderMnemonicFlavor {
    Wallet,
    Heir,
}

#[doc = "Properties for the [`ShowKeyProviderMnemonic`] component."]
#[allow(missing_docs)]
#[derive(Props, Clone, PartialEq)]
#[allow(non_camel_case_types)]
pub struct ShowKeyProviderMnemonicProps {
    pub flavor: ShowKeyProviderMnemonicFlavor,
}
#[doc = " Component that displays a button to show the key provider mnemonic"]
#[doc = " When clicked, opens a modal with security disclaimer and mnemonic display"]
#[doc = "# Props\n*For details, see the [props struct definition](ShowKeyProviderMnemonicProps).*"]
#[doc = "- [`flavor`](ShowKeyProviderMnemonicProps::flavor) : `ShowKeyProviderMnemonicFlavor`"]
#[allow(non_snake_case)]
pub fn ShowKeyProviderMnemonic<KP: KeyProvider + 'static>(
    ShowKeyProviderMnemonicProps { flavor }: ShowKeyProviderMnemonicProps,
) -> Element {
    log::debug!("ShowKeyProviderMnemonic Rendered");

    let kp = use_context::<AsyncSignal<KP>>();
    let mnemonic_backup = use_resource(move || async move {
        log::debug!("use_resource_kp_mnemonic_backup - start");
        let backup = kp
            .with(async |kp| kp.backup_mnemonic().await.map_err(log_error_ccstr))
            .await;
        log::debug!("use_resource_kp_mnemonic_backup - loaded");
        backup
    });

    let mut display_modal = use_signal(|| false);
    let mut disclaimer_accepted = use_signal(|| false);

    let show_mnemonic_click = move |_| {
        *display_modal.write() = true;
        *disclaimer_accepted.write() = false;
    };

    let mnemonic_info = use_memo(move || {
        mnemonic_backup
            .read()
            .as_ref()
            .and_then(|backup| backup.as_ref().ok())
            .map(|backup| {
                (
                    CCStr::from(backup.mnemonic.words().collect::<Vec<&str>>().join(" ")),
                    backup.fingerprint,
                    backup.with_password,
                )
            })
    });

    let fingerprint =
        use_memo(move || mnemonic_info.lmap(|(_, fg, _)| CCStr::from(fg.to_string())));

    let with_password =
        use_memo(move || mnemonic_info.lmap(|(_, _, with_password)| *with_password));
    let password_hint = use_memo(move || {
        with_password().map(|with_password| match (flavor, with_password) {
            (ShowKeyProviderMnemonicFlavor::Wallet, true) => {
                "Yes - This wallet requires a password to access private keys"
            }
            (ShowKeyProviderMnemonicFlavor::Wallet, false) => {
                "No - This wallet does not use password protection"
            }
            (ShowKeyProviderMnemonicFlavor::Heir, true) => {
                "Yes - The heir requires a password to spend inheritances"
            }
            (ShowKeyProviderMnemonicFlavor::Heir, false) => {
                "No - The heir does not need a password"
            }
        })
    });
    let password_hint_color = use_memo(move || with_password().is_some_and(|b| !b));

    use_drop(|| log::debug!("ShowKeyProviderMnemonic Dropped"));

    rsx! {
        MaybeHighlight {
            step: OnboardingStep::ClickHeirShowMnemonic,
            context_filter: consume_onboarding_context(),
            button {
                class: "btn btn-xs btn-outline btn-error",
                onclick: show_mnemonic_click,
                DrawSvg::<Seed> { size: Size4 }
                "Show Mnemonic"
            }
        }
        InfoModal { is_open: display_modal, title: "Wallet Mnemonic Phrase",
            div { class: "flex flex-col gap-4 max-w-2xl",
                div { class: "alert alert-error",
                    DrawSvg::<Alert> {}
                    div {
                        h3 { class: "font-bold", "Security Warning" }
                        match flavor {
                            ShowKeyProviderMnemonicFlavor::Wallet => rsx! {
                                p {
                                    "This mnemonic words are the master key to your Bitcoin wallet. "
                                    "Anyone with access to these words may spend all your coins. "
                                    "Store them securely and never share them with anyone."
                                }
                            },
                            ShowKeyProviderMnemonicFlavor::Heir => rsx! {
                                p {
                                    "This mnemonic words allow the heir to spend their inheritance. "
                                    "Anyone with access to these words may spend it. "
                                    "The heir must store them securely and never share them with anyone."
                                }
                            },
                        }
                    }
                }

                div { class: "flex flex-col gap-1 p-4 bg-base-200 rounded-lg",
                    span { class: "text-sm font-semibold", "Wallet Fingerprint:" }
                    span { class: "font-mono text-lg font-black",
                        LoadedComponent::<CCStr> { input: fingerprint().into() }
                    }

                    span { class: "text-sm font-semibold", "Password Protection:" }
                    span {
                        class: "text-lg",
                        class: if password_hint_color() { "text-warning" },
                        LoadedComponent::<&str> { input: password_hint().into() }
                    }
                }

                if let Some((mnemonic_words, _, _)) = mnemonic_info() {
                    div { class: "flex flex-col gap-2",
                        div { class: "font-semibold", "Mnemonic Phrase:" }
                        MaybeHighlight {
                            step: OnboardingStep::CheckHeirRevealMnemonic,
                            context_filter: consume_onboarding_context(),
                            label { class: "label justify-start gap-2",
                                input {
                                    r#type: "checkbox",
                                    class: "checkbox checkbox-error",
                                    checked: disclaimer_accepted(),
                                    onchange: move |evt| *disclaimer_accepted.write() = evt.checked(),
                                }
                                "I understand that this mnemonic phrase must be kept secret and secure"
                            }
                        }
                        MaybeHighlight {
                            step: OnboardingStep::HoverHeirMnemonic,
                            progress: MaybeHighlightProgressType::Hover(1),
                            context_filter: consume_onboarding_context(),
                            div { class: if !disclaimer_accepted() { "blur-xs" },
                                CopyTextarea {
                                    value: if disclaimer_accepted() { mnemonic_words.clone() } else { "this is not the real mnemonic phrase but just place holder data".into() },
                                    rows: 3,
                                    text_size: "text-sm",
                                    copy_btn_disabled: !disclaimer_accepted(),
                                }
                            }
                        }
                        div { class: "text-sm text-base-content/60",
                            "Write down these words in order and store them in a safe place."
                        }
                    }
                }
            }
        }
    }
}

/// Component that displays a button to backup online wallet data
/// When clicked, opens a modal with backup data and download functionality
#[component]
pub fn BackupOnlineWallet(wallet_name: CCStr) -> Element {
    log::debug!("BackupOnlineWallet Rendered");

    let descriptor_backup = use_context::<FResource<HeritageWalletBackup>>();

    let mut display_modal = use_signal(|| false);

    let show_backup_click = move |_| {
        *display_modal.write() = true;
    };

    let backup_data = use_memo(move || {
        descriptor_backup
            .lrmap_ok(|backup| {
                CCStr::from(
                    serde_json::to_string_pretty(backup).expect("HeritageBackup is serializable"),
                )
            })
            .unwrap_or_default()
    });

    let mut backup_directory = use_signal(|| {
        dirs_next::home_dir()
            .unwrap_or_default()
            .to_str()
            .unwrap_or_default()
            .to_owned()
    });
    let backup_path = use_memo(move || {
        let ts_str = timestamp_to_file_string(timestamp_now());
        format!(
            "{}/backup-{wallet_name}-{ts_str}.json",
            backup_directory.read(),
        )
    });

    #[cfg(feature = "desktop")]
    const CAN_DOWNLOAD: bool = true;
    #[cfg(not(feature = "desktop"))]
    const CAN_DOWNLOAD: bool = false;

    let download_backup = move |_| async move {
        #[cfg(feature = "desktop")]
        {
            let file_path = backup_path.read();
            let data = backup_data();

            match fs::write(file_path.as_str(), data.as_ref()) {
                Ok(()) => {
                    log::info!("Backup file written successfully to: {}", file_path);
                    alert_info(format!("Backup saved to: {}", file_path));
                    *display_modal.write() = false;
                }
                Err(e) => {
                    log::error!("Failed to write backup file: {}", e);
                    alert_error(format!("Failed to save backup: {}", e));
                }
            }
        }
        #[cfg(not(feature = "desktop"))]
        {
            log::info!("File save unavailable on this platform");
            alert_error("File save unavailable on this platform");
        }
    };

    use_drop(|| log::debug!("BackupOnlineWallet Dropped"));

    rsx! {
        MaybeHighlight {
            step: OnboardingStep::ClickBackupDescriptors,
            context_filter: consume_onboarding_context(),
            button { class: "btn btn-xs btn-outline", onclick: show_backup_click,
                DrawSvg::<FileDownload> { size: Size4 }
                "Backup Descriptors"
            }
        }
        InfoModal { is_open: display_modal, title: "Online Wallet Descriptors Backup",
            div { class: "flex flex-col gap-4 max-w-2xl",
                div { class: "alert alert-info",
                    DrawSvg::<InfoCircleOutline> {}
                    div {
                        h3 { class: "font-bold", "Wallet Backup Data" }
                        p {
                            "This backup contains your wallet's Bitcoin descriptors.
                            Descriptors can be used to restore your wallet's online functionality
                            if need be, including on another device or wallet software that
                            supports Taproot descriptors."
                        }
                    }
                }

                div { class: "flex flex-col gap-2",
                    div { class: "font-semibold", "Backup Data:" }
                    CopyTextarea { value: backup_data(), rows: 12 }
                }

                if CAN_DOWNLOAD {
                    div { class: "flex justify-center",
                        FileInput {
                            display_path: ReadOnlySignal::from(backup_path),
                            directory: true,
                            onchange: move |evt: Event<FormData>| async move {
                                if let Some(file_engine) = evt.files().clone() {
                                    for file in file_engine.files() {
                                        backup_directory.set(file);
                                    }
                                }
                            },
                        }
                        MaybeHighlight {
                            step: OnboardingStep::ClickSaveBackup,
                            context_filter: consume_onboarding_context(),
                            button { class: "btn", onclick: download_backup,
                                DrawSvg::<FileDownload> { size: Size4 }
                                "Save Backup"
                            }
                        }
                    }
                }
            }
        }
    }
}

pub trait LocalKeyUnlocker {
    fn unlock(&mut self, password: String) -> Result<(), String>;
}
macro_rules! impl_lku {
    ($name:ident) => {
        impl LocalKeyUnlocker for $name {
            fn unlock(&mut self, password: String) -> Result<(), String> {
                match self.key_provider_mut() {
                    AnyKeyProvider::LocalKey(lk) => {
                        lk.init_local_key(Some(password)).map_err(log_error)
                    }
                    _ => Err("Wrong key provider type".to_owned()),
                }
            }
        }
    };
}
impl_lku!(Wallet);
impl_lku!(HeirWallet);

/// Component that displays a button to unlock a local key with password
/// When clicked, opens a modal with password input field
#[component]
pub fn UnlockLocalKey<LKU: LocalKeyUnlocker + 'static>() -> Element {
    log::debug!("UnlockLocalKey Rendered");

    let mut lku = use_context::<AsyncSignal<LKU>>();

    let mut display_modal = use_signal(|| false);
    let mut password = use_signal(|| String::new());
    let mut is_unlocking = use_signal(|| false);

    let show_unlock_click = move |_| {
        *display_modal.write() = true;
        *password.write() = String::new();
    };

    let unlock_process = move || async move {
        log::info!("Attempting to unlock local key with password");
        *is_unlocking.write() = true;

        let password = password();
        match lku.with_mut(async move |lku| lku.unlock(password)).await {
            Ok(()) => {
                log::info!("Local Key Provider successfully unlocked");
                alert_info("Local Key Provider successfully unlocked");
                *display_modal.write() = false;
            }
            Err(e) => {
                log::error!("Failed to unlock Local Key Provider: {e}");
                alert_error(e);
            }
        }
        *is_unlocking.write() = false;
    };

    let unlock_click = move |_| async move { unlock_process().await };

    let password_valid = use_memo(move || !password.read().is_empty());

    use_drop(|| log::debug!("UnlockLocalKey Dropped"));

    rsx! {
        button {
            class: "btn btn-xs btn-outline btn-secondary",
            onclick: show_unlock_click,
            DrawSvg::<Unlock> { size: Size4 }
            "Unlock"
        }
        InfoModal { is_open: display_modal, title: "Unlock Local Key Provider",
            div { class: "flex flex-col gap-4 max-w-2xl",
                div { class: "alert alert-info",
                    DrawSvg::<InfoCircleOutline> {}
                    div {
                        h3 { class: "font-bold", "Password Required" }
                        p {
                            "This wallet requires a password to access the private keys. "
                            "Enter your password to unlock the key provider."
                        }
                    }
                }

                div { class: "flex flex-col gap-4",
                    fieldset { class: "fieldset",
                        legend { class: "fieldset-legend text-nowrap", "Password" }
                        input {
                            r#type: "password",
                            class: "input w-full",
                            placeholder: "Enter your password",
                            value: password(),
                            oninput: move |evt| *password.write() = evt.value(),
                            onkeydown: move |evt| {
                                if evt.key() == Key::Enter && password_valid() && !is_unlocking() {
                                    spawn(unlock_process());
                                }
                            },
                            disabled: is_unlocking(),
                        }
                    }

                    div { class: "flex justify-end gap-2",
                        button {
                            class: "btn btn-outline",
                            onclick: move |_| *display_modal.write() = false,
                            disabled: is_unlocking(),
                            DrawSvg::<Cancel> {}
                            "Cancel"
                        }
                        button {
                            class: "btn btn-primary",
                            disabled: !password_valid() || is_unlocking(),
                            onclick: unlock_click,
                            if is_unlocking() {
                                span { class: "loading loading-spinner loading-sm mr-2" }
                                "Unlocking..."
                            } else {
                                DrawSvg::<Unlock> {}
                                "Unlock"
                            }
                        }
                    }
                }
            }
        }
    }
}
