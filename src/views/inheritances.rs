use crate::components::LoremIpsum;
use dioxus::prelude::*;

#[component]
pub fn InheritanceListView() -> Element {
    rsx! {
        LoremIpsum {}
    }
}
