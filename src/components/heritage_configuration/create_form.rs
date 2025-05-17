use btc_heritage_wallet::btc_heritage::HeritageConfig;
use chrono::{DateTime, Local, NaiveDate, Utc};
use dioxus::prelude::*;
use std::collections::HashSet;

use crate::{
    components::{
        misc::{Modal, Tooltip},
        timestamp::UITimestamp,
    },
    helper_hooks::{use_memo_heirs, use_resource_database_heirs, use_resource_service_heirs},
    loaded::prelude::*,
    utils::{ArcStr, ArcType},
};

/// Form for creating or updating a heritage configuration.
///
/// This component provides a user interface for configuring inheritance settings,
/// including:
/// - Setting the reference date
/// - Adding heirs and their maturity delays
/// - Configuring minimum maturity times
#[component]
pub fn NewHeritageConfigForm() -> Element {
    log::debug!("NewHeritageConfigForm Rendered");

    // Context resources
    let wallet_heritage_configs = use_context::<Resource<ArcType<[ArcType<HeritageConfig>]>>>();
    let database_heirs = use_resource_database_heirs();
    let service_heirs = use_resource_service_heirs();
    let heirs = use_memo_heirs(database_heirs, service_heirs);

    // Form state
    let mut date_panel_open = use_signal(|| false);
    let mut creating = use_signal(|| false);

    // Initial data
    let today = today_midnight();

    // Heritage config state
    let mut new_heritage_config = use_signal(|| HeritageConfigState {
        reference_date: today,
        minimum_lock_time: 30,
        heritages: vec![],
    });

    // Compute if we already have heritage configs
    let has_heritage_configs = use_memo(move || {
        if let Some(configs) = wallet_heritage_configs.cloned() {
            return !configs.is_empty();
        }
        false
    });

    // Get the current heritage config if it exists
    let current_heritage_config = use_memo(move || {
        if let Some(configs) = wallet_heritage_configs.cloned() {
            if !configs.is_empty() {
                return Some(configs[0].clone());
            }
        }
        None
    });

    // Initialize form with current config data if available
    use_effect(move || {
        if let Some(current_config) = current_heritage_config() {
            let config_v1 = current_config
                .heritage_config_v1()
                .expect("Should be V1 config");

            // Convert timestamp to local date
            let timestamp_seconds = config_v1.reference_timestamp.as_u64() as i64;
            let datetime = DateTime::<Utc>::from_timestamp(timestamp_seconds, 0)
                .expect("Invalid timestamp")
                .with_timezone(&Local);

            // Create heritages data
            let heritages = config_v1
                .iter_heritages()
                .map(|h| {
                    // Find heir from heir config
                    let heir_id = heirs.read().iter().find_map(|(_, composite_heir)| {
                        if composite_heir.heir_config.as_ref() == &h.heir_config {
                            if let Some(service_heir) = &composite_heir.service_heir {
                                return Some(service_heir.id.clone());
                            }
                        }
                        None
                    });

                    HeritageState {
                        heir_id,
                        time_lock: h.time_lock.as_u16(),
                    }
                })
                .collect::<Vec<_>>();

            *new_heritage_config.write() = HeritageConfigState {
                reference_date: datetime.date_naive(),
                minimum_lock_time: config_v1.minimum_lock_time.as_days().as_u16(),
                heritages,
            };
        } else {
            // Default values for new config
            *new_heritage_config.write() = HeritageConfigState {
                reference_date: today,
                minimum_lock_time: 30,
                heritages: vec![],
            };
        }
    });

    // Detect duplicate heirs
    let new_heritage_config_heirs_duplicates = use_memo(move || {
        log::debug!("use_memo_new_heritage_config_heirs_duplicates - start compute");
        let heritages = &new_heritage_config.read().heritages;
        let mut already_seen_heir_ids = HashSet::new();
        let mut duplicates = HashSet::new();

        for heritage in heritages {
            if let Some(heir_id) = &heritage.heir_id {
                if already_seen_heir_ids.contains(heir_id) {
                    duplicates.insert(heir_id.clone());
                } else {
                    already_seen_heir_ids.insert(heir_id.clone());
                }
            }
        }

        log::debug!("use_memo_new_heritage_config_heirs_duplicates - finish compute");

        duplicates
    });

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

    // Get available heirs for dropdown
    let heir_list = use_memo(move || {
        let mut lst = Vec::new();
        for composite_heir in heirs.read().values() {
            if let Some(service_heir) = &composite_heir.service_heir {
                lst.push(HeirOption {
                    name: composite_heir.name.clone(),
                    email: service_heir.main_contact.email.to_string(),
                    heir_id: service_heir.id.clone(),
                });
            }
        }
        lst
    });

    // Check if the form is valid
    let all_valid = use_memo(move || {
        // Check if all heirs have a selected heir_id
        let all_heirs_selected = new_heritage_config
            .read()
            .heritages
            .iter()
            .all(|h| h.heir_id.is_some());

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
                heir_id: None,
                time_lock: 360, // Default to 360 days for first heir
            });
        } else {
            let previous_time_lock = heritages.last().expect("non empty vec").time_lock;
            heritages.push(HeritageState {
                heir_id: None,
                time_lock: previous_time_lock + 30, // Default to previous + 30 days
            });
        }
    };

    // Function to remove an heir from the config
    let mut remove_heir = move |index: usize| {
        let heritages = &mut new_heritage_config.write().heritages;
        heritages.remove(index);
    };

    // Function to handle form submission
    let submit_form = move |_| {
        // Set creating state to true to show loading UI
        *creating.write() = true;

        // In a real implementation, this would call your API
        spawn(async move {
            // Simulate API call delay
            tokio::time::sleep(std::time::Duration::from_secs(1)).await;

            log::info!(
                "Would create/update heritage config: {:?}",
                *new_heritage_config.read()
            );

            // Reset creating state
            *creating.write() = false;
        });
    };

    let action_text = if *has_heritage_configs.read() {
        "Update"
    } else {
        "Create"
    };

    // Render the form
    rsx! {
        div { class: "flex flex-col gap-4 w-full",
            // Reference Date Section
            div { class: "card bg-base-100 shadow-sm",
                div { class: "card-body",
                    h3 { class: "card-title text-xl", "Reference Date" }
                    div { class: "text-sm mb-4",
                        "The reference date is the starting point from which all maturity delays are calculated."
                    }

                    button {
                        class: "btn btn-primary",
                        onclick: move |_| *date_panel_open.write() = true,
                        "{format_date(new_heritage_config.read().reference_date)}"

                        Modal { is_open: date_panel_open, persistent: false,
                            div { class: "flex flex-col gap-4 p-4",
                                h3 { class: "text-lg font-bold", "Select Reference Date" }

                                // Simple date input (a proper date picker component would be better)
                                input {
                                    r#type: "date",
                                    class: "input input-bordered",
                                    value: new_heritage_config.read().reference_date.format("%Y-%m-%d").to_string(),
                                    min: today.format("%Y-%m-%d").to_string(),
                                    onchange: move |evt| {
                                        if let Ok(date) = NaiveDate::parse_from_str(&evt.value(), "%Y-%m-%d") {
                                            new_heritage_config.write().reference_date = date;
                                        }
                                    },
                                }

                                div { class: "flex justify-center",
                                    button {
                                        class: "btn btn-primary",
                                        onclick: move |_| *date_panel_open.write() = false,
                                        "Done"
                                    }
                                }
                            }
                        }
                    }
                }
            }

            // Heirs and Maturity Delays Section
            div { class: "card bg-base-100 shadow-sm",
                div { class: "card-body",
                    h3 { class: "card-title text-xl", "Heirs and Maturity Delays" }
                    div { class: "text-sm mb-2",
                        "Configure which heirs have access to the funds and after how many days."
                    }
                    div { class: "text-sm mb-4",
                        "Each heir must have a longer waiting time than the previous one."
                    }

                    div { class: "text-base font-bold mb-2",
                        "Heirs: {new_heritage_config.read().heritages.len()}"
                    }

                    // Timeline of heirs
                    if new_heritage_config.read().heritages.is_empty() {
                        div { class: "alert",
                            "No heirs configured yet. Use the button below to add your first heir."
                        }
                    } else {
                        ul { class: "timeline timeline-vertical timeline-compact",
                            {
                                new_heritage_config
                                    .read()
                                    .heritages
                                    .iter()
                                    .enumerate()
                                    .map(|(index, heritage)| {
                                        let heritage_index = index;
                                        let ref_ts = new_heritage_config
                                            .read()
                                            .reference_date
                                            .and_hms_opt(12, 0, 0)
                                            .expect("valid hour/min/sec values")
                                            .and_utc()
                                            .timestamp();
                                        let maturity_ts = ref_ts + (heritage.time_lock as i64 * 24 * 3600);
                                        let maturity_date = UITimestamp::new_date_only(maturity_ts as u64);
                                        let is_duplicate = heritage
                                            .heir_id
                                            .as_ref()
                                            .map(|id| new_heritage_config_heirs_duplicates.read().contains(id))
                                            .unwrap_or(false);
                                        rsx! {
                                            li { class: "w-fit group",
                                                hr { class: "bg-base-content" }
                                                div { class: "timeline-middle",
                                                    div { class: "bg-primary rounded-full aspect-square flex items-center justify-center",
                                                        span { class: "m-1 font-bold", "#{index + 1}" }
                                                    }
                                                }
                                                div { class: "timeline-end timeline-box",
                                                    div { class: "card-body p-4",
                                                        // Heir selection dropdown
                                                        div { class: "flex flex-row items-center gap-2 mb-2",
                                                            div { class: "grow",
                                                                select {
                                                                    class: if is_duplicate { "select select-bordered select-error w-full" } else { "select select-bordered w-full" },
                                                                    value: heritage.heir_id.clone().unwrap_or_default(),
                                                                    onchange: move |evt| {
                                                                        let heir_id = evt.value();
                                                                        if !heir_id.is_empty() {
                                                                            new_heritage_config.write().heritages[heritage_index].heir_id = Some(
                                                                                heir_id,
                                                                            );
                                                                        } else {
                                                                            new_heritage_config.write().heritages[heritage_index].heir_id = None;
                                                                        }
                                                                    },
                                                                    option {
                                                                        value: "",
                                                                        disabled: true,
                                                                        selected: heritage.heir_id.is_none(),
                                                                        "Select an heir"
                                                                    }
                                                                    {
                                                                        heir_list
                                                                            .read()
                                                                            .iter()
                                                                            .map(|heir| {
                                                                                let display_text = format!("{} ({})", heir.name, heir.email);
                                                                                rsx! {
                                                                                    option {
                                                                                        key: "{heir.heir_id}",
                                                                                        value: "{heir.heir_id}",
                                                                                        selected: heritage.heir_id.as_ref() == Some(&heir.heir_id),
                                                                                        "{display_text}"
                                                                                    }
                                                                                }
                                                                            })
                                                                    }
                                                                }
                                            
                                                                if is_duplicate {
                                                                    div { class: "text-error text-xs mt-1", "This heir is already selected" }
                                                                }
                                                            }
                                            
                                                            button {
                                                                class: "btn btn-circle btn-sm",
                                                                onclick: move |_| remove_heir(heritage_index),
                                                                "×"
                                                            }
                                                        }
                                            
                                                        // Maturity delay controls
                                                        div { class: "flex flex-row items-center gap-2 my-2",
                                                            span { "Maturity delay:" }
                                                            input {
                                                                r#type: "number",
                                                                class: "input input-bordered input-sm w-20",
                                                                min: "{min_max_time_locks.read()[heritage_index].min}",
                                                                max: "{min_max_time_locks.read()[heritage_index].max}",
                                                                value: "{heritage.time_lock}",
                                                                onchange: move |evt| {
                                                                    if let Ok(value) = evt.value().parse::<u16>() {
                                                                        let min = min_max_time_locks.read()[heritage_index].min;
                                                                        let max = min_max_time_locks.read()[heritage_index].max;
                                                                        let clamped = value.clamp(min, max);
                                                                        new_heritage_config.write().heritages[heritage_index].time_lock = clamped;
                                                                    }
                                                                },
                                                            }
                                                            span { "days" }
                                                        }
                                            
                                                        // Slider for time lock
                                                        div { class: "w-full mt-2 px-2",
                                                            div { class: "flex justify-between text-xs mb-1",
                                                                Tooltip { tooltip_text: ArcStr::from(min_max_time_locks.read()[heritage_index].min_tt.as_str()),
                                                                    span { "Min: {min_max_time_locks.read()[heritage_index].min}" }
                                                                }
                                                                Tooltip { tooltip_text: ArcStr::from(min_max_time_locks.read()[heritage_index].max_tt.as_str()),
                                                                    span { "Max: {min_max_time_locks.read()[heritage_index].max}" }
                                                                }
                                                            }
                                                            input {
                                                                r#type: "range",
                                                                class: "range range-primary range-sm",
                                                                min: "{min_max_time_locks.read()[heritage_index].min}",
                                                                max: "{min_max_time_locks.read()[heritage_index].max}",
                                                                step: "30",
                                                                value: "{heritage.time_lock}",
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
                                                                div { class: "font-bold text-sm", "Delay" }
                                                                div { class: "text-base", "{heritage.time_lock} days" }
                                                            }
                                                            div {
                                                                div { class: "font-bold text-sm", "Maturity Date" }
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
                                    })
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
                        div { class: "flex justify-between text-xs mb-1",
                            span { "Min: 30" }
                            span { "Max: 360" }
                        }
                        input {
                            r#type: "range",
                            class: "range range-primary range-sm",
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
            div { class: "flex justify-center gap-4 mt-4",
                button {
                    class: "btn btn-primary btn-lg",
                    disabled: !*all_valid.read() || *creating.read(),
                    onclick: submit_form,
                    svg {
                        class: "size-6 me-2",
                        xmlns: "http://www.w3.org/2000/svg",
                        view_box: "0 0 24 24",
                        fill: "none",
                        stroke: "currentColor",
                        stroke_width: "2",
                        stroke_linecap: "round",
                        stroke_linejoin: "round",
                        path { d: "M20.71,7.04C20.37,7.38 20.04,7.71 20.03,8.04C20,8.36 20.34,8.69 20.66,9C21.14,9.5 21.61,9.95 21.59,10.44C21.57,10.93 21.06,11.44 20.55,11.94L16.42,16.08L15,14.66L19.25,10.42L18.29,9.46L16.87,10.87L13.12,7.12L16.96,3.29C17.35,2.9 18,2.9 18.37,3.29L20.71,5.63C21.1,6 21.1,6.65 20.71,7.04M3,17.25L12.56,7.68L16.31,11.43L6.75,21H3V17.25Z" }
                    }
                    "{action_text}"
                }
            }
        }
    }
}

// Helper function to get midnight today (start of day)
fn today_midnight() -> NaiveDate {
    Local::now().date_naive()
}

// Format a NaiveDate for display
fn format_date(date: NaiveDate) -> String {
    date.format("%B %d, %Y").to_string()
}

// Structure to hold form state
#[derive(Debug)]
struct HeritageConfigState {
    reference_date: NaiveDate,
    minimum_lock_time: u16,
    heritages: Vec<HeritageState>,
}

#[derive(Debug)]
struct HeritageState {
    heir_id: Option<String>,
    time_lock: u16,
}

// Structure for heir dropdown options
#[derive(Debug, PartialEq)]
struct HeirOption {
    name: ArcStr,
    email: String,
    heir_id: String,
}

// Structure for time constraints
#[derive(Debug, PartialEq)]
struct TimeConstraint {
    min: u16,
    min_tt: String,
    max: u16,
    max_tt: String,
}
