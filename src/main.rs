mod clients;
mod components;
mod gui;
mod utils;
mod views;
fn main() {
    env_logger::Builder::from_env(
        env_logger::Env::default().default_filter_or("debug,tracing::span=warn"),
    )
    .format_timestamp_micros()
    .init();
    gui::launch_gui()
}
