use dioxus::prelude::*;

#[component]
pub fn Modal(
    is_open: Signal<bool>,
    #[props(default = false)] persistent: bool,
    children: Element,
) -> Element {
    let mut scroll_blocking = use_context::<Signal<crate::gui::ScrollBlocking>>();
    let mut classes = use_signal(|| {
        "fixed top-0 left-0 h-screen w-screen bg-base-content/25 z-50 cursor-default".to_string()
    });

    if *is_open.read() {
        scroll_blocking.write().block();
    } else {
        scroll_blocking.write().unblock();
    }

    rsx! {
        if *is_open.read() {
            div {
                class: classes,
                onclick: move |event| {
                    event.stop_propagation();
                    if persistent {
                        spawn(async move {
                            let orig_len = classes.read().len();
                            *classes.write() += " animate-scalebump";
                            tokio::time::sleep(tokio::time::Duration::from_millis(200)).await;
                            classes.write().truncate(orig_len);
                        });
                    } else {
                        *is_open.write() = false;
                    }
                },
                div { class: "absolute top-1/2 left-1/2 -translate-x-1/2 -translate-y-1/2 size-fit max-h-[90vh] max-w-[90vw] bg-base-100 z-50 rounded-lg overflow-hidden",
                    { children }
                }
            }
        }
    }
}
