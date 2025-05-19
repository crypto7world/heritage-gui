use dioxus::prelude::*;

use crate::utils::ArcStr;

#[component]
pub fn Tooltip(tooltip_text: ArcStr, children: Element) -> Element {
    rsx! {
        div {
            class: "tooltip before:text-xs before:text-white hover:after:delay-300 hover:before:delay-300",
            "data-tip": "{tooltip_text}",
            {children}
        }
    }
}

#[component]
pub fn Modal(
    is_open: Signal<bool>,
    #[props(default = false)] persistent: bool,
    children: Element,
) -> Element {
    let mut classes = use_signal(|| String::new());

    rsx! {
        input {
            r#type: "checkbox",
            name: "modal-toggle",
            class: "modal-toggle",
            tabindex: "-1",
            checked: is_open(),
        }
        div {
            class: "modal z-40",
            role: "dialog",
            onclick: move |event| {
                event.stop_propagation();
                if persistent {
                    spawn(async move {
                        let orig_len = classes.read().len();
                        *classes.write() += "animate-scalebump";
                        tokio::time::sleep(tokio::time::Duration::from_millis(200)).await;
                        classes.write().truncate(orig_len);
                    });
                } else {
                    *is_open.write() = false;
                }
            },
            div { class: "modal-box max-w-max p-0 {classes}",
                div { class: "p-6 w-fit", {children} }
            }
        }
    }
}

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
