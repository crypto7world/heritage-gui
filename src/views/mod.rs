use crate::prelude::*;

use crate::{
    components::svg::{DrawSvg, PlusCircle, SvgSize::Size8},
    utils::CCStr,
    Route,
};

pub mod app_config;
pub mod heirs;
pub mod heirwallet;
pub mod heirwallet_create;
pub mod heirwallet_list;
pub mod main_layout;
pub mod onboarding;
pub mod splashscreen;
pub mod wallet;
pub mod wallet_create;
pub mod wallet_list;

#[component]
fn TitledView(
    title: CCStr,
    subtitle: CCStr,
    left: Option<Element>,
    right: Option<Element>,
    children: Element,
) -> Element {
    rsx! {
        div { class: "flex justify-evenly gap-4",
            div { class: "w-1/2 flex justify-start", {left} }
            div { class: "shrink-0",
                h1 { class: "text-6xl font-black text-center", {title} }
                h2 { class: "text-base font-light text-center", {subtitle} }
            }
            div { class: "w-1/2 flex justify-end", {right} }
        }
        div { class: "mb-4 h-px border-t border-solid border-gray-500" }
        {children}
    }
}

/// Reusable create button component for navigating to create views
#[component]
pub fn CreateLinkButton(route: Route, label: CCStr, size_classes: Option<CCStr>) -> Element {
    let click_create = move |_| {
        navigator().push(route.clone());
    };

    let size_classes = size_classes.unwrap_or_else(|| CCStr::from(""));

    rsx! {
        div {
            class: "card card-lg border-2 border-dashed border-base-300 shadow-xl cursor-pointer \
            transition-transform hover:scale-105 hover:border-primary {size_classes}",
            onclick: click_create,
            div { class: "card-body items-center justify-center",
                div { class: "btn btn-circle btn-primary btn-lg",
                    DrawSvg::<PlusCircle> { size: Size8 }
                }
                div { class: "text-lg font-bold mt-4", {label} }
            }
        }
    }
}
