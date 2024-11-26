use std::error::Error;

pub fn log_error<E: Error>(error: E) -> String {
    log::error!("{error}");
    error.to_string()
}
