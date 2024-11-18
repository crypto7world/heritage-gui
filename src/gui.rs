#![allow(non_snake_case)]

use dioxus::{
    desktop::{Config, LogicalSize, WindowBuilder},
    prelude::*,
};

use crate::{
    components::{Footer, NavBar},
    views::{heirs::HeirListView, inheritances::InheritanceListView, wallets::WalletListView},
};

pub(crate) fn launch_gui() {
    log::info!("starting app");
    LaunchBuilder::desktop()
        .with_cfg(
            Config::new()
                .with_window(
                    WindowBuilder::new()
                        .with_title("Heritage Wallet")
                        .with_inner_size(LogicalSize::new(1920, 1080))
                        .with_resizable(true),
                )
                .with_custom_head(
                    r#"<link rel="stylesheet" href="assets/tailwind.css">"#.to_string(),
                ),
        )
        .launch(App)
}

#[derive(Clone, Routable, Debug, PartialEq)]
#[rustfmt::skip]
pub(crate) enum Route {
    #[layout(MainView)]
    #[route("/")]
    Home {},
    #[nest("/wallets")]
    #[route("/")]
    WalletListView {},
    #[end_nest]
    #[nest("/heirs")]
    #[route("/")]
    HeirListView {},
    #[end_nest]
    #[nest("/inheritances")]
    #[route("/")]
    InheritanceListView {},
    #[end_nest]
    #[end_layout]
    #[route("/:..route")]
    PageNotFound { route: Vec<String> },
}

fn App() -> Element {
    rsx! {
        div { id: "app", class: "light", Router::<Route> {} }
    }
}

#[component]
fn MainView() -> Element {
    rsx! {
        div { class: "bg-back text-front flex flex-col relative min-h-dvh",
            header { class: "bg-back fixed top-0 w-full z-50 shadow-lg shadow-front/10 h-12",
                NavBar {}
            }
            main { class: "pt-12 pb-16 mx-8 text-justify", Outlet::<Route> {} }
            footer { class: "absolute bottom-0 w-full h-12 px-8 z-50 ",
                div { class: "h-px border-t border-solid border-gray-500" }
                Footer {}
            }
        }
    }
}

#[component]
fn Home() -> Element {
    rsx! {
        p { "Welcome" }
    }
}

#[component]
fn PageNotFound(route: Vec<String>) -> Element {
    rsx! {
        h1 { "Page not found" }
        p { "We are terribly sorry, but the page you requested doesn't exist." }
        pre { color: "red", "log:\nattemped to navigate to: {route:?}" }
    }
}
