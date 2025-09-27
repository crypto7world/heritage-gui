use super::*;

pub fn copy_to_clipboard(clipboard_service: Coroutine<ClipboardCommand>, s: impl Into<String>) {
    log::debug!("copy_to_clipboard - start");
    clipboard_service.send(ClipboardCommand::Set(s.into()));
    log::debug!("copy_to_clipboard - finished");
}
