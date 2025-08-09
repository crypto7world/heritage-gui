use crate::prelude::*;

use super::super::sync::WalletSync;

use btc_heritage_wallet::{
    btc_heritage::{errors::ParseBlockInclusionObjectiveError, BlockInclusionObjective},
    online_wallet::{OnlineWallet, WalletStatus},
    Wallet,
};

use crate::utils::feerate_sat_per_vb;

#[component]
pub(super) fn BlockInclusionObjectiveConfig() -> Element {
    let mut wallet = use_context::<AsyncSignal<Wallet>>();
    let wallet_status = use_context::<FResource<WalletStatus>>();

    let mut in_operation = use_signal(|| false);
    let mut bio = use_signal(|| BlockInclusionObjective::default());
    let current_bio = use_memo(move || {
        if let Some(Ok(ref wallet_status)) = *wallet_status.read() {
            wallet_status.block_inclusion_objective
        } else {
            BlockInclusionObjective::default()
        }
    });
    use_effect(move || {
        *bio.write() = current_bio();
    });

    let current_feerate = use_memo(move || match &*wallet_status.read() {
        Some(Ok(ref wallet_status)) if wallet_status.last_fee_rate.is_some() => {
            format!(
                "{} sat/vB",
                feerate_sat_per_vb(wallet_status.last_fee_rate.unwrap())
            )
        }
        _ => "- sat/vB".to_owned(),
    });

    // Function to handle form submission
    let update_bio = move |_| async move {
        // Set creating state to true to show loading UI
        *in_operation.write() = true;

        match wallet
            .with_mut(async |wallet| wallet.set_block_inclusion_objective(bio()).await)
            .await
        {
            Ok(_) => {
                log::info!("Successfully updated the Block Inclusion Objective");
                alert_success("Block Inclusion Objective updated, please synchronize your wallet to update the fee rate.");
            }
            Err(e) => {
                log::error!("Failed to update the Block Inclusion Objective: {e}");
                alert_error(format!(
                    "Failed to update the Block Inclusion Objective: {e}"
                ));
            }
        };
        *in_operation.write() = false;
    };

    rsx! {
        div { class: "rounded-box border border-base-content/5 shadow-md p-4 my-4",
            h2 { class: "text-2xl font-bold mb-4", "Block Inclusion Objective" }

            div { class: "text-sm font-light mb-4",
                "The Block Inclusion Objective determines the target number of blocks for transaction confirmation. \
                This value is used to estimate the appropriate fee rate when creating transactions. \
                Lower values result in higher fees but faster confirmation times."
            }

            div { class: "flex flex-row gap-8 items-center mb-4",
                div { class: "flex flex-col",
                    div { class: "text-base font-semibold", "Current Fee Rate" }
                    div { class: "text-sm font-light", "{current_feerate()}" }
                }
                WalletSync {}
            }

            div { class: "flex flex-row gap-4 items-end",
                div { role: "fieldset", class: "fieldset w-48",
                    legend { class: "fieldset-legend", "Target blocks" }
                    input {
                        class: "input input-bordered w-full",
                        r#type: "number",
                        min: "{BlockInclusionObjective::MIN}",
                        max: "{BlockInclusionObjective::MAX}",
                        value: "{bio()}",
                        oninput: move |evt| {
                            let new_bio = match evt.parsed() {
                                Ok(new_bio) => new_bio,
                                Err(ParseBlockInclusionObjectiveError::InvalidInt) => {
                                    BlockInclusionObjective::default()
                                }
                                Err(ParseBlockInclusionObjectiveError::ValueTooLow) => {
                                    BlockInclusionObjective::MIN
                                }
                                Err(ParseBlockInclusionObjectiveError::ValueTooHigh) => {
                                    BlockInclusionObjective::MAX
                                }
                            };
                            bio.set(new_bio);
                        },
                    }
                }
                button {
                    class: "btn btn-primary",
                    disabled: bio() == current_bio() || in_operation(),
                    onclick: update_bio,
                    "Update"
                }
            }
        }
    }
}
