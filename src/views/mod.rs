use dioxus::prelude::*;

pub mod heir_list;
pub mod home;
pub mod inheritances;
pub mod main_layout;
pub mod wallet;
pub mod wallet_list;

#[component]
fn TitledView(
    title: String,
    subtitle: String,
    left: Option<Element>,
    right: Option<Element>,
    children: Element,
) -> Element {
    rsx! {
        div { class: "flex justify-evenly gap-4",
            div { class: "w-1/2 flex justify-start", {left} }
            div { class: "shrink-0",
                h1 { class: "text-6xl font-black text-center", "{title}" }
                h2 { class: "text-base font-light text-center", "{subtitle}" }
            }
            div { class: "w-1/2 flex justify-end", {right} }
        }
        div { class: "mb-4 h-px border-t border-solid border-gray-500" }
        {children}
    }
}
