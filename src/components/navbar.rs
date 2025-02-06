use dioxus::prelude::*;

use crate::{
    components::service_connector::ServiceConnector,
    gui::{DarkMode, Route},
};

#[component]
pub fn NavBar() -> Element {
    log::debug!("NavBar reload");

    let navigator = use_navigator();

    rsx! {
        nav { class: "h-12 px-2 flex flex-row gap-2",
            div {
                onclick: move |_| {
                    navigator.push(Route::Home {});
                },
                class: "h-full flex flex-none gap-2 hover:cursor-pointer",
                img {
                    src: asset!("/assets/crypto7world-logo-v2-250.png"),
                    class: "self-center flex-none h-11",
                }
                div {
                    div { class: "text-lg font-black text-nowrap", "Heritage Wallet" }
                    div { class: "text-xs text-primary italic", "by Crypto7.world" }
                }
            }
            div { class: "basis-10" }
            NavLink { route: Route::WalletListView {}, "Wallets" }
            NavLink { route: Route::HeirListView {}, "Heirs" }
            NavLink { route: Route::InheritanceListView {}, "Inheritances" }
            div { class: "grow" }
            DarkModeToggle {}
            ServiceConnector {}
        }
    }
}

#[component]
fn NavLink(route: Route, children: Element) -> Element {
    rsx! {
        div { class: "basis-10 content-center flex",
            Link {
                class: "h-full px-4 content-center text-lg font-bold uppercase hover:bg-primary/10",
                active_class: "bg-primary/10 text-primary",
                to: route,
                {children}
            }
        }
    }
}

#[component]
pub fn DarkModeToggle() -> Element {
    let mut dark_mode = use_context::<Signal<DarkMode>>();

    rsx! {
        input {
            r#type: "checkbox",
            name: "theme",
            class: "theme-controller hidden",
            value: if dark_mode.read().get() { "dark" } else { "light" },
            tabindex: "-1",
            checked: true,
        }
        label { class: "swap swap-rotate",
            input {
                r#type: "checkbox",
                name: "theme",
                tabindex: "-1",
                oninput: move |event| dark_mode.write().set(event.checked()),
                checked: dark_mode.read().get(),
            }
            svg {
                class: "swap-off h-10 w-10 fill-current",
                xmlns: "http://www.w3.org/2000/svg",
                view_box: "0 0 24 24",
                path { d: "M5.64,17l-.71.71a1,1,0,0,0,0,1.41,1,1,0,0,0,1.41,0l.71-.71A1,1,0,0,0,5.64,17ZM5,12a1,1,0,0,0-1-1H3a1,1,0,0,0,0,2H4A1,1,0,0,0,5,12Zm7-7a1,1,0,0,0,1-1V3a1,1,0,0,0-2,0V4A1,1,0,0,0,12,5ZM5.64,7.05a1,1,0,0,0,.7.29,1,1,0,0,0,.71-.29,1,1,0,0,0,0-1.41l-.71-.71A1,1,0,0,0,4.93,6.34Zm12,.29a1,1,0,0,0,.7-.29l.71-.71a1,1,0,1,0-1.41-1.41L17,5.64a1,1,0,0,0,0,1.41A1,1,0,0,0,17.66,7.34ZM21,11H20a1,1,0,0,0,0,2h1a1,1,0,0,0,0-2Zm-9,8a1,1,0,0,0-1,1v1a1,1,0,0,0,2,0V20A1,1,0,0,0,12,19ZM18.36,17A1,1,0,0,0,17,18.36l.71.71a1,1,0,0,0,1.41,0,1,1,0,0,0,0-1.41ZM12,6.5A5.5,5.5,0,1,0,17.5,12,5.51,5.51,0,0,0,12,6.5Zm0,9A3.5,3.5,0,1,1,15.5,12,3.5,3.5,0,0,1,12,15.5Z" }
            }
            svg {
                class: "swap-on h-10 w-10 fill-current",
                xmlns: "http://www.w3.org/2000/svg",
                view_box: "0 0 24 24",
                path { d: "M21.64,13a1,1,0,0,0-1.05-.14,8.05,8.05,0,0,1-3.37.73A8.15,8.15,0,0,1,9.08,5.49a8.59,8.59,0,0,1,.25-2A1,1,0,0,0,8,2.36,10.14,10.14,0,1,0,22,14.05,1,1,0,0,0,21.64,13Zm-9.5,6.69A8.14,8.14,0,0,1,7.08,5.22v.27A10.15,10.15,0,0,0,17.22,15.63a9.79,9.79,0,0,0,2.1-.22A8.11,8.11,0,0,1,12.14,19.73Z" }
            }
        }
    }
}
