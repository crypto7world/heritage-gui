use dioxus::prelude::*;

use super::TitledView;
use crate::components::LoremIpsum;

#[component]
pub fn InheritanceListView() -> Element {
    rsx! {
        TitledView {
            title: "Inheritances",
            subtitle: "Inheritances in which other members referenced you.",
            LoremIpsum {}
        }
    }
}
