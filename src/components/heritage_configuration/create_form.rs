use dioxus::prelude::*;

use btc_heritage_wallet::{
    btc_heritage::{heritage_config::v1::Heritage, HeirConfig, HeritageConfig},
    OnlineWallet, Wallet,
};
use chrono::{Local, NaiveDate};
use futures_util::future::BoxFuture;
use std::{
    collections::{HashMap, HashSet},
    future::Future,
};

use crate::{
    components::{
        alerts::{alert_error, alert_info, alert_success},
        misc::Tooltip,
        timestamp::UITimestamp,
    },
    helper_hooks::{async_init::AsyncSignal, CompositeHeir},
    loaded::prelude::*,
    utils::{timestamp_to_date_string, ArcStr, ArcType},
};

/// Form for creating or updating a heritage configuration.
///
/// This component provides a user interface for configuring inheritance settings,
/// including:
/// - Setting the reference date
/// - Adding heirs and their maturity delays
/// - Configuring minimum maturity times
#[component]
pub fn NewHeritageConfigForm(
    existing_heritage_config: ReadOnlySignal<Option<ArcType<HeritageConfig>>>,
    new_heritage_config_modal: Signal<bool>,
) -> Element {
    log::debug!("NewHeritageConfigForm Rendered");

    // Context resources
    let mut wallet = use_context::<AsyncSignal<Wallet>>();
    let heirs = use_context::<Memo<HashMap<ArcType<HeirConfig>, CompositeHeir>>>();

    // Form state
    let mut creating = use_signal(|| false);

    let today = today_noon();

    // Heritage config state
    let mut new_heritage_config = use_signal(|| HeritageConfigState {
        reference_ts: today,
        minimum_lock_time: 30,
        heritages: vec![],
    });
    use_context_provider(|| new_heritage_config);

    // Compute if we already have heritage configs
    let has_heritage_configs = use_memo(move || existing_heritage_config.read().is_some());

    // Initialize form with current config data if available
    use_effect(move || {
        if let Some(current_config) = existing_heritage_config.read().as_ref() {
            let config_v1 = current_config
                .heritage_config_v1()
                .expect("Should be V1 config");

            // Create heritages data
            let heritages = config_v1
                .iter_heritages()
                .map(|h| HeritageState {
                    heir_config: Some(ArcType::from(h.heir_config.clone())),
                    time_lock: h.time_lock.as_u16(),
                })
                .collect::<Vec<_>>();

            let hcs = &mut *new_heritage_config.write();
            hcs.minimum_lock_time = config_v1.minimum_lock_time.as_days().as_u16();
            hcs.heritages = heritages;
        }
    });

    // Detect duplicate heirs
    let new_heritage_config_heirs_duplicates = use_memo(move || {
        log::debug!("use_memo_new_heritage_config_heirs_duplicates - start compute");
        let mut already_seen_heir_ids = HashSet::new();
        let mut duplicates = HashSet::new();

        for heritage in new_heritage_config.read().heritages.iter() {
            if let Some(hc) = &heritage.heir_config {
                if already_seen_heir_ids.contains(hc) {
                    duplicates.insert(hc.clone());
                } else {
                    already_seen_heir_ids.insert(hc.clone());
                }
            }
        }

        log::debug!("use_memo_new_heritage_config_heirs_duplicates - finish compute");

        duplicates
    });
    use_context_provider(|| new_heritage_config_heirs_duplicates);

    // Calculate min and max time locks for each heir position
    let min_max_time_locks = use_memo(move || {
        log::debug!("use_memo_min_max_time_locks - start compute");
        let heritages = &new_heritage_config.read().heritages;
        let mut results = Vec::with_capacity(heritages.len() + 1);

        // The minimum of the first heir is always 180 days
        results.push(TimeConstraint {
            min: 180,
            min_tt: "First heir must wait at least 6 months".into(),
            max: 1825,
            max_tt: "First heir can wait up to 5 years".into(),
        });

        // For subsequent heirs, min is previous + 30 days, max is previous + 2 years (capped at 10 years)
        for heritage in heritages.iter() {
            results.push(TimeConstraint {
                min: heritage.time_lock + 30,
                min_tt: "Next heir must wait at least 30 days more than the previous heir".into(),
                max: (heritage.time_lock + 730).min(3650),
                max_tt: "Next heir can wait up to 2 years more than the previous heir, but not more than 10 years total".into(),
            });
        }

        log::debug!("use_memo_min_max_time_locks - finish compute");

        results
    });
    use_context_provider(|| min_max_time_locks);

    // Enforce time lock constraints
    use_effect(move || {
        log::debug!("use_effect_min_max_time_locks - start");

        let constraints = &*min_max_time_locks.read();
        for (i, heritage) in new_heritage_config.write().heritages.iter_mut().enumerate() {
            if heritage.time_lock < constraints[i].min {
                heritage.time_lock = constraints[i].min;
            }
            if heritage.time_lock > constraints[i].max {
                heritage.time_lock = constraints[i].max;
            }
        }
        log::debug!("use_effect_min_max_time_locks - finish");
    });

    // Get available heirs indexed for dropdown
    let heir_options = use_memo(move || {
        let mut hash_map = HashMap::new();
        for (heir_config, composite_heir) in heirs.read().iter() {
            let name = composite_heir.name.clone();
            let heir_config = heir_config.clone();
            let id = heir_config.to_string();
            let email = composite_heir
                .service_heir
                .as_ref()
                .map(|sh| ArcStr::from(sh.main_contact.email.as_str()));
            let heir_option = HeirOption {
                id: ArcStr::from(&id),
                name,
                email,
                heir_config,
            };
            hash_map.insert(id, heir_option);
        }
        hash_map
    });
    use_context_provider(|| heir_options);

    // Check if the form is valid
    let all_valid = use_memo(move || {
        // Check if all heirs have a selected heir_id
        let all_heirs_selected = new_heritage_config
            .read()
            .heritages
            .iter()
            .all(|h| h.heir_config.is_some());

        // Check if minimum lock time is valid
        let min_lock_valid = {
            let min_lock = new_heritage_config.read().minimum_lock_time;
            min_lock >= 30 && min_lock <= 360
        };

        // Check that we have at least one heir
        let has_heirs = !new_heritage_config.read().heritages.is_empty();

        all_heirs_selected && min_lock_valid && has_heirs
    });

    // Function to add a new heir to the config
    let add_heir = move |_| {
        let heritages = &mut new_heritage_config.write().heritages;

        if heritages.is_empty() {
            heritages.push(HeritageState {
                heir_config: None,
                time_lock: 360, // Default to 360 days for first heir
            });
        } else {
            let previous_time_lock = heritages.last().expect("non empty vec").time_lock;
            heritages.push(HeritageState {
                heir_config: None,
                time_lock: previous_time_lock + 30, // Default to previous + 30 days
            });
        }
    };

    // Function to handle form submission
    // let submit_form = move |_| async move {
    //     // Set creating state to true to show loading UI
    //     *creating.write() = true;

    //     let Some(wallet) = &mut *wallet.write() else {
    //         // Should really never happen
    //         log::error!("The wallet resource is not loaded");
    //         alert_error("Could not upate the Heritage Configuration");
    //         return;
    //     };

    //     let new_hc = HeritageConfig::from(&*new_heritage_config.read());
    //     match wallet.online_wallet_mut().set_heritage_config(new_hc).await {
    //         Ok(new_hc) => {
    //             log::info!("Successfully created a new Heritage Configuration for the wallet");
    //             log::debug!("new_hc={new_hc:?}");
    //             alert_success("New Heritage Configuration created");
    //         }
    //         Err(e) => {
    //             log::error!("Failed to create a new Heritage Configuration for the wallet: {e}");
    //             alert_error(format!(
    //                 "Failed to create a new Heritage Configuration: {e}"
    //             ));
    //         }
    //     };
    // };
    let submit_form = move |_| async move {
        // Set creating state to true to show loading UI
        *creating.write() = true;

        let new_hc = HeritageConfig::from(&*new_heritage_config.read());

        match wallet
            .with_mut(|wallet: &mut Wallet| wallet.online_wallet_mut().set_heritage_config(new_hc))
            .await
        {
            Ok(new_hc) => {
                log::info!("Successfully created a new Heritage Configuration for the wallet");
                log::debug!("new_hc={new_hc:?}");
                alert_success("New Heritage Configuration created");
            }
            Err(e) => {
                log::error!("Failed to create a new Heritage Configuration for the wallet: {e}");
                alert_error(format!(
                    "Failed to create a new Heritage Configuration: {e}"
                ));
            }
        };
    };

    let action_text = if *has_heritage_configs.read() {
        "Update"
    } else {
        "Create"
    };

    // Render the form
    rsx! {
        div { class: "flex flex-col gap-4 pb-4",
            div { class: "flex flex-row justify-between",
                h2 { class: "text-2xl font-bold mb-4", "New Heritage Configuration" }
                button {
                    class: "btn btn-circle btn-outline btn-primary btn-lg",
                    onclick: move |_| *new_heritage_config_modal.write() = false,
                    svg {
                        class: "size-8 fill-current",
                        xmlns: "http://www.w3.org/2000/svg",
                        view_box: "0 0 24 24",
                        path { d: "M19,6.41L17.59,5L12,10.59L6.41,5L5,6.41L10.59,12L5,17.59L6.41,19L12,13.41L17.59,19L19,17.59L13.41,12L19,6.41Z" }
                    }
                }
            }
            // Heirs and Maturity Delays Section
            div { class: "card bg-base-100 shadow-sm",
                div { class: "card-body",
                    h3 { class: "card-title text-xl", "Heirs and Maturity Delays" }
                    div { class: "mb-2",
                        "Configure which heirs have access to the funds and after how many days."
                    }
                    div { class: "mb-4",
                        "Each heir must have a longer waiting time than the previous one."
                    }

                    div { class: "font-bold", "Heirs: {new_heritage_config.read().heritages.len()}" }

                    // Timeline of heirs
                    if new_heritage_config.read().heritages.is_empty() {
                        div { class: "alert",
                            "No heirs configured yet. Use the button below to add your first heir."
                        }
                    } else {
                        ul { class: "timeline timeline-vertical timeline-compact overflow-y-auto",
                            for (heritage_index , heritage_state) in new_heritage_config.read().heritages.iter().enumerate() {
                                NewHeirForm {
                                    heritage_index,
                                    heritage_state: heritage_state.clone(),
                                }
                            }
                        }
                    }

                    // Add heir button
                    div { class: "flex justify-center mt-4",
                        button { class: "btn btn-primary", onclick: add_heir,
                            svg {
                                class: "size-5 me-2",
                                xmlns: "http://www.w3.org/2000/svg",
                                view_box: "0 0 24 24",
                                fill: "none",
                                stroke: "currentColor",
                                stroke_width: "2",
                                stroke_linecap: "round",
                                stroke_linejoin: "round",
                                path { d: "M12 5v14M5 12h14" }
                            }
                            "Add Heir"
                        }
                    }
                }
            }
            // Reference Date Section
            div { class: "card bg-base-100 shadow-sm",
                div { class: "card-body",
                    h3 { class: "card-title text-xl", "Reference Date" }
                    div { class: "text-sm mb-4",
                        "The reference date is the starting point from which all maturity delays are calculated."
                    }
                    // Simple date input (a proper date picker component would be better)
                    input {
                        r#type: "date",
                        class: "input input-bordered",
                        value: timestamp_to_date_string(new_heritage_config.read().reference_ts),
                        min: timestamp_to_date_string(today),
                        onchange: move |evt| {
                            if let Ok(date) = NaiveDate::parse_from_str(&evt.value(), "%Y-%m-%d") {
                                new_heritage_config.write().reference_ts = date_to_noon_ts(date);
                            }
                        },
                    }
                
                }
            }
            // Minimum Maturity Delay Section
            div { class: "card bg-base-100 shadow-sm",
                div { class: "card-body",
                    h3 { class: "card-title text-xl", "Minimum Maturity Delay" }
                    div { class: "text-sm mb-4",
                        "The minimum time that must pass before any additional heirs can be added to this configuration."
                    }

                    div { class: "flex flex-row items-center gap-2 my-2",
                        span { "Minimum delay:" }
                        input {
                            r#type: "number",
                            class: "input input-bordered input-sm w-20",
                            min: "30",
                            max: "360",
                            value: "{new_heritage_config.read().minimum_lock_time}",
                            onchange: move |evt| {
                                if let Ok(value) = evt.value().parse::<u16>() {
                                    let clamped = value.clamp(30, 360);
                                    new_heritage_config.write().minimum_lock_time = clamped;
                                }
                            },
                        }
                        span { "days" }
                    }

                    // Slider for minimum lock time
                    div { class: "w-full mt-2 px-2",
                        div { class: "flex justify-between mb-1",
                            span { "Min: 30" }
                            span { "Max: 360" }
                        }
                        input {
                            r#type: "range",
                            class: "range range-primary range-sm w-full",
                            min: "30",
                            max: "360",
                            step: "10",
                            value: "{new_heritage_config.read().minimum_lock_time}",
                            oninput: move |evt| {
                                if let Ok(value) = evt.value().parse::<u16>() {
                                    new_heritage_config.write().minimum_lock_time = value;
                                }
                            },
                        }
                    }
                }
            }

            // Action buttons
            div { class: "flex gap-4 mt-4 justify-center",
                button {
                    class: "btn btn-primary",
                    disabled: !*all_valid.read() || *creating.read(),
                    onclick: submit_form,
                    svg {
                        class: "size-8 fill-current",
                        xmlns: "http://www.w3.org/2000/svg",
                        view_box: "0 0 24 24",
                        path { d: "M20.71,7.04C20.37,7.38 20.04,7.71 20.03,8.04C20,8.36 20.34,8.69 20.66,9C21.14,9.5 21.61,9.95 21.59,10.44C21.57,10.93 21.06,11.44 20.55,11.94L16.42,16.08L15,14.66L19.25,10.42L18.29,9.46L16.87,10.87L13.12,7.12L16.96,3.29C17.35,2.9 18,2.9 18.37,3.29L20.71,5.63C21.1,6 21.1,6.65 20.71,7.04M3,17.25L12.56,7.68L16.31,11.43L6.75,21H3V17.25Z" }
                    }
                    "{action_text}"
                }
                button {
                    class: "btn btn-outline btn-primary",
                    onclick: move |_| *new_heritage_config_modal.write() = false,
                    svg {
                        class: "size-8 fill-current",
                        xmlns: "http://www.w3.org/2000/svg",
                        view_box: "0 0 24 24",
                        path { d: "M12 2C17.5 2 22 6.5 22 12S17.5 22 12 22 2 17.5 2 12 6.5 2 12 2M12 4C10.1 4 8.4 4.6 7.1 5.7L18.3 16.9C19.3 15.5 20 13.8 20 12C20 7.6 16.4 4 12 4M16.9 18.3L5.7 7.1C4.6 8.4 4 10.1 4 12C4 16.4 7.6 20 12 20C13.9 20 15.6 19.4 16.9 18.3Z" }
                    }

                    "Cancel"
                }
            }
        }
    }
}

#[component]
fn NewHeirForm(heritage_index: usize, heritage_state: HeritageState) -> Element {
    let mut new_heritage_config = use_context::<Signal<HeritageConfigState>>();
    let new_heritage_config_heirs_duplicates = use_context::<Memo<HashSet<ArcType<HeirConfig>>>>();
    let min_max_time_locks = use_context::<Memo<Vec<TimeConstraint>>>();
    let heir_options = use_context::<Memo<HashMap<String, HeirOption>>>();

    let ref_ts = new_heritage_config.read().reference_ts;
    let maturity_date =
        UITimestamp::new_date_only(ref_ts + heritage_state.time_lock as u64 * 24 * 3600);

    let is_duplicate = heritage_state
        .heir_config
        .as_ref()
        .map(|id| new_heritage_config_heirs_duplicates.read().contains(id))
        .unwrap_or(false);

    let min_max_time_lock = min_max_time_locks.read()[heritage_index].clone();

    rsx! {
        li { class: "group pr-1",
            hr { class: "bg-base-content" }
            div { class: "timeline-middle",
                div { class: "bg-primary rounded-full aspect-square content-center",
                    span { class: "m-1 font-bold", "#{heritage_index + 1}" }
                }
            }
            div { class: "timeline-end timeline-box w-full",
                div { class: "flex flex-col p-4",
                    // Heir selection dropdown
                    div { class: "flex flex-row items-center gap-2 mb-2",
                        div { class: "grow",
                            select {
                                class: "select select-bordered w-full",
                                class: if is_duplicate { "select-error" },
                                value: heritage_state.heir_config.as_ref().map(|hc| hc.to_string()).unwrap_or_default(),
                                onchange: move |evt| {
                                    let heir_config_str = evt.value();
                                    if !heir_config_str.is_empty() {
                                        new_heritage_config.write().heritages[heritage_index].heir_config = Some(
                                            heir_options()
                                                .get(&heir_config_str)
                                                .expect("came from our own to_string")
                                                .heir_config
                                                .clone(),
                                        );
                                    } else {
                                        new_heritage_config.write().heritages[heritage_index].heir_config = None;
                                    }
                                },
                                NewHeirFormOptions { selected_heir_config: heritage_state.heir_config.clone() }
                            }

                            if is_duplicate {
                                div { class: "text-error mt-1", "This heir is already used" }
                            }
                        }

                        button {
                            class: "btn btn-circle btn-outline btn-primary btn-sm",
                            onclick: move |_| {
                                new_heritage_config.write().heritages.remove(heritage_index);
                            },
                            svg {
                                class: "size-4 fill-current",
                                xmlns: "http://www.w3.org/2000/svg",
                                view_box: "0 0 24 24",
                                path { d: "M19,6.41L17.59,5L12,10.59L6.41,5L5,6.41L10.59,12L5,17.59L6.41,19L12,13.41L17.59,19L19,17.59L13.41,12L19,6.41Z" }
                            }
                        }
                    }

                    // Maturity delay controls
                    div { class: "flex flex-row items-center gap-2 my-2",
                        span { "Maturity delay:" }
                        input {
                            r#type: "number",
                            class: "input input-bordered w-20",
                            min: min_max_time_lock.min,
                            max: min_max_time_lock.max,
                            value: heritage_state.time_lock,
                            onchange: move |evt| {
                                if let Ok(value) = evt.value().parse::<u16>() {
                                    let TimeConstraint { min, max, .. } = min_max_time_lock;
                                    let clamped = value.clamp(min, max);
                                    new_heritage_config.write().heritages[heritage_index].time_lock = clamped;
                                }
                            },
                        }
                        span { "days" }
                    }

                    // Slider for Maturity delay
                    div { class: "w-full mt-2 px-2",
                        div { class: "flex justify-between mb-1",
                            Tooltip { tooltip_text: min_max_time_lock.min_tt.clone(),
                                span { "Min: {min_max_time_locks.read()[heritage_index].min}" }
                            }
                            Tooltip { tooltip_text: min_max_time_lock.max_tt.clone(),
                                span { "Max: {min_max_time_locks.read()[heritage_index].max}" }
                            }
                        }
                        input {
                            r#type: "range",
                            class: "range range-primary range-sm w-full",
                            min: min_max_time_lock.min,
                            max: min_max_time_lock.max,
                            step: "30",
                            value: heritage_state.time_lock,
                            oninput: move |evt| {
                                if let Ok(value) = evt.value().parse::<u16>() {
                                    new_heritage_config.write().heritages[heritage_index].time_lock = value;
                                }
                            },
                        }
                    }

                    // Summary information
                    div { class: "grid grid-cols-2 gap-2 mt-4",
                        div {
                            div { class: "font-bold", "Delay" }
                            div { class: "text-base", "{heritage_state.time_lock} days" }
                        }
                        div {
                            div { class: "font-bold", "Maturity Date" }
                            div { class: "text-base",
                                AlwaysLoadedComponent { input: maturity_date }
                            }
                        }
                    }
                }
            }
            hr { class: "bg-base-content group-last:hidden" }
        }
    }
}

#[component]
fn NewHeirFormOptions(selected_heir_config: Option<ArcType<HeirConfig>>) -> Element {
    let heir_options = use_context::<Memo<HashMap<String, HeirOption>>>();
    rsx! {
        option {
            value: "",
            disabled: true,
            selected: selected_heir_config.is_none(),
            "Select an heir"
        }
        {
            heir_options
                .read()
                .iter()
                .map(|(id, heir_option)| {
                    let display_text = if let Some(email) = &heir_option.email {
                        format!("{} ({})", heir_option.name, email)
                    } else {
                        heir_option.name.to_string()
                    };
                    rsx! {
                        option {
                            key: "{id}",
                            value: id.clone(),
                            selected: selected_heir_config.as_ref() == Some(&heir_option.heir_config),
                            "{display_text}"
                        }
                    }
                })
        }
    }
}

// Helper function to get midnight today (start of day)
fn today_noon() -> u64 {
    date_to_noon_ts(Local::now().date_naive())
}

// Format a NaiveDate for display
fn date_to_noon_ts(date: NaiveDate) -> u64 {
    date.and_hms_opt(12, 0, 0)
        .expect("valid hour/min/sec values")
        .and_utc()
        .timestamp() as u64
}

// Structure to hold form state
#[derive(Debug)]
struct HeritageConfigState {
    reference_ts: u64,
    minimum_lock_time: u16,
    heritages: Vec<HeritageState>,
}
impl From<&HeritageConfigState> for HeritageConfig {
    fn from(value: &HeritageConfigState) -> Self {
        HeritageConfig::builder_v1()
            .reference_time(value.reference_ts)
            .minimum_lock_time(value.minimum_lock_time)
            .expand_heritages(value.heritages.iter().map(|h| {
                Heritage::new(
                    h.heir_config
                        .as_ref()
                        .expect("heir_config should be present before attempting conversion")
                        .as_ref()
                        .clone(),
                )
                .time_lock(h.time_lock)
            }))
            .build()
    }
}

#[derive(Debug, Clone, PartialEq)]
struct HeritageState {
    heir_config: Option<ArcType<HeirConfig>>,
    time_lock: u16,
}

// Structure for heir dropdown options
#[derive(Debug, Clone, PartialEq)]
struct HeirOption {
    id: ArcStr,
    name: ArcStr,
    email: Option<ArcStr>,
    heir_config: ArcType<HeirConfig>,
}

// Structure for time constraints
#[derive(Debug, Clone, PartialEq)]
struct TimeConstraint {
    min: u16,
    min_tt: ArcStr,
    max: u16,
    max_tt: ArcStr,
}
