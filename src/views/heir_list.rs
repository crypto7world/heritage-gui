use dioxus::prelude::*;

use super::TitledView;
use crate::hook_helpers;

#[component]
pub fn HeirListView() -> Element {
    log::debug!("HeirListView Rendered");

    use_drop(|| log::debug!("HeirListView Dropped"));
    rsx! {
        TitledView {
            title: "Heirs",
            subtitle: "Heirs that you can reference in the Heritage configuration of your wallets.",
            HeirList {}
        }
    }
}

#[component]
fn HeirList() -> Element {
    log::debug!("HeirList Rendered");

    let heir_names = hook_helpers::use_resource_heir_names();

    use_drop(|| log::debug!("HeirList Dropped"));
    rsx! {
        ul {
            if let Some(ref heir_names) = *heir_names.read() {
                for heir_name in heir_names {
                    li { key: "{heir_name}", "{heir_name}" }
                }
            }
        }
    }
}
