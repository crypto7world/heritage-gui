#![allow(non_snake_case)]

use dioxus::{
    desktop::{Config, LogicalSize, WindowBuilder},
    prelude::*,
};

use crate::{
    components::{Footer, NavBar},
    utils::ArcStr,
    views::{
        heir_list::HeirListView, inheritances::InheritanceListView, wallet::WalletView,
        wallet_list::WalletListView,
    },
};

static TITLE: &'static str = "Heritage Wallet";

pub fn launch_gui() {
    log::info!("starting app");

    LaunchBuilder::desktop()
        .with_cfg(
            Config::new().with_window(
                WindowBuilder::new()
                    .with_title(TITLE)
                    .with_inner_size(LogicalSize::new(1920, 1080))
                    .with_resizable(true),
            ),
        )
        .launch(App)
}

#[derive(Clone, Routable, Debug, PartialEq)]
#[rustfmt::skip]
pub enum Route {
    #[layout(MainView)]
        #[route("/")]
        Home {},
        #[nest("/wallets")]
            #[route("/")]
            WalletListView {},
            #[route("/:wallet_name")]
            WalletView { wallet_name: ArcStr },
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

pub static DARK_MODE: GlobalSignal<bool> = Signal::global(|| true);

fn App() -> Element {
    log::debug!("App reload");

    crate::state_management::init_services();

    use_drop(|| log::debug!("App Dropped"));

    rsx! {
        document::Title { "{TITLE}" }
        document::Link { rel: "icon", href: asset!("/assets/favicon.ico") }
        document::Stylesheet { href: asset!("/assets/tailwind.css") }

        div { id: "app", class: if DARK_MODE() { "dark" }, Router::<Route> {} }
    }
}

#[component]
fn MainView() -> Element {
    log::debug!("MainView reload");

    use_drop(|| log::debug!("MainView Dropped"));

    rsx! {
        div { class: "relative min-h-dvh",
            header { class: "bg-base-100 fixed top-0 w-full z-10 shadow-lg shadow-base-content/10",
                NavBar {}
            }
            main { class: "pt-12 pb-16 mx-8 text-justify", Outlet::<Route> {} }
            footer { class: "absolute bottom-px w-full h-12 px-8 z-0",
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
