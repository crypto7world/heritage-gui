use btc_heritage_wallet::{DatabaseItem, Heir};
use dioxus::prelude::*;

use super::TitledView;
use crate::{clients::database, utils::log_error};

#[component]
pub fn HeirListView() -> Element {
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
    let heir_names = Heir::list_names(&database())
        .map_err(log_error)
        .unwrap_or_default();
    rsx! {
        ul {
            for heir_name in heir_names {
                li { { heir_name } }
            }
        }
    }
}
