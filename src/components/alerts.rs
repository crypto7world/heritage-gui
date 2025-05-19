#![allow(unused)]

use dioxus::prelude::*;
use uuid::Uuid;

use std::collections::VecDeque;
use std::time::Duration;

use crate::utils::ArcStr;

pub static ALERTS: GlobalSignal<VecDeque<Alert>> = Signal::global(|| VecDeque::new());

/// Maximum number of alerts to display at once
const MAX_ALERTS: usize = 10;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum AlertClass {
    Success,
    Warn,
    Error,
    Info,
}
impl core::fmt::Display for AlertClass {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AlertClass::Success => write!(f, "alert-success"),
            AlertClass::Warn => write!(f, "alert-warning"),
            AlertClass::Error => write!(f, "alert-error"),
            AlertClass::Info => write!(f, "alert-info"),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Alert {
    uuid: Uuid,
    alert_class: AlertClass,
    title: ArcStr,
    message: ArcStr,
    timeout_ms: u64,
}
impl Alert {
    pub fn custom(
        alert_class: AlertClass,
        title: impl Into<ArcStr>,
        message: impl Into<ArcStr>,
        timeout_ms: u64,
    ) -> Self {
        Self {
            uuid: Uuid::new_v4(),
            alert_class,
            title: title.into(),
            message: message.into(),
            timeout_ms,
        }
    }
    pub fn success(message: impl Into<ArcStr>) -> Self {
        Self::custom(AlertClass::Success, "Success", message, 5000)
    }
    pub fn warn(message: impl Into<ArcStr>) -> Self {
        Self::custom(AlertClass::Warn, "Warning", message, 5000)
    }
    pub fn error(message: impl Into<ArcStr>) -> Self {
        Self::custom(AlertClass::Error, "Error", message, 5000)
    }
    pub fn info(message: impl Into<ArcStr>) -> Self {
        Self::custom(AlertClass::Info, "Info", message, 5000)
    }

    /// Creates a custom alert with a specified title
    pub fn with_title(mut self, title: impl Into<ArcStr>) -> Self {
        self.title = title.into();
        self
    }

    /// Changes the timeout for this alert
    pub fn with_timeout(mut self, timeout_ms: u64) -> Self {
        self.timeout_ms = timeout_ms;
        self
    }
}

// Helper functions to show alerts
pub fn add_alert(alert: Alert) {
    let mut alerts = ALERTS.write();
    // Remove oldest alerts if we've reached the maximum
    while alerts.len() >= MAX_ALERTS {
        alerts.pop_front();
    }
    alerts.push_back(alert);
}

pub fn alert_success(message: impl Into<ArcStr>) {
    add_alert(Alert::success(message));
}

pub fn alert_error(message: impl Into<ArcStr>) {
    add_alert(Alert::error(message));
}

pub fn alert_warning(message: impl Into<ArcStr>) {
    add_alert(Alert::warn(message));
}

pub fn alert_info(message: impl Into<ArcStr>) {
    add_alert(Alert::info(message));
}

#[component]
pub fn AlertsContainer() -> Element {
    rsx! {
        div { class: "fixed z-50 top-2 w-[90%] left-[5%] md:w-[60%] md:left-[20%] lg:w-1/2 lg:left-1/4",
            for alert in ALERTS().iter() {
                AlertDisplay { key: "{alert.uuid}", alert: alert.clone() }
            }
        }
    }
}

#[component]
pub fn AlertDisplay(alert: Alert) -> Element {
    log::debug!("AlertDisplay Rendered: {alert:?}");
    fn close_alert(uuid: Uuid) {
        ALERTS.write().retain(|a| a.uuid != uuid);
    }

    // Set up auto-dismiss
    spawn(async move {
        log::debug!("Countdown to close alert: {}", alert.uuid);
        tokio::time::sleep(Duration::from_millis(alert.timeout_ms)).await;
        log::debug!("Closing alert: {}", alert.uuid);
        close_alert(alert.uuid);
    });

    use_drop(move || log::debug!("AlertDisplay Dropped: {}", alert.uuid));

    rsx! {
        div {
            role: "alert",
            class: "alert p-1 rounded-xl mb-1 gap-1 md:gap-4 {alert.alert_class}",
            div { class: "flex text-sm col-start-1 col-span-12 sm:col-auto",
                svg {
                    xmlns: "http://www.w3.org/2000/svg",
                    class: "h-5 w-5 shrink-0 fill-current",
                    view_box: "0 0 24 24",
                    path { d: "M13,13H11V7H13M13,17H11V15H13M12,2A10,10 0 0,0 2,12A10,10 0 0,0 12,22A10,10 0 0,0 22,12A10,10 0 0,0 12,2Z" }
                }
                b { "{alert.title}" }
            }
            span { class: "text-xs col-start-1 col-span-11 sm:col-auto", "{alert.message}" }
            button {
                class: "btn btn-circle btn-outline btn-xs text-current col-start-12 col-span-1 sm:col-auto",
                onclick: move |_| close_alert(alert.uuid),
                svg {
                    xmlns: "http://www.w3.org/2000/svg",
                    class: "h-5 w-5",
                    view_box: "0 0 24 24",
                    path { d: "M19,6.41L17.59,5L12,10.59L6.41,5L5,6.41L10.59,12L5,17.59L6.41,19L12,13.41L17.59,19L19,17.59L13.41,12L19,6.41Z" }
                }
            }
        }
    }
}
