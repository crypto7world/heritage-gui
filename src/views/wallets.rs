use crate::components::LoremIpsum;
use dioxus::prelude::*;

#[component]
pub fn WalletListView() -> Element {
    rsx! {
        LoremIpsum {}
    }
}
