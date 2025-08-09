use crate::{
    prelude::*,
    utils::{log_error, CheapClone},
};

use std::{collections::HashSet, time::Duration};

use btc_heritage_wallet::{
    btc_heritage::{
        bitcoincore_rpc::jsonrpc::serde_json, utils::timestamp_now, HeritageWalletBackup,
    },
    heritage_service_api_client::Fingerprint,
    DatabaseItem, Heir,
};

use crate::{
    components::svg::{Cancel, DrawSvg, Update},
    utils::CCStr,
};

#[component]
pub fn RadioChoices(count: usize, children: Element) -> Element {
    let grid_classes = match count {
        2 => "grid-rows-2 grid-cols-1 sm:grid-rows-1 sm:grid-cols-2",
        3 => "grid-rows-3 grid-cols-1 lg:grid-rows-1 lg:grid-cols-3",
        4 => "grid-rows-4 grid-cols-1 sm:grid-rows-2 sm:grid-cols-2",
        _ => "grid-cols-1 lg:grid-cols-3",
    };
    rsx! {
        div { class: "grid {grid_classes} gap-4", {children} }
    }
}

#[component]
pub fn RadioChoice<T: 'static + Clone + Copy + PartialEq>(
    name: &'static str,
    state: Signal<T>,
    value: T,
    title: &'static str,
    subtitle: &'static str,
    disabled: Option<bool>,
) -> Element {
    log::debug!("RadioChoice {name} Rendered");

    use_drop(move || log::debug!("RadioChoice {name} Dropped"));
    rsx! {
        label { class: "label has-[input:disabled]:cursor-not-allowed border rounded-lg p-4 hover:bg-base-200",
            input {
                r#type: "radio",
                name,
                class: "radio radio-primary",
                checked: state() == value,
                onchange: move |_| state.set(value),
                disabled,
            }
            div { class: "ml-3",
                span { class: "text-xl text-base-content font-semibold", {title} }
                div { class: "text-base text-base-content/60 text-wrap", {subtitle} }
            }
        }
    }
}

#[component]
pub fn InputField<T: 'static + Clone + PartialEq + core::fmt::Display + core::str::FromStr>(
    title: Option<&'static str>,
    description: Option<&'static str>,
    value: Signal<T>,
    r#type: Option<&'static str>,
    placeholder: Option<&'static str>,
    value_error: ReadOnlySignal<Option<CCStr>>,
) -> Element {
    let (error_display, mut signal_activity, onfocusout) = use_future_error_feedback(value_error);

    rsx! {
        fieldset { class: "fieldset",
            if let Some(title) = title {
                legend { class: "fieldset-legend", {title} }
            }
            if let Some(description) = description {
                div { class: "fieldset-description", {description} }
            }
            input {
                r#type: r#type.unwrap_or("text"),
                class: "input w-full",
                class: if error_display().is_some() { "input-error" },
                placeholder,
                value: "{value.read()}",
                oninput: move |evt| {
                    signal_activity();
                    if let Ok(v) = evt.parsed() {
                        value.set(v)
                    }
                },
                onfocusout,
            }
            div {
                class: "fieldset-label text-error",
                class: if error_display().is_none() { "invisible" },
                if let Some(e) = error_display() {
                    {e}
                } else {
                    "ph"
                }
            }
        }
    }
}

#[component]
pub fn FileInput(
    display_path: Option<ReadOnlySignal<String>>,
    accept: Option<&'static str>,
    multiple: Option<bool>,
    directory: Option<bool>,
    onchange: Option<Callback<Event<FormData>>>,
    onfocusout: Option<Callback<Event<FocusData>>>,
) -> Element {
    let mut paths = use_signal(|| vec![]);
    let display = use_memo(move || {
        let path = if let Some(display_path) = display_path {
            display_path()
        } else {
            paths.read().join(", ")
        };
        if !path.is_empty() {
            path
        } else {
            if directory.unwrap_or(false) {
                "no directory selected".to_owned()
            } else {
                "no file selected".to_owned()
            }
        }
    });
    rsx! {
        // See issue tracked:
        // https://github.com/DioxusLabs/dioxus/issues/3439
        div { class: "relative w-full",
            input {
                r#type: "file",
                class: "file-input w-0 border-none focus:outline-none file:text-transparent file:text-shadow-none",
                accept: accept.unwrap_or("*"),
                multiple: multiple.unwrap_or(false),
                directory: directory.unwrap_or(false),
                onchange: move |evt| {
                    if let Some(file_engine) = evt.files().clone() {
                        paths.set(file_engine.files());
                    }
                    if let Some(cb) = onchange {
                        cb.call(evt);
                    }
                },
                onfocusout: move |evt| {
                    if let Some(cb) = onfocusout {
                        cb.call(evt)
                    }
                },
            }
            div { class: "absolute top-0 left-0 w-[112px] h-full flex justify-center items-center text-sm text-(--btn-fg) text-center font-semibold cursor-pointer pointer-events-none",
                if directory.unwrap_or(false) {
                    "Choose Directory"
                } else {
                    "Choose File"
                }
            }
            div { class: "absolute top-0 left-[112px] w-[calc(100%-112px)] h-full pl-2 flex items-center text-sm text-nowrap overflow-auto border-t border-b border-r border-base-content/20 rounded-e",
                {display()}
            }
        }
    }
}

/// Backup restore section
#[component]
pub fn BackupRestoreSection(
    heritage_wallet_backup_state: Signal<Result<HeritageWalletBackup, CCStr>>,
    expected_fingerprint: Option<Fingerprint>,
) -> Element {
    // Internal state - not exposed to parent
    let mut backup_data = use_signal(String::new);
    let mut backup_file_path = use_signal(String::new);

    // Internal validation

    let backup_data_fingerprint = use_memo(move || {
        let generic_error = CCStr::from("Cannot extract fingerprint from backup data");
        match &*heritage_wallet_backup_state.read() {
            Ok(bkp) => match bkp.fingerprint() {
                Ok(Some(fg)) => Ok(fg),
                Ok(None) => Err(generic_error),
                Err(e) => match e {
                    btc_heritage_wallet::btc_heritage::errors::Error::InvalidBackup(s) => {
                        Err(CCStr::from(s))
                    }
                    _ => Err(generic_error),
                },
            },
            Err(s) => Err(s.clone()),
        }
    });

    let success_display = use_memo(move || {
        backup_data_fingerprint().is_ok_and(|fg| expected_fingerprint.is_none_or(|efg| efg == fg))
    });
    let backup_error = use_memo(move || {
        if backup_data.read().is_empty() {
            Some(CCStr::from(
                "Provide backup data, either directly or using a file",
            ))
        } else if let Err(ref e) = *heritage_wallet_backup_state.read() {
            Some(e.clone())
        } else if let Err(ref e) = *backup_data_fingerprint.read() {
            Some(e.clone())
        } else if backup_data_fingerprint()
            .is_ok_and(|fg| expected_fingerprint.is_some_and(|efg| efg != fg))
        {
            Some(CCStr::from(
                "The provided backup data do not have the expected fingerprint",
            ))
        } else {
            None
        }
    });
    let (error_display, mut signal_activity, onfocusout) =
        use_future_error_feedback_with_delay(backup_error.into(), 0);

    // Update parent signal when internal state changes
    use_effect(move || {
        heritage_wallet_backup_state.set(
            serde_json::from_str::<HeritageWalletBackup>(backup_data.read().as_str()).map_err(
                |e| {
                    log::warn!("{e}");
                    CCStr::from(e.to_string())
                },
            ),
        );
    });

    rsx! {

        div { class: "flex flex-col",

            fieldset { class: "fieldset",
                legend { class: "fieldset-legend", "Backup File" }
                FileInput {
                    accept: ".txt,.json",
                    onchange: move |evt: Event<FormData>| async move {
                        signal_activity();
                        if let Some(file_engine) = evt.files().clone() {
                            for file in file_engine.files() {
                                backup_data
                                    .set(
                                        file_engine.read_file_to_string(&file).await.unwrap_or_default(),
                                    );
                                backup_file_path.set(file);
                            }
                        }
                    },
                    onfocusout,
                }
            }
            fieldset { class: "fieldset w-full",
                legend { class: "fieldset-legend", "Backup Data" }
                textarea {
                    class: "textarea textarea-bordered font-mono text-xs w-full",
                    class: if error_display().is_some() { "textarea-error" },
                    class: if success_display() { "textarea-success" },
                    rows: "12",
                    placeholder: "Paste your backup string here...",
                    value: backup_data(),
                    oninput: move |evt| {
                        signal_activity();
                        backup_data.set(evt.value());
                    },
                    onfocusout,
                }
                div {
                    class: "fieldset-label",
                    class: if error_display().is_none() && backup_data_fingerprint().is_err() { "invisible" },
                    class: if error_display().is_some() { "text-error" },
                    class: if success_display() { "text-success" },
                    if let Some(e) = error_display() {
                        {e}
                    } else if let Ok(fg) = backup_data_fingerprint() {
                        "Valid backup for a wallet with fingerprint "
                        span { class: "font-bold", "{fg}" }
                    } else {
                        "ph"
                    }
                }
            }
        }
    }
}

pub fn use_future_error_feedback(
    value_error: ReadOnlySignal<Option<CCStr>>,
) -> (
    Memo<Option<CCStr>>,
    impl FnMut() + Copy,
    impl FnMut(Event<FocusData>) + Copy,
) {
    use_future_error_feedback_with_delay(value_error, 2)
}
pub fn use_future_error_feedback_with_delay(
    value_error: ReadOnlySignal<Option<CCStr>>,
    delay_sec: u64,
) -> (
    Memo<Option<CCStr>>,
    impl FnMut() + Copy,
    impl FnMut(Event<FocusData>) + Copy,
) {
    let (feed_back_active, signal_activity, onfocusout) = use_future_feedback_with_delay(delay_sec);
    let error_display = use_future_error_feedback_from_parts(feed_back_active, value_error);
    (error_display, signal_activity, onfocusout)
}
pub fn use_future_error_feedback_from_parts(
    feed_back_active: Memo<bool>,
    value_error: ReadOnlySignal<Option<CCStr>>,
) -> Memo<Option<CCStr>> {
    use_memo(move || feed_back_active().then(|| value_error()).flatten())
}
pub fn use_future_feedback() -> (
    Memo<bool>,
    impl FnMut() + Copy,
    impl FnMut(Event<FocusData>) + Copy,
) {
    use_future_feedback_with_delay(2)
}
pub fn use_future_feedback_with_delay(
    delay_sec: u64,
) -> (
    Memo<bool>,
    impl FnMut() + Copy,
    impl FnMut(Event<FocusData>) + Copy,
) {
    let mut last_activity_ts = use_signal(|| None);
    let mut timed_feedback = use_signal(|| false);
    let mut immediate_feedback = use_signal(|| false);
    use_future(move || async move {
        loop {
            if last_activity_ts().is_some_and(|ts| ts + delay_sec < timestamp_now()) {
                if !timed_feedback() {
                    *timed_feedback.write() = true;
                }
            }
            tokio::time::sleep(Duration::from_millis(250)).await;
        }
    });

    let feed_back_active = use_memo(move || (timed_feedback() || immediate_feedback()));
    let signal_activity = move || {
        *last_activity_ts.write() = Some(timestamp_now());
        if immediate_feedback() {
            *immediate_feedback.write() = false;
        }
        if timed_feedback() {
            *timed_feedback.write() = false;
        }
    };
    let onfocusout = move |_| *immediate_feedback.write() = true;
    (feed_back_active, signal_activity, onfocusout)
}

#[component]
pub fn RenameDatabaseItem<DBI: DatabaseItem + 'static + Send>() -> Element {
    let database_item = use_context::<AsyncSignal<DBI>>();

    let database_service = state_management::use_database_service();

    let existing_dbi_names = use_resource(move || async move {
        state_management::blocking_db_service_operation(database_service, |db| {
            DBI::list_names(&db).unwrap_or_default()
        })
        .await
        .into_iter()
        .collect::<HashSet<_>>()
    });

    let current_name = use_memo(move || {
        database_item
            .lmap(|dbi| CCStr::from(dbi.name()))
            .unwrap_or_default()
    });
    let mut new_name = use_signal(|| current_name().to_string());

    // Internal validation
    let name_present = use_memo(move || !new_name.read().trim().is_empty());
    let name_available = use_memo(move || {
        let name_ref = new_name.read();
        let name = name_ref.trim();
        if name.is_empty() {
            true // Don't show error for empty name
        } else {
            existing_dbi_names
                .lmap(|set| !set.contains(name))
                .unwrap_or(true)
        }
    });
    let name_forbidden = use_memo(move || new_name.read().trim() == "create");

    let has_changes = use_memo(move || *new_name.read() != *current_name());

    let name_error = use_memo(move || {
        if !has_changes() {
            None
        } else if !name_present() {
            Some(CCStr::from("Name is required"))
        } else if name_forbidden() {
            Some(CCStr::from("\"create\" cannot be used as a name"))
        } else if !name_available() {
            Some(CCStr::from("This name is already in use"))
        } else {
            None
        }
    });

    let can_change = use_memo(move || name_error.read().is_none());

    let mut updating = use_signal(move || false);

    let update_handler = move |_| async move {
        *updating.write() = true;

        let mut abort = |message: &str| {
            alert_error(message);
            log::error!("{message}");
            *updating.write() = false;
        };

        let s_current_name = current_name().to_string();
        let Ok(mut owned_dbi) =
            state_management::blocking_db_service_operation(database_service, move |db| {
                DBI::load(&db, &s_current_name).map_err(log_error)
            })
            .await
        else {
            return abort(&format!("Internal error"));
        };

        // Change in database
        let s_new_name = new_name();
        match state_management::blocking_db_service_operation(database_service, move |mut db| {
            owned_dbi.db_rename(&mut db, s_new_name)
        })
        .await
        {
            Ok(()) => {
                let msg = format!("Name changed successfully to {}", new_name.read());
                alert_success(&msg);
                log::info!("{msg}");

                // Return to list depending on the type
                match DBI::item_key_prefix() {
                    "wallet#" => {
                        navigator().push(crate::Route::WalletListView {});
                    }
                    "heirwallet#" => {
                        navigator().push(crate::Route::HeirWalletListView {});
                    }
                    "heir#" => {
                        let mut database_heirs = use_context::<Resource<Vec<CheapClone<Heir>>>>();
                        database_heirs.restart();
                        navigator().push(crate::Route::HeirListView {});
                    }
                    _ => (),
                }
            }
            Err(e) => {
                return abort(&format!("Fail to change name: {e}"));
            }
        }

        *updating.write() = false;
    };

    let (error_display, mut signal_activity, onfocusout) =
        use_future_error_feedback(name_error.into());

    rsx! {
        div { class: "card [--cardtitle-fs:var(--text-2xl)] border border-base-content/5 shadow-md my-4",
            div { class: "card-body",
                h2 { class: "card-title", "Rename" }
                div { class: "card-subtitle", "Change the name used in the app." }
                fieldset { class: "fieldset w-80",
                    legend { class: "fieldset-legend", "New name" }
                    input {
                        r#type: "text",
                        class: "input",
                        value: "{new_name}",
                        disabled: *updating.read(),
                        oninput: move |event| {
                            signal_activity();
                            new_name.set(event.value());
                        },
                        onfocusout,
                        placeholder: "Enter a unique new name...",
                    }
                    div {
                        class: "fieldset-label",
                        class: if error_display().is_some() { "text-error" },
                        if let Some(e) = error_display() {
                            {e}
                        } else {
                            "Current: {current_name()}"
                        }
                    }
                }
                div { class: "card-actions",
                    button {
                        class: "btn btn-primary",
                        disabled: updating() || !has_changes() || !can_change(),
                        onclick: update_handler,
                        if updating() {
                            span { class: "loading loading-spinner loading-sm mr-2" }
                            "Updating..."
                        } else {
                            DrawSvg::<Update> {}
                            "Update Name"
                        }
                    }
                    button {
                        class: "btn btn-primary btn-outline",
                        disabled: updating() || !has_changes(),
                        onclick: move |_| {
                            new_name.set(current_name().to_string());
                        },
                        DrawSvg::<Cancel> {}
                        "Reset"
                    }
                }
            }
        }
    }
}
