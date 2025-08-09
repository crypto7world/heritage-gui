use btc_heritage_wallet::btc_heritage::utils::timestamp_now;

use crate::prelude::*;

use crate::{
    components::{
        misc::TextTooltip,
        modal::Modal,
        svg::{
            AlertCircle, Close, DrawSvg, InfoCircle,
            SvgSize::{Size5, Size8},
        },
    },
    onboarding::Exclusive,
    utils::async_sleep,
};

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum MaybeHighlightProgressType {
    Click,
    Signal(ReadOnlySignal<bool>),
    ContextAdded(OnboardingContextItemId),
    /// Hover for the given amount of seconds to validate
    Hover(u64),
}

/// Component that highlights its content when the onboarding is at a specific step.
///
/// When the in-progress `Onboarding` is at the `OnboardingStepId` of the component,
/// the component will highlight its content until clicked. When clicked, it will
/// progress the onboarding to the next step.
///
/// The `context_filter`, if provided, is used to verify the onboarding context matches
/// before highlighting. The `context_provider` callback is used to provide context
/// from the current state of the parent component (e.g., extract the wallet name when
/// the user clicks on the "Create" button).
///
/// If provided, the `context_provider` result will be used to add context to the onboarding.
#[component]
pub fn MaybeHighlight(
    /// The onboarding step ID to match against
    step: OnboardingStep,
    /// Optional context filter to verify before highlighting
    context_filter: Option<OnboardingContextItem>,
    #[props(default = MaybeHighlightProgressType::Click)] progress: MaybeHighlightProgressType,
    /// Optional callback called before progressing, typically used to add context
    #[props(default = Callback::default())]
    context_callback: Callback<(), Option<(OnboardingContextItem, Exclusive)>>,
    /// Child elements to potentially highlight
    children: Element,
) -> Element {
    log::debug!("MaybeHighlight Rendered - Step ID: {:?}", step);

    // If no onboarding at all, just return the children
    match *state_management::ONBOARDING_STATUS.peek() {
        OnboardingStatus::Pending | OnboardingStatus::Completed => return children,
        OnboardingStatus::InProgress(_) => (),
    }

    // Check if this step should be highlighted
    let should_highlight = use_memo(move || match *state_management::ONBOARDING_STATUS.read() {
        OnboardingStatus::InProgress(ref onboarding) => {
            onboarding.is_active(step, context_filter.clone())
        }
        _ => false,
    });

    let onprogress = move || {
        if should_highlight() {
            if let OnboardingStatus::InProgress(ref mut onboarding) =
                *state_management::ONBOARDING_STATUS.write()
            {
                if let Some((ctx, exclusive)) = context_callback.call(()) {
                    onboarding.add_context(ctx, exclusive);
                };
                // Progress the onboarding
                onboarding.progress(step);
            }
        }
    };

    use_drop(move || log::debug!("MaybeHighlight Dropped - Step ID: {:?}", step));

    rsx! {
        GenericHightlight { should_highlight, progress, onprogress, {children} }
    }
}

#[component]
pub fn MaybeOnPathHighlight(
    /// The onboarding step IDs to match against
    steps: &'static [OnboardingStep],
    /// Optional context filter to verify before highlighting
    context_filter: Option<OnboardingContextItem>,
    #[props(default = MaybeHighlightProgressType::Click)] progress: MaybeHighlightProgressType,
    /// Child elements to potentially highlight
    children: Element,
) -> Element {
    log::debug!("MaybeOnPathHighlight Rendered - Step IDs: {:?}", steps);
    // If no onboarding at all, just return the children
    match *state_management::ONBOARDING_STATUS.peek() {
        OnboardingStatus::Pending | OnboardingStatus::Completed => return children,
        OnboardingStatus::InProgress(_) => (),
    }

    let mut stop_highlight = use_signal(|| false);
    // Check if this step should be highlighted
    let should_highlight = use_memo(move || {
        !stop_highlight()
            && match *state_management::ONBOARDING_STATUS.read() {
                OnboardingStatus::InProgress(ref onboarding) => steps
                    .iter()
                    .any(|step| onboarding.is_active(*step, context_filter.clone())),
                _ => false,
            }
    });

    let onprogress = move || {
        stop_highlight.set(true);
    };

    use_drop(move || log::debug!("MaybeOnPathHighlight Dropped"));

    rsx! {
        GenericHightlight { should_highlight, progress, onprogress, {children} }
    }
}

#[component]
fn GenericHightlight(
    /// The onboarding step IDs to match against
    should_highlight: ReadOnlySignal<bool>,
    progress: MaybeHighlightProgressType,
    onprogress: Callback,
    /// Child elements to potentially highlight
    children: Element,
) -> Element {
    let mut hover_started_ts = use_signal(|| None);
    use_future(move || async move {
        while let MaybeHighlightProgressType::Hover(wait_time_sec) = progress {
            if hover_started_ts().is_some_and(|ts| ts + wait_time_sec < timestamp_now()) {
                onprogress.call(())
            }
            tokio::time::sleep(tokio::time::Duration::from_millis(50)).await;
        }
    });

    let mut disable_highlight = use_signal(|| false);
    let onmouseover = move |_| async move {
        disable_highlight.set(true);
        async_sleep(5000).await;
        disable_highlight.set(false);
    };
    let onmouseenter = move |_| async move {
        *hover_started_ts.write() = Some(timestamp_now());
    };
    let onmouseleave = move |_| async move {
        *hover_started_ts.write() = None;
    };

    let onclick = move |_| {
        if matches!(progress, MaybeHighlightProgressType::Click) {
            onprogress.call(())
        }
    };

    let progress_effect = move || match progress {
        MaybeHighlightProgressType::Signal(read_only_signal) => {
            if read_only_signal() {
                onprogress.call(())
            }
        }
        MaybeHighlightProgressType::ContextAdded(onboarding_context_item_id) => {
            let read_guard = state_management::ONBOARDING_STATUS.read();
            if let OnboardingStatus::InProgress(ref onboarding) = *read_guard {
                if onboarding
                    .context()
                    .get(&onboarding_context_item_id)
                    .is_some()
                {
                    drop(read_guard);
                    onprogress.call(())
                }
            }
        }
        // Do Nothing
        MaybeHighlightProgressType::Click | MaybeHighlightProgressType::Hover(_) => (),
    };
    use_effect(progress_effect);
    use_drop(progress_effect);

    rsx! {
        div {
            class: "contents",
            class: if should_highlight() && !disable_highlight() { "highlight-children" },
            onclick,
            onmouseover,
            onmouseenter,
            onmouseleave,
            {children}
        }
    }
}

/// Component that displays the current onboarding message.
///
/// Reads the current `ONBOARDING_STATUS` and, if it is in progress, gets the current
/// message and displays it like an alert. The component is semi-transparent and
/// positioned to not hide useful UI components.
#[component]
pub fn OnboardingMessage() -> Element {
    log::debug!("OnboardingMessage Rendered");

    // Extract current message
    let current_message = use_memo(move || match *state_management::ONBOARDING_STATUS.read() {
        OnboardingStatus::InProgress(ref onboarding) => onboarding.current_message(),
        _ => None,
    });

    let mut hide = use_signal(|| false);

    use_drop(|| log::debug!("OnboardingMessage Dropped"));

    rsx! {
        if let Some(message) = current_message() {
            div {
                class: "fixed z-50 top-14 left-8",
                class: if hide() { "hidden" },
                div {
                    role: "alert",
                    class: "alert alert-info bg-(--alert-color)/75 p-1 grid-rows-[repeat(2,auto)] rounded-xl mb-1 gap-y-1 min-w-sm max-w-[max(var(--container-sm),25vw)]",
                    div { class: "flex gap-2 text-sm col-auto row-start-1",
                        DrawSvg::<AlertCircle> { size: Size5 }
                        b { "Onboarding" }
                    }
                    TextTooltip { tooltip_text: "Close 5 secs".into(),
                        button {
                            class: "btn btn-circle btn-outline btn-xs col-start-12 col-span-1",
                            onclick: move |_| async move {
                                hide.set(true);
                                crate::utils::async_sleep(5000).await;
                                hide.set(false);
                            },
                            DrawSvg::<Close> { size: Size5 }
                        }
                    }
                    span { class: "text-base col-span-12 row-start-2", "{message}" }
                }
            }
        }
    }
}

/// Modal component linked to an onboarding step.
///
/// The modal is displayed when the onboarding is at the specified step.
/// It can be closed by a "Got it!" button which will progress the onboarding
/// to the next step.
#[component]
pub fn OnboardingInfoModal(
    step: OnboardingStep,
    btn_text: Option<&'static str>,
    children: Element,
) -> Element {
    log::debug!("OnboardingInfoModal Rendered - Step ID: {:?}", step);

    // Check if this modal should be shown
    let should_show = use_memo(move || match *state_management::ONBOARDING_STATUS.read() {
        OnboardingStatus::InProgress(ref onboarding) => onboarding.is_active(step, None),
        _ => false,
    });
    let mut is_open = use_signal(|| false);
    use_effect(move || {
        *is_open.write() = should_show();
    });

    let handle_close = move |_| {
        log::debug!("OnboardingInfoModal closed - Step ID: {:?}", step);

        if let OnboardingStatus::InProgress(ref mut onboarding) =
            *state_management::ONBOARDING_STATUS.write()
        {
            onboarding.progress(step);
        }
    };

    let btn_text = btn_text.unwrap_or("Got it!");

    use_drop(move || log::debug!("OnboardingInfoModal Dropped - Step ID: {:?}", step));

    rsx! {
        Modal { is_open, persistent: true, higher_modal: true,
            // Modal header
            div { class: "flex flex-row justify-start items-center gap-4 mb-6",
                div { class: "text-info",
                    DrawSvg::<InfoCircle> { size: Size8 }
                }
                h2 { class: "text-2xl font-bold", "Onboarding Info" }
            }

            // Modal body
            {children}

            // Modal actions
            div { class: "modal-action",
                button { class: "btn btn-primary", onclick: handle_close, {btn_text} }
            }
        }
    }
}
