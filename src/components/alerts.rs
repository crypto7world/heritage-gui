#![allow(unused)]

use crate::prelude::*;

use uuid::Uuid;

use std::collections::VecDeque;
use std::time::Duration;

use crate::components::svg::{AlertCircle, Close, DrawSvg, SvgSize::Size5};
use crate::utils::CCStr;

static ALERTS: GlobalSignal<VecDeque<Alert>> = Signal::global(|| VecDeque::new());

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
        f.write_str(match self {
            Self::Success => "alert-success",
            Self::Warn => "alert-warning",
            Self::Error => "alert-error",
            Self::Info => "alert-info",
        })
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Alert {
    uuid: Uuid,
    alert_class: AlertClass,
    title: CCStr,
    message: CCStr,
    timeout_ms: u64,
}
impl Alert {
    pub fn custom(
        alert_class: AlertClass,
        title: impl Into<CCStr>,
        message: impl Into<CCStr>,
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
    pub fn success(message: impl Into<CCStr>) -> Self {
        Self::custom(AlertClass::Success, "Success", message, 5000)
    }
    pub fn warn(message: impl Into<CCStr>) -> Self {
        Self::custom(AlertClass::Warn, "Warning", message, 5000)
    }
    pub fn error(message: impl Into<CCStr>) -> Self {
        Self::custom(AlertClass::Error, "Error", message, 5000)
    }
    pub fn info(message: impl Into<CCStr>) -> Self {
        Self::custom(AlertClass::Info, "Info", message, 5000)
    }

    /// Creates a custom alert with a specified title
    pub fn with_title(mut self, title: impl Into<CCStr>) -> Self {
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

pub fn alert_success(message: impl Into<CCStr>) {
    add_alert(Alert::success(message));
}

pub fn alert_error(message: impl Into<CCStr>) {
    add_alert(Alert::error(message));
}

pub fn alert_warn(message: impl Into<CCStr>) {
    add_alert(Alert::warn(message));
}

pub fn alert_info(message: impl Into<CCStr>) {
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
                DrawSvg::<AlertCircle> { size: Size5 }
                b { "{alert.title}" }
            }
            span { class: "text-xs col-start-1 col-span-11 sm:col-auto", "{alert.message}" }
            button {
                class: "btn btn-circle btn-outline btn-xs col-start-12 col-span-1 sm:col-auto",
                onclick: move |_| close_alert(alert.uuid),
                DrawSvg::<Close> { size: Size5 }
            }
        }
    }
}
