mod addresses_history;
pub mod configuration;
mod heritage_configurations_history;
mod receive;
pub mod spend;
mod sync;
mod transactions_history;

use crate::prelude::*;

use btc_heritage_wallet::{online_wallet::WalletStatus, Wallet};
use receive::ReceiveButton;
use sync::WalletSync;

use crate::{
    components::{
        badge::{ExternalDependencyStatus, KeyProviderType, OnlineWalletType, UIBadge},
        balance::UIWalletBalance,
        quick_actions::{
            BackupOnlineWallet, ShowKeyProviderMnemonic, ShowKeyProviderMnemonicFlavor,
            UnlockLocalKey,
        },
        svg::{Cog, DrawSvg, SvgSize::Full},
    },
    utils::CCStr,
    Route,
};

#[component]
pub fn WalletWrapperLayout(wallet_name: CCStr) -> Element {
    log::debug!("WalletWrapperLayout Rendered");

    let wallet = helper_hooks::use_async_wallet(wallet_name.clone());

    let wallet_status = helper_hooks::use_resource_wallet_status(wallet);

    let wallet_transactions = helper_hooks::use_resource_wallet_transactions(wallet);
    let tx_stats_by_address = helper_hooks::use_memo_tx_stats_by_address(wallet_transactions);

    let wallet_utxos = helper_hooks::use_resource_wallet_utxos(wallet);
    let balance_by_heritage_config =
        helper_hooks::use_memo_balance_by_heritage_config(wallet_utxos);
    let utxo_stats_by_address = helper_hooks::use_memo_utxo_stats_by_address(wallet_utxos);

    let wallet_subwallet_configs = helper_hooks::use_resource_wallet_subwallet_configs(wallet);
    let heritage_configs_with_info = helper_hooks::use_memo_heritage_configs_with_info(
        wallet_subwallet_configs,
        balance_by_heritage_config,
    );
    let heritage_configs_with_info_indexed_by_origin_info =
        helper_hooks::use_memo_heritage_configs_with_info_indexed_by_origin_info(
            heritage_configs_with_info,
        );
    let heritage_configs_with_info_indexed_by_heritage_config =
        helper_hooks::use_memo_heritage_configs_with_info_indexed_by_heritage_config(
            heritage_configs_with_info,
        );

    let backup = helper_hooks::use_resource_wallet_descriptor_backup(wallet);
    let memo_backup = helper_hooks::use_memo_resource(backup);
    let ledger_registered_policies = helper_hooks::use_memo_ledger_registered_policies(wallet);
    let ledger_unregistered_policies = helper_hooks::use_memo_ledger_unregistered_policies(
        ledger_registered_policies,
        memo_backup,
    );
    let keyprovider_status = helper_hooks::use_memo_wallet_keyprovider_status(
        wallet,
        Some(ledger_unregistered_policies),
    );

    let online_status = helper_hooks::use_memo_wallet_online_status(wallet);

    let utxo_with_info = helper_hooks::use_memo_utxo_with_info(
        wallet_utxos,
        heritage_configs_with_info_indexed_by_heritage_config,
    );

    let wallet_addresses = helper_hooks::use_resource_wallet_addresses(wallet);
    let addresses_with_info = helper_hooks::use_memo_addresses_with_info(
        wallet_addresses,
        heritage_configs_with_info_indexed_by_origin_info,
        tx_stats_by_address,
        utxo_stats_by_address,
    );
    let addresses_set = helper_hooks::use_memo_addresses_set(wallet_addresses);
    let ready_to_use_address = helper_hooks::use_memo_ready_to_use_address(addresses_with_info);

    let database_heirs = helper_hooks::use_resource_database_heirs();
    let service_heirs = helper_hooks::use_resource_service_heirs();
    let heirs = helper_hooks::use_memo_heirs(database_heirs, service_heirs);

    // Provide the wallet resources to all child that may want it
    use_context_provider(|| wallet);
    use_context_provider(|| wallet_status);
    use_context_provider(|| wallet_transactions);

    use_context_provider(|| heritage_configs_with_info);

    use_context_provider(|| backup);
    use_context_provider(|| ledger_registered_policies);
    use_context_provider(|| ledger_unregistered_policies);

    use_context_provider(|| keyprovider_status);
    use_context_provider(|| online_status);

    use_context_provider(|| addresses_with_info);
    use_context_provider(|| addresses_set);
    use_context_provider(|| ready_to_use_address);

    use_context_provider(|| utxo_with_info);

    use_context_provider(|| service_heirs);
    use_context_provider(|| heirs);

    use_context_provider(|| OnboardingContextItemId::WalletName.item(wallet_name.to_string()));

    use_drop(|| log::debug!("WalletWrapperLayout Dropped"));
    rsx! {
        Outlet::<crate::Route> {}
    }
}

#[component]
pub fn WalletView(wallet_name: CCStr) -> Element {
    log::debug!("WalletView Rendered");

    let wallet = use_context::<AsyncSignal<Wallet>>();
    let wallet_status = use_context::<FResource<WalletStatus>>();

    let fingerprint = helper_hooks::use_memo_fingerprint(wallet);
    let keyprovider_status =
        use_context::<Memo<Option<(KeyProviderType, ExternalDependencyStatus)>>>();
    let can_show_mnemo = use_memo(move || match keyprovider_status() {
        Some((KeyProviderType::LocalKey, _)) => true,
        _ => false,
    });
    let local_key_need_password = use_memo(move || match keyprovider_status() {
        Some((KeyProviderType::LocalKey, ExternalDependencyStatus::NeedUserAction)) => true,
        _ => false,
    });

    let online_status = use_context::<Memo<Option<(OnlineWalletType, ExternalDependencyStatus)>>>();
    let can_backup = use_memo(move || match online_status() {
        Some((OnlineWalletType::Local, _)) => true,
        Some((OnlineWalletType::Service, ExternalDependencyStatus::Available)) => true,
        _ => false,
    });
    let not_sign_only =
        use_memo(move || !matches!(online_status(), Some((OnlineWalletType::None, _))));

    let wn = wallet_name.clone();
    let click_config = move |_| {
        navigator().push(Route::WalletConfigurationView {
            wallet_name: wn.clone(),
        });
    };
    let wn = wallet_name.clone();
    let click_send = move |_| {
        navigator().push(Route::WalletSpendView {
            wallet_name: wn.clone(),
        });
    };

    use_drop(|| log::debug!("WalletView Dropped"));

    rsx! {
        super::TitledView {
            title: wallet_name.clone(),
            subtitle: fingerprint.cloned(),
            left: rsx! {
                if not_sign_only() {
                    WalletSync {}
                }
            },
            right: rsx! {
                div { class: "h-full content-center",
                    MaybeHighlight {
                        step: OnboardingStep::OpenWalletConfiguration,
                        context_filter: consume_onboarding_context(),
                        button {
                            class: "btn btn-circle btn-outline btn-primary btn-lg p-2",
                            onclick: click_config,
                            DrawSvg::<Cog> { size: Full }
                        }
                    }
                }
            },

            div { class: "flex flex-row justify-center gap-8 m-4",
                div { class: "flex flex-col gap-1",
                    div { class: "flex flex-row flex-wrap justify-center gap-1",
                        span { "Key Provider: " }
                        LoadedComponent::<UIBadge> { input: keyprovider_status.into() }
                    }

                    if local_key_need_password() {
                        UnlockLocalKey::<Wallet> {}
                    }
                    if can_show_mnemo() {
                        ShowKeyProviderMnemonic::<Wallet> { flavor: ShowKeyProviderMnemonicFlavor::Wallet }
                    }
                
                }
                div { class: "flex flex-col gap-1",
                    div { class: "flex flex-row flex-wrap justify-center gap-1",
                        span { "Online Wallet: " }
                        LoadedComponent::<UIBadge> { input: online_status.into() }
                    }
                    if can_backup() {
                        BackupOnlineWallet { wallet_name: wallet_name.clone() }
                    }
                }
            }
            div { class: "flex flex-row flex-wrap justify-center items-center gap-4 m-4",
                if not_sign_only() {
                    LoadedComponent::<UIWalletBalance> { input: wallet_status.into() }
                }
                div { class: "inline-flex flex-row flex-wrap justify-center gap-4 m-4",
                    button {
                        class: "btn btn-secondary size-64 rounded-4xl uppercase text-3xl font-black",
                        onclick: click_send,
                        "Send"
                    }
                    if not_sign_only() {
                        ReceiveButton {}
                    }
                }
            }
            if not_sign_only() {
                transactions_history::TransactionsHistory {}
                heritage_configurations_history::HeritageConfigurationsHistory {}
                addresses_history::AddressesHistory {}
            }

            OnboardingInfoModal { step: OnboardingStep::ModalFinishCreatingFirstWallet,
                div { class: "flex flex-col gap-4 max-w-xl text-base",
                    p { class: "text-lg font-semibold text-success", "🎉 Congratulations!" }
                    p {
                        "Your Heritage Wallet has been successfully created and is now ready to use.
                        You can send bitcoins to the address your just generated from any
                        other Bitcoin wallet or Exchange service."
                    }
                    p { class: "text-sm opacity-75",
                        "Welcome to Heritage Wallet - your Bitcoin inheritance solution."
                    }
                }
            }
        }
    }
}
