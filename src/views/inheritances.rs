use dioxus::prelude::*;

use crate::components::misc::LoremIpsum;

#[component]
pub fn InheritanceListView() -> Element {
    rsx! {
        super::TitledView {
            title: "Inheritances",
            subtitle: "Inheritances in which other members referenced you.",
            LoremIpsum {}
        }
    }
}
