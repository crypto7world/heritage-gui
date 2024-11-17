use crate::Route;
use dioxus::prelude::*;

#[component]
pub fn NavBar() -> Element {
    rsx! {
        nav { class: "h-full px-2 flex flex-row gap-2",
            img {
                src: "assets/crypto7world-logo-v2-250.png",
                class: "self-center basis-10 h-11"
            }
            div { class: "basis-auto",
                div { class: "flex flex-col",
                    span { class: "text-lg font-black text-nowrap", "Heritage Wallet" }
                    span { class: "text-xs text-primary italic", "by Crypto7.world" }
                }
            }
            div { class: "basis-10" }
            NavLink { route: Route::Wallets {}, "Wallets" }
            NavLink { route: Route::Heirs {}, "Heirs" }
            NavLink { route: Route::Inheritances {}, "Inheritances" }
        }
    }
}
#[component]
fn NavLink(route: Route, children: Element) -> Element {
    rsx! {
        div { class: "basis-10 content-center",
            Link { active_class: "bg-primary/10 text-primary", to: route,
                div { class: "h-full px-4 content-center text-lg font-bold uppercase bg-inherit hover:bg-primary/5",
                    { children }
                }
            }
        }
    }
}
