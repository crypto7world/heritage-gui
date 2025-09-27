use crate::prelude::*;

use crate::{
    components::svg::{ContentCopy, DrawSvg, SvgSize::Size5},
    utils::CCStr,
};

#[component]
pub fn CopyToClipboardButtonIcon(value: CCStr, disabled: Option<bool>) -> Element {
    let clipboard_service = state_management::use_clipboard_service();
    rsx! {
        button {
            class: "btn btn-circle btn-xs",
            onclick: move |_| {
                state_management::copy_to_clipboard(clipboard_service, value.as_ref());
            },
            disabled,
            DrawSvg::<ContentCopy> { size: Size5 }
        }
    }
}
#[component]
pub fn CopyTextarea(
    value: CCStr,
    rows: Option<usize>,
    text_size: Option<&'static str>,
    copy_btn_disabled: Option<bool>,
) -> Element {
    let text_size = text_size.unwrap_or("text-xs");
    rsx! {
        div { class: "relative",
            textarea {
                class: "textarea textarea-bordered font-mono pr-8 {text_size} w-full",
                readonly: true,
                rows: rows.unwrap_or(10),
                value: "{value}",
            }
            div { class: "absolute top-2 right-2",
                CopyToClipboardButtonIcon { value: value.clone(), disabled: copy_btn_disabled }
            }
        }
    }
}
