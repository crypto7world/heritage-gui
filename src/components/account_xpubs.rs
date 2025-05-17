use btc_heritage_wallet::{
    heritage_service_api_client::AccountXPubWithStatus, AnyKeyProvider, Wallet,
};
use dioxus::prelude::*;

use crate::{
    components::badge::UIBadge,
    helper_hooks::use_resource_wallet_account_xpubs,
    loaded::prelude::*,
    utils::{ArcStr, ArcType},
};

#[derive(Debug, Clone, Copy, PartialEq)]
struct UIXPubStatusBadge(UIBadge);
impl LoadedElement for UIXPubStatusBadge {
    type Loader = SkeletonLoader;
    #[inline(always)]
    fn element<M: LoadedComponentInputMapper>(self, m: M) -> Element {
        self.0.element(m)
    }

    fn place_holder() -> Self {
        Self(UIBadge::place_holder())
    }
}
impl FromRef<AccountXPubWithStatus> for UIXPubStatusBadge {
    fn from_ref(status: &AccountXPubWithStatus) -> Self {
        Self(match status {
            AccountXPubWithStatus::Used(_) => UIBadge {
                text: "Used",
                color_class: "badge-success",
                tooltip: "This XPub has been consummed",
            },

            AccountXPubWithStatus::Unused(_) => UIBadge {
                text: "Unused",
                color_class: "badge-warning",
                tooltip: "This XPub is available for future use",
            },
        })
    }
}

#[derive(Debug, Clone, PartialEq)]
struct UIXPubRow {
    badge: UIXPubStatusBadge,
    descriptor: ArcStr,
}
impl LoadedElement for UIXPubRow {
    type Loader = TransparentLoader;
    #[inline(always)]
    fn element<M: LoadedComponentInputMapper>(self, m: M) -> Element {
        rsx! {
            tr { key: "{self.descriptor}",
                td {
                    LoadedComponent { input: m.map(self.badge) }
                }
                td {
                    div { class: "font-mono text-sm",
                        LoadedComponent { input: m.map(self.descriptor) }
                    }
                }
            
            }
        }
    }
    fn place_holder() -> Self {
        Self {
            badge: UIXPubStatusBadge::place_holder(),
            descriptor: ArcStr::place_holder(),
        }
    }
}
impl FromRef<AccountXPubWithStatus> for UIXPubRow {
    fn from_ref(status: &AccountXPubWithStatus) -> Self {
        let descriptor = match status {
            AccountXPubWithStatus::Used(account_xpub)
            | AccountXPubWithStatus::Unused(account_xpub) => ArcStr::from(account_xpub.to_string()),
        };
        let badge = UIXPubStatusBadge::from_ref(status);
        Self { badge, descriptor }
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
pub(super) fn AccountXPubConfig() -> Element {
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
                        LoadedComponent::<ArcType<[UIXPubRow]>> { input: account_xpubs.into() }
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
