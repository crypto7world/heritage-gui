use crate::prelude::*;

mod delete;

use btc_heritage_wallet::HeirWallet;

use crate::{
    components::{inputs::RenameDatabaseItem, misc::BackButton},
    utils::CCStr,
    Route,
};

#[component]
pub fn HeirWalletConfigurationView(heirwallet_name: CCStr) -> Element {
    log::debug!("HeirWalletConfigurationView Rendered");

    let heirwallet = use_context::<AsyncSignal<HeirWallet>>();
    let fingerprint = helper_hooks::use_memo_heirwallet_fingerprint(heirwallet);

    use_drop(|| log::debug!("HeirWalletConfigurationView Dropped"));

    rsx! {
        super::super::TitledView {
            title: heirwallet_name.clone(),
            subtitle: fingerprint.cloned(),
            left: rsx! {
                BackButton {
                    route: Route::HeirWalletView {
                        heirwallet_name: heirwallet_name.clone(),
                    },
                }
            },
            RenameDatabaseItem::<HeirWallet> {}
            delete::DeleteHeirWalletConfig {}
        }
    }
}
