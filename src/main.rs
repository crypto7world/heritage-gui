mod components;
mod helper_hooks;
mod state_management;
mod utils;
mod views;

use dioxus::prelude::*;

use utils::RcStr;
use views::{
    heir_list::HeirListView,
    home::Home,
    inheritances::InheritanceListView,
    main_layout::MainLayout,
    wallet::{configuration::WalletConfigurationView, WalletView, WalletWrapperLayout},
    wallet_list::WalletListView,
};

#[derive(Clone, Routable, Debug, PartialEq)]
#[rustfmt::skip]
pub enum Route {
    #[layout(MainLayout)]
        #[route("/")]
        Home {},
        #[nest("/wallets")]
            #[route("/")]
            WalletListView {},
            #[nest("/:wallet_name")]
            #[layout(WalletWrapperLayout)]
                #[route("/")]
                WalletView{ wallet_name: RcStr },
                #[route("/configuration")]
                WalletConfigurationView{ wallet_name: RcStr },
            #[end_layout]
            #[end_nest]
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

static TITLE: &'static str = "Heritage Wallet";
pub static DARK_MODE: GlobalSignal<bool> = Signal::global(|| true);

#[allow(non_snake_case)]
fn App() -> Element {
    log::debug!("App reload");

    _ = crate::state_management::use_init_services();

    use_drop(|| log::debug!("App Dropped"));

    rsx! {
        document::Title { "{TITLE}" }
        document::Link { rel: "icon", href: asset!("/assets/favicon.ico") }
        document::Stylesheet { href: asset!("/assets/tailwind.css") }

        div { id: "app", class: if DARK_MODE() { "dark" }, Router::<Route> {} }
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

fn main() {
    env_logger::Builder::from_env(
        env_logger::Env::default().default_filter_or("debug,tracing::span=warn"),
    )
    .format_timestamp_micros()
    .init();

    log::info!("starting app");

    LaunchBuilder::desktop()
        .with_cfg(
            dioxus::desktop::Config::new().with_window(
                dioxus::desktop::WindowBuilder::new()
                    .with_title(TITLE)
                    .with_inner_size(dioxus::desktop::LogicalSize::new(1920, 1080))
                    .with_resizable(true),
            ),
        )
        .launch(App)
}
