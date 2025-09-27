use dioxus::prelude::*;

use futures_util::stream::StreamExt;

use crate::prelude::{alert_error, alert_info, alert_warn};

#[derive(Debug)]
pub enum ClipboardCommand {
    Set(String),
}

pub(super) fn use_clipboard_service() -> Coroutine<ClipboardCommand> {
    use_coroutine(
        move |mut rx: UnboundedReceiver<ClipboardCommand>| async move {
            log::info!("clipboard_service (coroutine) - start");

            let mut clipboard = match arboard::Clipboard::new() {
                Ok(c) => Some(c),
                Err(e) => {
                    log::warn!("Clipboard service failed to init: {e}");
                    alert_warn("Failed to initialize Clipboard!");
                    None
                }
            };

            while let Some(cmd) = rx.next().await {
                log::debug!("clipboard_service (coroutine) - Processing commmand {cmd:?}...");
                match cmd {
                    ClipboardCommand::Set(str) => match clipboard.as_mut() {
                        Some(clipboard) => match clipboard.set_text(str) {
                            Ok(_) => alert_info("Copied to Clipboard!"),
                            Err(_) => alert_error("Failed to copy to Clipboard!"),
                        },
                        None => {
                            alert_error("No Clipboard service!");
                        }
                    },
                }
                log::debug!("clipboard_service (coroutine) - Command processed");
            }
        },
    )
}
