#![allow(non_snake_case)]
use dioxus::{
    desktop::{Config, LogicalSize, WindowBuilder},
    prelude::*,
};

use crate::{
    clients::get_userid,
    components::{Footer, NavBar},
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
            WalletView { wallet_name: String },
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

#[derive(Debug, Clone, Copy)]
pub struct DarkMode(bool);

impl DarkMode {
    pub fn get(&self) -> bool {
        self.0
    }
    pub fn set(&mut self, v: bool) {
        self.0 = v;
    }
}

#[derive(Debug, Clone, Copy)]
pub struct ScrollBlocking(bool);

impl ScrollBlocking {
    pub fn block(&mut self) {
        self.0 = true;
    }
    pub fn unblock(&mut self) {
        self.0 = false;
    }
}

fn App() -> Element {
    log::debug!("App reload");

    use_context_provider(|| Signal::new(DarkMode(true)));
    use_context_provider(|| Signal::new(ScrollBlocking(false)));
    use_context_provider(|| Signal::<_, SyncStorage>::new_maybe_sync(get_userid()));
    let dark_mode = use_context::<Signal<DarkMode>>();
    let scroll_blocking = use_context::<Signal<ScrollBlocking>>();

    rsx! {
        document::Title { "{TITLE}" }
        document::Link { rel: "icon", href: asset!("/assets/favicon.ico") }
        document::Stylesheet { href: asset!("/assets/tailwind.css") }

        div {
            id: "app",
            class: if scroll_blocking().0 { "no-doc-scroll" },
            class: if dark_mode().0 { "dark" },
            Router::<Route> {}
        }
    }
}

#[component]
fn MainView() -> Element {
    log::debug!("MainView reload");

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
