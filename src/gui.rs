#![allow(non_snake_case)]
use dioxus::{
    desktop::{Config, LogicalSize, WindowBuilder},
    prelude::*,
};

use crate::{
    components::{Footer, NavBar},
    views::{heirs::HeirListView, inheritances::InheritanceListView, wallets::WalletListView},
};

pub fn launch_gui() {
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
pub enum Route {
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
    use_context_provider(|| Signal::new(DarkMode(true)));
    use_context_provider(|| Signal::new(ScrollBlocking(false)));
    let dark_mode = use_context::<Signal<DarkMode>>();
    let data_theme = if dark_mode().0 { "dark" } else { "light" };
    let scroll_blocking = use_context::<Signal<ScrollBlocking>>();
    let block_scrolling = if scroll_blocking().0 {
        Some("no-doc-scroll")
    } else {
        None
    };
    rsx! {
        div { id: "app", "data-theme": "{data_theme}", class: block_scrolling, Router::<Route> {} }
    }
}

#[component]
fn MainView() -> Element {
    rsx! {
        div { class: "flex flex-col relative min-h-dvh",
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
