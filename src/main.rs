#![allow(non_snake_case)]
mod components;
mod views;
use components::{Footer, NavBar};
use dioxus::{
    desktop::{Config, LogicalSize, WindowBuilder},
    prelude::*,
};
use dioxus_logger::tracing::{info, Level};
use views::{
    heirs::HeirListView, inheritances::InheritanceListView, wallets::WalletListView,
    TitledView,
};
#[derive(Clone, Routable, Debug, PartialEq)]
#[rustfmt::skip]
enum Route {
    #[layout(MainView)]
    #[route("/")]
    Home {},
    #[nest("/wallets")]
    #[layout(Wallets)]
    #[route("/")]
    Wallets {},
    #[end_layout]
    #[end_nest]
    #[nest("/heirs")]
    #[layout(Heirs)]
    #[route("/")]
    Heirs {},
    #[end_layout]
    #[end_nest]
    #[nest("/inheritances")]
    #[layout(Inheritances)]
    #[route("/")]
    Inheritances {},
    #[end_layout]
    #[end_nest]
    #[end_layout]
    #[route("/:..route")]
    PageNotFound { route: Vec<String> },
}
fn main() {
    dioxus_logger::init(Level::INFO).expect("failed to init logger");
    info!("starting app");
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
pub fn App() -> Element {
    rsx! {
        Router::<Route> {}
    }
}
#[component]
fn MainView() -> Element {
    rsx! {
        div { class: "flex flex-col relative min-h-dvh",
            header { class: "fixed top-0 w-full z-50 bg-white shadow-lg h-12", NavBar {} }
            main { class: "pt-12 pb-16 mx-8 text-justify", Outlet::<Route> {} }
            footer { class: "absolute bottom-0 w-full h-12 px-8 z-50 bg-white",
                div { class: "h-px border-t border-solid border-gray-200" }
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
fn Wallets() -> Element {
    rsx! {
        TitledView {
            title: "Wallets",
            subtitle: "Heritage wallets with simple Heritage configurations instead of complex Bitcoin scripts.",
            WalletListView {}
        }
    }
}
#[component]
fn Heirs() -> Element {
    rsx! {
        TitledView {
            title: "Heirs",
            subtitle: "Heirs that you can reference in the Heritage configuration of your wallets.",
            HeirListView {}
        }
    }
}
#[component]
fn Inheritances() -> Element {
    rsx! {
        TitledView {
            title: "Inheritances",
            subtitle: "Inheritances in which other members referenced you.",
            InheritanceListView {}
        }
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
