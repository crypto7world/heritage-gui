use crate::prelude::*;

use crate::{
    components::svg::{ArrowLeft, DrawSvg, SvgSize::Custom},
    utils::CCStr,
};

#[component]
pub fn TextTooltip(tooltip_text: CCStr, children: Element) -> Element {
    rsx! {
        div { class: "contents", title: "{tooltip_text}", {children} }
    }
}
#[component]
pub fn Tooltip(content: Element, children: Element) -> Element {
    rsx! {
        div { class: "tooltip before:text-xs before:text-white hover:after:delay-300 hover:before:delay-300",
            div { class: "tooltip-content", role: "tooltip", {content} }
            {children}
        }
    }
}

#[component]
pub fn Divider(children: Element) -> Element {
    rsx! {
        div { class: "divider text-base-content/60 text-base text-nowrap mt-6 mb-4",
            {children}
        }
    }
}

/// Reusable back button component for navigation
#[component]
pub fn BackButton(route: crate::Route) -> Element {
    let click_back = move |_| {
        navigator().push(route.clone());
    };

    rsx! {
        div { class: "h-full content-center",
            button {
                class: "btn btn-outline btn-primary btn-lg",
                onclick: click_back,
                DrawSvg::<ArrowLeft> { size: Custom("h-full") }
                "Back"
            }
        }
    }
}

#[component]
pub fn Teleport(children: Element) -> Element {
    let id = use_hook(|| uuid::Uuid::new_v4());

    log::debug!("Teleport {id} Rendered");

    use_effect(move || {
        document::eval(&format!(
            r#"
            const div_to_tp = document.getElementById("{id}");
            const div_app = document.getElementById("app");
            div_app.append(div_to_tp);
        "#
        ));
    });

    use_drop(move || {
        log::debug!("Teleport {id} Dropped");
        document::eval(&format!(
            r#"
            const div_to_tp = document.getElementById("{id}");
            const div_orig_parent = document.getElementById("parent_{id}");
            div_orig_parent.append(div_to_tp);
        "#
        ));
    });

    rsx! {
        div { id: "parent_{id}", class: "fixed",
            div { id: "{id}", {children} }
        }
    }
}

macro_rules! arcstr_loaded_elem {
    ($name:ident, $ph:literal) => {
        #[derive(Debug, Clone, PartialEq)]
        pub struct $name(CCStr);
        impl<T> From<T> for $name
        where
            CCStr: From<T>,
        {
            fn from(value: T) -> Self {
                Self(CCStr::from(value))
            }
        }
        impl LoadedElement for $name {
            type Loader = SkeletonLoader;
            #[inline(always)]
            fn element<M: LoadedComponentInputMapper>(self, _m: M) -> Element {
                rsx! {
                    {self.0}
                }
            }

            fn place_holder() -> Self {
                Self(CCStr::from($ph))
            }
        }
    };
}
arcstr_loaded_elem!(
    UITxId,
    "cac2bb7eec4960f5e048704ff4efe18cc6852b0fa291a267d9da56b8f56ddcc9"
);
arcstr_loaded_elem!(
    UIBtcAddr,
    "bc1prdsu09f3peyzk8yrupyw2vn73zn7kjda9vc3cejfhakljh8c2xcsuc6zkq"
);
