use crate::components::LoremIpsum;
use dioxus::prelude::*;

#[component]
pub fn HeirListView() -> Element {
    rsx! {
        LoremIpsum {}
    }
}
