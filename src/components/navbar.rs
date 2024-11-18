use crate::gui::Route;
use dioxus::prelude::*;

#[component]
pub fn NavBar() -> Element {
    let navigator = use_navigator();

    rsx! {
        nav { class: "h-full px-2 flex flex-row gap-2",
            div {
                onclick: move |_| {
                    navigator.push(Route::Home {});
                },
                class: "h-full flex flex-row gap-2 hover:cursor-pointer",
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
            }
            div { class: "basis-10" }
            NavLink { route: Route::WalletListView {}, "Wallets" }
            NavLink { route: Route::HeirListView {}, "Heirs" }
            NavLink { route: Route::InheritanceListView {}, "Inheritances" }
        }
    }
}

#[component]
fn NavLink(route: Route, children: Element) -> Element {
    rsx! {
        div { class: "basis-10 content-center flex",
            Link {
                class: "h-full px-4 content-center text-lg font-bold uppercase hover:bg-primary/5",
                active_class: "bg-primary/10 text-primary",
                to: route,
                { children }
            }
        }
    }
}
