use crate::prelude::*;

use btc_heritage_wallet::{
    bitcoin::Network,
    btc_heritage::{
        heritage_config::v1::Heritage, utils::bitcoin_network, HeirConfig, HeritageConfig,
    },
    OnlineWallet, Wallet,
};
use chrono::{Local, NaiveDate};

use std::collections::{HashMap, HashSet};

use crate::{
    components::{
        misc::TextTooltip,
        modal::CloseModalButton,
        svg::{
            Close, DrawSvg, Edit, PlusCircle,
            SvgSize::{Size4, Size5},
        },
        timestamp::UITimestamp,
    },
    utils::{timestamp_to_date_string, CCStr, CheapClone},
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
    existing_heritage_config: ReadOnlySignal<Option<CheapClone<HeritageConfig>>>,
    new_heritage_config_modal: Signal<bool>,
) -> Element {
    log::debug!("NewHeritageConfigForm Rendered");

    // Context resources
    let mut wallet = use_context::<AsyncSignal<Wallet>>();
    let heirs = use_context::<Memo<Vec<CompositeHeir>>>();

    // Form state
    let mut creating = use_signal(|| false);

    let today = today_noon();
    let min_min_lock = match bitcoin_network::get() {
        Network::Bitcoin => 10,
        _ => 1,
    };

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
        if let OnboardingStatus::InProgress(ref onboarding) =
            *state_management::ONBOARDING_STATUS.peek()
        {
            let heirs_by_name = heirs
                .read()
                .iter()
                .map(|ch| (ch.name.clone(), ch.heir_config.clone()))
                .collect::<HashMap<_, _>>();
            new_heritage_config.write().heritages = onboarding
                .context()
                .get(&OnboardingContextItemId::HeirName)
                .iter()
                .flat_map(|hs| hs.iter().map(|s| s.as_str()))
                .zip((365..).step_by(90))
                .map(|(heir_name, time_lock)| HeritageState {
                    heir_config: heirs_by_name.get(heir_name).cloned(),
                    time_lock,
                })
                .collect();
        }
        if let Some(current_config) = existing_heritage_config.read().as_ref() {
            let config_v1 = current_config
                .heritage_config_v1()
                .expect("Should be V1 config");

            // Create heritages data
            let heritages = config_v1
                .iter_heritages()
                .map(|h| HeritageState {
                    heir_config: Some(CheapClone::from(h.heir_config.clone())),
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
        for composite_heir in heirs.read().iter() {
            let name = composite_heir.name.clone();
            let heir_config = composite_heir.heir_config.clone();
            let id = heir_config.to_string();
            let email = composite_heir
                .service_heir
                .lmap(|sh| sh.lmap(|sh| CCStr::from(sh.main_contact.email.as_str())))
                .flatten();
            let heir_option = HeirOption {
                id: CCStr::from(&id),
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
            min_lock >= min_min_lock && min_lock <= 360
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

    // Function for the minlock_time
    let min_lock_time = move |evt: Event<FormData>| {
        if let Ok(value) = evt.parsed::<u16>() {
            let clamped = value.clamp(min_min_lock, 360);
            new_heritage_config.write().minimum_lock_time = clamped;
        }
    };

    // Function to handle form submission
    let submit_form = move |_| async move {
        // Set creating state to true to show loading UI
        *creating.write() = true;

        let new_hc = HeritageConfig::from(&*new_heritage_config.read());
        let op_result = wallet
            .with_mut(async |wallet: &mut Wallet| wallet.set_heritage_config(new_hc).await)
            .await;
        *creating.write() = false;

        match op_result {
            Ok(new_hc) => {
                log::info!("Successfully created a new Heritage Configuration for the wallet");
                log::debug!("new_hc={new_hc:?}");
                alert_success("New Heritage Configuration created");
                // Close the modal
                *new_heritage_config_modal.write() = false;
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
                            DrawSvg::<PlusCircle> { size: Size5 }
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
                        class: "input",
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
                        "The minimum time that heirs must wait after a new transaction is received
                        before being able to inherit it, even if the Heritage Configuration is completely
                        expired."
                    }

                    div { class: "flex flex-row items-center gap-2 my-2",
                        span { "Minimum delay:" }
                        input {
                            r#type: "number",
                            class: "input input-sm w-20",
                            min: "{min_min_lock}",
                            max: "360",
                            value: "{new_heritage_config.read().minimum_lock_time}",
                            onchange: min_lock_time,
                        }
                        span { "days" }
                    }

                    // Slider for minimum lock time
                    div { class: "w-full mt-2 px-2",
                        div { class: "flex justify-between mb-1",
                            span { "Min: {min_min_lock}" }
                            span { "Max: 360" }
                        }
                        input {
                            r#type: "range",
                            class: "range range-primary range-sm w-full",
                            // This is to ensure the slider steps are multiple of 10
                            min: "{min_min_lock - min_min_lock % 10}",
                            max: "360",
                            step: "10",
                            value: "{new_heritage_config.read().minimum_lock_time}",
                            oninput: min_lock_time,
                        }
                    }
                }
            }

            // Action buttons
            div { class: "flex gap-4 mt-4 justify-center",

                MaybeHighlight {
                    step: OnboardingStep::ClickCreateHeritageConfigurationButton2,
                    context_filter: consume_onboarding_context(),
                    button {
                        class: "btn btn-primary",
                        disabled: !*all_valid.read() || *creating.read(),
                        onclick: submit_form,
                        DrawSvg::<Edit> {}
                        "{action_text}"
                    }
                }
                CloseModalButton { signal: new_heritage_config_modal }
            }
        }
    }
}

#[component]
fn NewHeirForm(heritage_index: usize, heritage_state: HeritageState) -> Element {
    let mut new_heritage_config = use_context::<Signal<HeritageConfigState>>();
    let new_heritage_config_heirs_duplicates =
        use_context::<Memo<HashSet<CheapClone<HeirConfig>>>>();
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
                            DrawSvg::<Close> { size: Size4 }
                        }
                    }

                    // Maturity delay controls
                    div { class: "flex flex-row items-center gap-2 my-2",
                        span { "Maturity delay:" }
                        input {
                            r#type: "number",
                            class: "input w-20",
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
                            TextTooltip { tooltip_text: min_max_time_lock.min_tt.clone(),
                                span { "Min: {min_max_time_locks.read()[heritage_index].min}" }
                            }
                            TextTooltip { tooltip_text: min_max_time_lock.max_tt.clone(),
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
fn NewHeirFormOptions(selected_heir_config: Option<CheapClone<HeirConfig>>) -> Element {
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
                            selected: selected_heir_config == Some(heir_option.heir_config.clone()),
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
    heir_config: Option<CheapClone<HeirConfig>>,
    time_lock: u16,
}

// Structure for heir dropdown options
#[derive(Debug, Clone, PartialEq)]
struct HeirOption {
    id: CCStr,
    name: CCStr,
    email: Option<CCStr>,
    heir_config: CheapClone<HeirConfig>,
}

// Structure for time constraints
#[derive(Debug, Clone, PartialEq)]
struct TimeConstraint {
    min: u16,
    min_tt: CCStr,
    max: u16,
    max_tt: CCStr,
}
