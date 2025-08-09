use crate::prelude::*;

use std::collections::HashSet;

use btc_heritage_wallet::{bitcoin::Address, Wallet};

use crate::{
    components::{
        badge::{ExternalDependencyStatus, KeyProviderType, OnlineWalletType},
        misc::BackButton,
        spend::{SpendTabs, SpendTabsType},
    },
    utils::CCStr,
    Route,
};

#[component]
pub fn WalletSpendView(wallet_name: CCStr) -> Element {
    log::debug!("WalletSpendView Rendered");

    let wallet = use_context::<AsyncSignal<Wallet>>();
    let fingerprint = helper_hooks::use_memo_fingerprint(wallet);
    let addresses_set = use_context::<FMemo<HashSet<Address>>>();
    let keyprovider_status =
        use_context::<Memo<Option<(KeyProviderType, ExternalDependencyStatus)>>>();
    let online_status = use_context::<Memo<Option<(OnlineWalletType, ExternalDependencyStatus)>>>();

    let cannot_create_reason = use_memo(move || match online_status() {
        Some((OnlineWalletType::None, _)) => {
            Some("Your wallet does not have an Online Wallet component.")
        }
        Some((OnlineWalletType::Service, ExternalDependencyStatus::Unavailable)) => {
            Some("The Heritage Service cannot currently serve your wallet.")
        }

        None => Some("Wallet is not loaded."),
        _ => None,
    });

    let cannot_sign_reason = use_memo(move || match keyprovider_status() {
        Some((KeyProviderType::None, _)) => {
            Some(("Your wallet does not have a Key Provider component.", false))
        }
        Some((KeyProviderType::LocalKey, ExternalDependencyStatus::NeedUserAction)) => Some((
            "Your wallet uses a Local Key that require a password, \
                but the password was not provided.",
            true,
        )),
        Some((KeyProviderType::Ledger, ExternalDependencyStatus::Unavailable)) => Some((
            "Your wallet uses a Ledger Hardware Wallet device, \
               but none can currently serve your wallet.",
            false,
        )),
        Some((KeyProviderType::Ledger, ExternalDependencyStatus::NeedUserAction)) => Some((
            "Your wallet uses a Ledger Hardware Wallet device, \
               but it is missing Ledger Policies to be able to sign transactions.",
            false,
        )),
        None => Some(("Wallet is not loaded.", false)),
        _ => None,
    });

    let cannot_broadcast_reason = use_memo(move || match online_status() {
        Some((OnlineWalletType::None, _)) => {
            Some("Your wallet does not have an Online Wallet component.")
        }
        Some((OnlineWalletType::Service, ExternalDependencyStatus::Unavailable)) => {
            Some("The Heritage Service cannot currently serve your wallet.")
        }
        Some((OnlineWalletType::Local, ExternalDependencyStatus::Unavailable)) => {
            Some("Your wallet use a Blockchain provider but none is accessible.")
        }
        None => Some("Wallet is not loaded."),
        _ => None,
    });

    use_drop(|| log::debug!("WalletSpendView Dropped"));

    rsx! {
        super::super::TitledView {
            title: wallet_name.clone(),
            subtitle: fingerprint.cloned(),
            left: rsx! {
                BackButton {
                    route: Route::WalletView {
                        wallet_name: wallet_name,
                    },
                }
            },
            SpendTabs::<Wallet> {
                spendtabs_type: SpendTabsType::Owner,
                cannot_create_reason,
                cannot_sign_reason,
                cannot_broadcast_reason,
                addresses_set,
            }
        }
    }
}
