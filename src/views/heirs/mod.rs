use crate::prelude::*;

pub mod configuration;
pub mod heir;
pub mod heir_create;
pub mod heir_list;

#[component]
pub fn HeirsWrapperLayout() -> Element {
    log::debug!("HeirsWrapperLayout Rendered");

    let database_heirs = helper_hooks::use_resource_database_heirs();
    let service_heirs = helper_hooks::use_resource_service_heirs();
    let composite_heirs = helper_hooks::use_memo_heirs(database_heirs, service_heirs);

    // Provide the heir resources to all child that may want it
    use_context_provider(|| database_heirs);
    use_context_provider(|| service_heirs);
    use_context_provider(|| composite_heirs);

    use_drop(|| log::debug!("HeirsWrapperLayout Dropped"));
    rsx! {
        Outlet::<crate::Route> {}
    }
}
