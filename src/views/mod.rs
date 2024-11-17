use dioxus::prelude::*;
pub mod heirs;
pub mod inheritances;
pub mod wallets;

#[component]
pub fn TitledView(title: String, subtitle: String, children: Element) -> Element {
    rsx! {
        h1 { class: "text-6xl font-black text-center", "{title}" }
        h2 { class: "text-base font-extralight text-center", "{subtitle}" }
        div { class: "mb-4 h-px border-t border-solid border-gray-200" }
        { children }
    }
}
