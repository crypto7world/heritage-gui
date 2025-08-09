use crate::prelude::*;

use btc_heritage_wallet::{btc_heritage::heritage_wallet::WalletAddress, OnlineWallet, Wallet};

use crate::{
    components::{misc::UIBtcAddr, modal::InfoModal, qrcode::UIQRCode},
    utils::CheapClone,
};

#[component]
pub fn ReceiveButton() -> Element {
    log::debug!("ReceiveButton Rendered");

    // Context resources
    let mut wallet = use_context::<AsyncSignal<Wallet>>();
    let ready_to_use_address = use_context::<Memo<Option<Option<CheapClone<WalletAddress>>>>>();

    let mut in_operation = use_signal(|| false);
    let mut display_modal = use_signal(|| false);
    let mut receive_address = use_signal(|| None);

    let receive_click = move |_| async move {
        *receive_address.write() = None;
        *display_modal.write() = true;

        // If the ready_to_use_address is not ready, just quit
        // Should never happen
        let Some(ready_to_use_address) = ready_to_use_address.cloned() else {
            return;
        };

        if let Some(ready_to_use_address) = ready_to_use_address {
            log::info!("receive_click - Using existing unused address");
            log::debug!("ready_to_use_address={ready_to_use_address}");
            *receive_address.write() = Some(ready_to_use_address);
        } else {
            *in_operation.write() = true;
            let op_result = wallet
                .with_mut(async |wallet: &mut Wallet| wallet.get_address().await)
                .await;
            *in_operation.write() = false;

            match op_result {
                Ok(new_address) => {
                    log::info!("receive_click - Successfully created a new address for the wallet");
                    log::debug!("new_address={new_address}");
                    *receive_address.write() = Some(CheapClone::new(new_address));
                }
                Err(e) => {
                    log::error!("Failed to generate a new address: {e}");
                    alert_error(format!("Failed to generate a new address: {e}"));
                }
            };
            *in_operation.write() = false;
        }
    };

    let address_string =
        use_memo(move || receive_address.lmap(|wa| UIBtcAddr::from(wa.address().to_string())));

    let address_qrcode =
        use_memo(move || receive_address.lmap(|wa| UIQRCode::from(wa.address().to_qr_uri())));

    use_drop(|| log::debug!("ReceiveButton Dropped"));

    rsx! {
        MaybeHighlight {
            step: OnboardingStep::ClickWalletReceive,
            context_filter: consume_onboarding_context(),
            button {
                class: "btn btn-secondary size-64 rounded-4xl uppercase text-3xl font-black",
                onclick: receive_click,
                disabled: in_operation() || ready_to_use_address.read().is_none(),
                "Receive"
            }
        }
        InfoModal { is_open: display_modal, title: "Receive address",
            div { class: "flex flex-col gap-2 items-center",
                LoadedComponent { input: address_qrcode.cloned().into() }
                div { class: "text-xl font-mono",
                    LoadedComponent { input: address_string.cloned().into() }
                }
            }
        }
    }
}
