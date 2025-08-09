#![windows_subsystem = "windows"]
mod components;
mod helper_hooks;
mod loaded;
mod onboarding;
mod state_management;
mod utils;
mod views;

mod prelude {
    pub use super::components::{
        alerts::{alert_error, alert_info, alert_success, alert_warn},
        onboarding::{MaybeHighlight, MaybeHighlightProgressType, OnboardingInfoModal},
    };
    pub use super::helper_hooks::prelude::*;
    pub use super::loaded::prelude::*;
    pub use super::onboarding::{
        consume_onboarding_context, OnboardingContextItem, OnboardingContextItemId, OnboardingStep,
    };
    pub use super::state_management::prelude::*;
    pub use dioxus::prelude::*;
}

use serde::{Deserialize, Serialize};

use components::alerts::AlertsContainer;
use prelude::*;

use utils::CCStr;
use views::{
    app_config::AppConfigView,
    heirs::{
        configuration::HeirConfigurationView,
        heir::{HeirView, HeirWrapperLayout},
        heir_create::HeirCreateView,
        heir_list::HeirListView,
        HeirsWrapperLayout,
    },
    heirwallet::{
        configuration::HeirWalletConfigurationView, spend::HeirWalletSpendView, HeirWalletView,
        HeirWalletWrapperLayout,
    },
    heirwallet_create::HeirWalletCreateView,
    heirwallet_list::HeirWalletListView,
    main_layout::MainLayout,
    onboarding::{
        OnboardingHowPrivateView, OnboardingHowPublicView, OnboardingLayout, OnboardingWhoView,
    },
    splashscreen::SplashScreenView,
    wallet::{
        configuration::WalletConfigurationView, spend::WalletSpendView, WalletView,
        WalletWrapperLayout,
    },
    wallet_create::WalletCreateView,
    wallet_list::WalletListView,
};

#[derive(Clone, Routable, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[rustfmt::skip]
pub enum Route {
    #[route("/")]
    SplashScreenView {},
    #[nest("/onboarding")]
    #[layout(OnboardingLayout)]
        #[route("/who")]
        OnboardingWhoView {},
        #[route("/how-public")]
        OnboardingHowPublicView {},
        #[route("/how-private")]
        OnboardingHowPrivateView {},
    #[end_layout]
    #[end_nest]
    #[nest("/main")]
    #[layout(MainLayout)]
        #[route("/config")]
        AppConfigView {},
        #[nest("/wallets")]
            #[route("/")]
            WalletListView {},
            #[route("/create")]
            WalletCreateView {},
            #[nest("/:wallet_name")]
            #[layout(WalletWrapperLayout)]
                #[route("/")]
                WalletView{ wallet_name: CCStr },
                #[route("/configuration")]
                WalletConfigurationView{ wallet_name: CCStr },
                #[route("/spend")]
                WalletSpendView{ wallet_name: CCStr },
            #[end_layout]
            #[end_nest]
        #[end_nest]
        #[nest("/heirs")]
        #[layout(HeirsWrapperLayout)]
            #[route("/")]
            HeirListView {},
            #[route("/create")]
            HeirCreateView {},
            #[nest("/:heir_index")]
            #[layout(HeirWrapperLayout)]
                #[route("/")]
                HeirView{ heir_index: usize },
                #[route("/configuration")]
                HeirConfigurationView{ heir_index: usize },
            #[end_layout]
            #[end_nest]
        #[end_layout]
        #[end_nest]
        #[nest("/heirwallet")]
            #[route("/")]
            HeirWalletListView {},
            #[route("/create")]
            HeirWalletCreateView {},
            #[nest("/:heirwallet_name")]
            #[layout(HeirWalletWrapperLayout)]
                #[route("/")]
                HeirWalletView{ heirwallet_name: CCStr },
                #[route("/configuration")]
                HeirWalletConfigurationView{ heirwallet_name: CCStr },
                #[route("/spend/:heritage_id")]
                HeirWalletSpendView{ heirwallet_name: CCStr, heritage_id:CCStr },
            #[end_layout]
            #[end_nest]
        #[end_nest]
    #[end_layout]
    #[end_nest]
    #[route("/:..route")]
    PageNotFound { route: Vec<String> },
}

static TITLE: &'static str = "Heritage Wallet";

#[allow(non_snake_case)]
fn App() -> Element {
    log::debug!("App reload");

    _ = crate::state_management::use_init_services();

    use_drop(|| log::debug!("App Dropped"));

    rsx! {
        document::Title { "{TITLE}" }
        document::Link { rel: "icon", href: asset!("/assets/crypto7world-logo.png") }
        document::Stylesheet { href: asset!("/assets/tailwind.css") }

        div {
            id: "app",
            class: "text-base",
            class: if matches!(prelude::state_management::THEME(), Theme::Dark) { "dark" },
            AlertsContainer {}
            Router::<Route> {}
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

fn main() {
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("warn"))
        .format_timestamp_micros()
        .init();

    log::info!("starting app");
    use dioxus::desktop::{tao::window::Icon, Config, WindowBuilder};
    LaunchBuilder::desktop()
        .with_cfg(
            Config::new().with_menu(None).with_window(
                WindowBuilder::new()
                    .with_title(TITLE)
                    .with_window_icon(Some(
                        Icon::from_rgba(
                            include_bytes!("../assets/crypto7world-logo.rgba").to_vec(),
                            256,
                            256,
                        )
                        .expect("image parse failed"),
                    ))
                    .with_inner_size(dioxus::desktop::LogicalSize::new(1920, 1080))
                    .with_maximized(true)
                    .with_resizable(true),
            ),
        )
        .launch(App)
}
