use dioxus::prelude::*;

use crate::views::TitledView;

#[component]
pub fn WalletView(wallet_name: String) -> Element {
    let mut s1 = use_signal(|| 0);
    let mut s2 = use_signal(|| 0);
    use_effect(move || {
        // Read to subscribre but we actually don't care about
        // the value, we just want to react to a change
        s1.read();
        s2 += 1;
    });
    let s3 = use_memo(move || s1 + 1);

    let mut s4 = use_signal(|| 0);
    use_effect(move || {
        s4.set(s1 + 1);
    });
    log::debug!("Rendered");

    rsx! {
        TitledView { title: "{wallet_name}", subtitle: "A wallet",
            div { "Balance send/receive, etc..." }
            div { "Signal2: {s2}" }
            button {
                class: "btn btn-primary",
                onclick: move |_| {
                    s1 += 1;
                },
                "Plus"
            }
            div { "Signal1: {s1}" }
            div { "Signal3: {s3}" }
            div { "Signal4: {s4}" }
        }
    }
}
