use dioxus::prelude::*;

#[component]
pub fn Footer() -> Element {
    rsx! {
        div { class: "h-full text-primary text-right content-center", "2024 — Crypto7.world" }
    }
}
