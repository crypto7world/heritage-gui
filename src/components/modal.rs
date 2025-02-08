use dioxus::prelude::*;

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
            class: "modal",
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
            div { class: "modal-box max-w-max {classes}", {children} }
        }
    }
}
