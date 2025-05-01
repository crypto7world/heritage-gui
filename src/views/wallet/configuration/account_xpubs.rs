use btc_heritage_wallet::heritage_service_api_client::AccountXPubWithStatus;
use btc_heritage_wallet::{AnyKeyProvider, Wallet};
use dioxus::prelude::*;

use crate::components::misc::{Badge, BadgeType, Tooltip};
use crate::helper_hooks::use_resource_wallet_account_xpubs;
use crate::utils::RcStr;

/// Represents the status of an account extended public key.
#[derive(Debug, Clone, PartialEq)]
enum XPubStatus {
    Used,
    Unused,
}

impl XPubStatus {
    /// Returns the appropriate badge text for the status.
    fn badge_text(&self) -> &'static str {
        match self {
            XPubStatus::Used => "Used",
            XPubStatus::Unused => "Unused",
        }
    }

    /// Returns the CSS color class for the status badge.
    fn color_class(&self) -> &'static str {
        match self {
            XPubStatus::Used => "badge-success",
            XPubStatus::Unused => "badge-warning",
        }
    }

    /// Returns the tooltip text for the status badge.
    fn tooltip(&self) -> &'static str {
        match self {
            XPubStatus::Used => "This XPub is currently in use",
            XPubStatus::Unused => "This XPub is not currently in use",
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
/// A custom badge type for XPub status display.
struct XPubBadge(XPubStatus);

impl BadgeType for XPubBadge {
    fn text(&self) -> &'static str {
        self.0.badge_text()
    }

    fn color_class(&self) -> &'static str {
        self.0.color_class()
    }

    fn tooltip(&self) -> &'static str {
        self.0.tooltip()
    }
}

/// Component to configure and manage Account eXtended Public Keys.
///
/// Displays a table with the status and value of each account extended public key,
/// and provides buttons to add and autofeed more keys.
///
/// # Examples
///
/// ```
/// rsx! {
///     AccountXPubConfig {}
/// }
/// ```
#[component]
pub fn AccountXPubConfig() -> Element {
    let wallet = use_context::<Resource<Wallet>>();
    let account_xpubs = use_resource_wallet_account_xpubs(wallet);

    let can_autofeed = use_memo(move || {
        if let Some(wallet_data) = wallet.read().as_ref() {
            matches!(
                wallet_data.key_provider(),
                AnyKeyProvider::LocalKey(_) | AnyKeyProvider::Ledger(_)
            )
        } else {
            false
        }
    });

    let add_xpubs = move |_| {
        // Placeholder for adding XPubs
        log::info!("Add XPubs clicked");
    };

    let autofeed_xpubs = move |_| {
        // Placeholder for auto-feeding 20 more XPubs
        log::info!("Autofeed 20 more XPubs clicked");
    };

    rsx! {
        div { class: "container mx-auto p-4",
            h2 { class: "text-2xl font-bold mb-4", "Account Extended Public Keys" }

            // Table of XPubs
            div { class: "overflow-x-auto bg-base-100 rounded-lg shadow",
                table { class: "table w-full",
                    thead {
                        tr {
                            th { class: "w-1/6", "Status" }
                            th { "Descriptor" }
                        }
                    }
                    tbody {
                        {
                            match account_xpubs.cloned() {
                                Some(xpubs) => {
                                    if xpubs.is_empty() {
                                        rsx! {
                                            tr {
                                                td { class: "text-center", colspan: "2", "No XPubs configured" }
                                            }
                                        }
                                    } else {
                                        rsx! {
                                            for xpub in xpubs.iter() {
                                                {
                                                    let (status, descriptor) = match xpub {
                                                        AccountXPubWithStatus::Used(x) => (XPubStatus::Used, x.to_string()),
                                                        AccountXPubWithStatus::Unused(x) => (XPubStatus::Unused, x.to_string()),
                                                    };
                                                    rsx! {
                                                        tr { key: "{descriptor}",
                                                            td {
                                                                Badge { badge: XPubBadge(status) }
                                                            }
                                                            td {
                                                                div { class: "font-mono text-sm truncate max-w-xs hover:text-clip",
                                                                    Tooltip { tooltip_text: RcStr::from(&descriptor),
                                                                        span { "{descriptor}" }
                                                                    }
                                                                }
                                                            }
                                                        }
                                                    }
                                                }
                                            }
                                        }
                                    }
                                }
                                None => {
                                    rsx! {
                                        tr {
                                            td { class: "animate-pulse", colspan: "2",
                                                div { class: "h-4 bg-base-300 rounded w-full" }
                                            }
                                        }
                                        tr {
                                            td { class: "animate-pulse", colspan: "2",
                                                div { class: "h-4 bg-base-300 rounded w-full" }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }

            // Action buttons
            div { class: "flex gap-4 mt-4",
                button { class: "btn btn-primary", onclick: add_xpubs, "Add XPubs" }

                if *can_autofeed.read() {
                    button { class: "btn btn-secondary", onclick: autofeed_xpubs, "Autofeed 20 more" }
                }
            }
        }
    }
}
