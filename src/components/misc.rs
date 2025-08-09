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

#[component]
pub fn LoremIpsum() -> Element {
    rsx! {
        p {
            "Lorem ipsum dolor sit amet, consectetur adipiscing elit. Morbi dictum, enim id finibus dignissim, erat tellus imperdiet sapien, eget consectetur felis odio ac metus. Mauris sem lectus, sagittis eget bibendum id, iaculis ac felis. Sed eu est aliquam, ullamcorper dolor maximus, aliquam enim. Pellentesque venenatis in nulla a fringilla. Curabitur elementum bibendum euismod. Orci varius natoque penatibus et magnis dis parturient montes, nascetur ridiculus mus. Curabitur quis ex aliquam, condimentum justo non, laoreet justo. Maecenas mauris ligula, mollis et ante vulputate, efficitur hendrerit orci. Donec et venenatis magna. In tincidunt nisl eu diam condimentum semper. Fusce vulputate orci eu ipsum ornare, vitae molestie quam feugiat. Sed condimentum velit vitae augue consequat vestibulum. Aenean bibendum purus enim, vel dapibus ipsum aliquet non."
        }
        p {
            "Etiam et libero ut eros tincidunt tristique ut vel lorem. Mauris dapibus auctor gravida. Ut ac nulla tempor, fringilla velit non, suscipit purus. Mauris ultrices, ante ut vehicula pulvinar, lorem eros egestas neque, non pellentesque sapien sapien ut quam. Integer posuere leo sit amet sem tempor sodales. Vestibulum dignissim et mauris vel sollicitudin. Ut ac enim eleifend, tempus mi condimentum, vehicula augue. Pellentesque facilisis a nulla a facilisis. Mauris dapibus commodo pharetra."
        }
        p {
            "In in volutpat diam. Nullam quis felis vel ligula pulvinar tempor in ac erat. Proin vel velit at velit cursus auctor. Duis eget posuere nisl, vehicula egestas sapien. Mauris quis mauris ipsum. Fusce cursus purus in mi feugiat faucibus. Vivamus vel commodo justo. Aenean a elit sit amet ante fermentum scelerisque in in nulla. Sed bibendum posuere ante, in blandit nisi. Donec dignissim massa vitae gravida eleifend. In interdum aliquet mauris, at dapibus sem aliquet at. Sed eu lacus vel odio laoreet pretium. Pellentesque vestibulum, diam quis elementum consectetur, nisi tellus suscipit nulla, quis lobortis erat ex id mi. Fusce sed mollis nulla, ac fermentum sem. Aliquam sagittis quam accumsan ligula condimentum, et lobortis lacus mollis."
        }
        p {
            "Proin nec augue eu justo sollicitudin ultricies eget pulvinar felis. Nulla ac porta turpis. Aenean fringilla ex eros, eu suscipit dui lobortis ut. Nunc sodales augue ut orci accumsan lobortis. Etiam lorem neque, viverra id enim non, laoreet pretium diam. Donec sagittis est vel pretium lobortis. Etiam volutpat, magna eu accumsan rutrum, lectus mi pellentesque urna, nec porta diam metus sit amet urna. Phasellus rhoncus eros vel eros auctor, eu sagittis ex lobortis. Pellentesque habitant morbi tristique senectus et netus et malesuada fames ac turpis egestas. Duis ullamcorper, diam vitae volutpat aliquet, purus arcu lacinia sem, maximus tristique tortor turpis suscipit turpis. Proin scelerisque eu arcu quis lobortis."
        }
        p {
            "Nam magna elit, lobortis sed malesuada vel, tempus a tellus. Maecenas ullamcorper posuere lacus et porttitor. Duis congue pulvinar metus, a pretium mi sodales vel. Lorem ipsum dolor sit amet, consectetur adipiscing elit. In quam neque, cursus a ex ut, ultrices imperdiet erat. Ut mollis at enim a tempus. Sed commodo eros ut neque scelerisque laoreet. Etiam sed sodales tellus. Sed commodo, ex ac tempus maximus, purus nunc tempor erat, sit amet dapibus turpis lacus nec augue. Nulla dapibus vel leo eget congue. Quisque rutrum lobortis purus, vel convallis est posuere eu."
        }
    }
}
