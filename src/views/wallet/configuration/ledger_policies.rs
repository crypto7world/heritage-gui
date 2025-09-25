use crate::prelude::*;

use std::{collections::BTreeMap, sync::Arc};

use btc_heritage_wallet::{
    btc_heritage::{AccountXPubId, HeritageWalletBackup},
    ledger::WalletPolicy,
    AnyKeyProvider, BoundFingerprint, LedgerPolicy, Wallet,
};

use crate::{
    components::{
        inputs::BackupRestoreSection,
        modal::ConfigModal,
        svg::{AlertOutline, CheckCircle, Delete, DrawSvg},
    },
    utils::{log_error, CCStr},
};

/// UI representation of a wallet policy for display
#[derive(Debug, Clone, PartialEq)]
struct UIWalletPolicy {
    name: CCStr,
    descriptor_template: CCStr,
    keys: Vec<CCStr>,
}

impl LoadedElement for UIWalletPolicy {
    type Loader = TransparentLoader;

    fn element<M: LoadedComponentInputMapper>(self, _m: M) -> Element {
        rsx! {
            div { class: "text-base flex flex-col gap-4",
                div {
                    h4 { class: "font-semibold text-base mb-2", "Account Name:" }
                    div { class: "font-mono p-2", "{self.name}" }
                }

                div {
                    h4 { class: "font-semibold text-base mb-2", "Wallet Policy:" }
                    div { class: "font-mono p-2 break-all", "{self.descriptor_template}" }
                }

                if !self.keys.is_empty() {
                    div {
                        h4 { class: "font-semibold text-base mb-2", "Keys" }
                        div { class: "flex flex-col gap-2",
                            for (index , key) in self.keys.iter().enumerate() {
                                div { class: "flex items-center gap-2",
                                    div { class: "font-mono font-bold text-sm bg-primary text-primary-content rounded px-2 py-1 flex-shrink-0",
                                        "@{index}"
                                    }
                                    div { class: "font-mono p-2 break-all flex-1", "{key}" }
                                }
                            }
                        }
                    }
                } else {
                    div {
                        h4 { class: "font-semibold text-base mb-2", "Keys" }
                        div { class: "text-sm text-base-content/60 italic",
                            "Key details are not available for this policy type"
                        }
                    }
                }
            }
        }
    }

    fn place_holder() -> Self {
        Self {
            name: CCStr::place_holder(),
            descriptor_template: CCStr::place_holder(),
            keys: vec![CCStr::place_holder()],
        }
    }
}

impl FromRef<WalletPolicy> for UIWalletPolicy {
    fn from_ref(policy: &WalletPolicy) -> Self {
        Self {
            name: CCStr::from(policy.name.clone()),
            descriptor_template: CCStr::from(policy.descriptor_template.clone()),
            keys: policy
                .keys
                .iter()
                .map(|k| CCStr::from(k.to_string()))
                .collect(),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
struct UILedgerPolicyLine {
    policy_id: AccountXPubId,
    policy: CCStr,
}
impl LoadedElement for UILedgerPolicyLine {
    type Loader = TransparentLoader;

    fn element<M: LoadedComponentInputMapper>(self, m: M) -> Element {
        rsx! {
            tr {
                td { class: "text-center align-middle",
                    LoadedComponent { input: m.map(self.policy_id) }
                }
                td { class: "font-mono text-sm break-all",
                    LoadedComponent { input: m.map(self.policy) }
                }
            }
        }
    }

    fn place_holder() -> Self {
        Self {
            policy_id: 0,
            policy: CCStr::place_holder(),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
struct UILedgerPoliciesTable(BTreeMap<AccountXPubId, UILedgerPolicyLine>);
impl LoadedElement for UILedgerPoliciesTable {
    type Loader = TransparentLoader;

    fn element<M: LoadedComponentInputMapper>(self, m: M) -> Element {
        rsx! {
            table { class: "table table-zebra",
                thead {
                    tr {
                        th { class: "w-32", "Account ID" }
                        th { "Policy" }
                    }
                }
                tbody {
                    LoadedComponent { input: m.map(self.0) }
                }
            }
        }
    }

    fn place_holder() -> Self {
        let line = UILedgerPolicyLine::place_holder();
        Self([(line.policy_id, line)].into())
    }
}
impl LoadedSuccessConversionMarker
    for TypeCouple<BTreeMap<AccountXPubId, LedgerPolicy>, UILedgerPoliciesTable>
{
}
impl FromRef<BTreeMap<AccountXPubId, LedgerPolicy>> for UILedgerPoliciesTable {
    fn from_ref(policies: &BTreeMap<AccountXPubId, LedgerPolicy>) -> Self {
        Self(
            policies
                .iter()
                .map(|(id, policy)| {
                    (
                        *id,
                        UILedgerPolicyLine {
                            policy_id: *id,
                            policy: CCStr::from(policy.to_string()),
                        },
                    )
                })
                .collect(),
        )
    }
}

#[component]
pub(super) fn LedgerPoliciesConfig(wallet_name: CCStr) -> Element {
    log::debug!("LedgerPoliciesConfig Rendered");

    let database_service = state_management::use_database_service();
    let service_client_service = state_management::use_service_client_service();
    let blockchain_provider_service = state_management::use_blockchain_provider_service();

    let mut wallet = use_context::<AsyncSignal<Wallet>>();
    let fingerprint = use_memo(move || wallet.lmap(|wallet| wallet.fingerprint().ok()).flatten());

    let uses_ledger = helper_hooks::use_memo_wallet_uses_ledger(wallet);
    let ledger_registered_policies =
        use_context::<Memo<Option<BTreeMap<AccountXPubId, LedgerPolicy>>>>();
    let ledger_unregistered_policies_from_online_wallet =
        use_context::<FMemo<BTreeMap<AccountXPubId, LedgerPolicy>>>();

    // // Manual backup input
    // let manual_backup_data = use_signal(|| String::new());

    // // Parse manual backup data
    // let manual_backup = use_memo(move || {
    //     let backup_str = manual_backup_data.read();
    //     if backup_str.trim().is_empty() {
    //         None
    //     } else {
    //         Some(
    //             serde_json::from_str::<HeritageWalletBackup>(backup_str.as_str())
    //                 .map_err(|e| CCStr::from(format!("Invalid backup data: {}", e))),
    //         )
    //     }
    // });
    let heritage_wallet_backup_state: Signal<Result<HeritageWalletBackup, CCStr>> =
        use_signal(|| Err(CCStr::default()));
    let manual_backup = use_memo(move || {
        let rbkp = heritage_wallet_backup_state();
        rbkp.is_ok().then_some(rbkp)
    });

    // Get unregistered policies from manual backup
    let ledger_unregistered_policies_from_manual =
        helper_hooks::use_memo_ledger_unregistered_policies(
            ledger_registered_policies,
            manual_backup,
        );

    // Combine both sources of unregistered policies
    let ledger_unregistered_policies = use_memo(move || {
        let mut ledger_unregistered_policies = BTreeMap::new();
        if let Some(Ok(btm)) = ledger_unregistered_policies_from_online_wallet
            .read()
            .as_ref()
        {
            ledger_unregistered_policies.extend(btm.iter().map(|(k, v)| (*k, v.clone())))
        }
        if let Some(Ok(btm)) = ledger_unregistered_policies_from_manual.read().as_ref() {
            ledger_unregistered_policies.extend(btm.iter().map(|(k, v)| (*k, v.clone())))
        }
        match (
            ledger_unregistered_policies_from_online_wallet
                .read()
                .as_ref(),
            ledger_unregistered_policies_from_manual.read().as_ref(),
        ) {
            (Some(Ok(_)), _) | (_, Some(Ok(_))) => Some(Ok(ledger_unregistered_policies)),
            (Some(Err(e1)), Some(Err(e2))) => Some(Err(CCStr::from(format!("{e1}|{e2}")))),
            (Some(Err(e1)), None) => Some(Err(e1.clone())),
            (None, Some(Err(e2))) => Some(Err(e2.clone())),
            (None, None) => None,
        }
    });

    let mut in_operation = use_signal(|| false);
    let mut register_modal = use_signal(|| false);
    let mut current_policy = use_signal(|| None::<UIWalletPolicy>);
    let mut current_policy_index = use_signal(|| 0);
    let mut policies_to_register_count = use_signal(|| 0);

    let has_policies_to_register =
        use_memo(move || match ledger_unregistered_policies.read().as_ref() {
            Some(Ok(h)) => !h.is_empty(),
            _ => false,
        });

    let ledger_ready = use_memo(move || state_management::ledger_is_ready().is_some());

    let can_register = use_memo(move || has_policies_to_register() && ledger_ready());

    let can_delete = use_memo(move || {
        ledger_registered_policies
            .lmap(|lrp| !lrp.is_empty())
            .unwrap_or_default()
    });

    let register_policies = {
        let wallet_name = wallet_name.clone();
        move |_| {
            let wallet_name = wallet_name.clone();
            async move {
                *in_operation.write() = true;
                *register_modal.write() = true;

                let policies_ref = ledger_unregistered_policies.read();
                let policies = match policies_ref.as_ref() {
                    Some(Ok(policies)) => policies,
                    _ => {
                        log::error!("No unregistered policies available");
                        alert_error("No unregistered policies available");
                        *in_operation.write() = false;
                        *register_modal.write() = false;
                        return;
                    }
                };

                *current_policy_index.write() = 0usize;
                *policies_to_register_count.write() = policies.len();

                let Ok(mut owned_wallet) = state_management::get_wallet(
                    database_service,
                    service_client_service,
                    blockchain_provider_service,
                    wallet_name,
                )
                .await
                .map_err(log_error) else {
                    alert_error("Failed to load the wallet from the Database");
                    return;
                };
                let ledger_key = match owned_wallet.key_provider_mut() {
                    AnyKeyProvider::Ledger(ledger_key) => ledger_key,
                    _ => {
                        alert_error("Wallet does not use Ledger key provider");
                        return;
                    }
                };
                let result = {
                    ledger_key
                        .register_policies(policies.values(), |wp: &WalletPolicy| {
                            *current_policy_index.write() += 1;
                            *current_policy.write() = Some(UIWalletPolicy::from_ref(wp));
                        })
                        .await
                        .map_err(log_error)
                };

                match result {
                    Ok(_) => {
                        log::info!("Successfully registered Ledger policies");
                        alert_success("Ledger policies registered successfully");
                    }
                    Err(e) => {
                        log::error!("Failed to register Ledger policies: {e}");
                        alert_error(format!("Failed to register Ledger policies: {e}"));
                    }
                }
                // Temporarily puting into an Arc to share ownership with the blocking thread
                let owned_wallet = Arc::new(owned_wallet);
                let Ok(_) = state_management::save_wallet(database_service, owned_wallet.clone())
                    .await
                    .map_err(log_error)
                else {
                    alert_error("Failed to save the wallet into the Database");
                    return;
                };
                // Taking it back from the Arc
                let owned_wallet = Arc::into_inner(owned_wallet).expect("save_wallet is finished");
                // Putting it inside the AsyncSignal
                wallet.write().replace(owned_wallet);

                *current_policy.write() = None;
                *in_operation.write() = false;
                *register_modal.write() = false;
            }
        }
    };

    let delete_registered_policies = {
        let wallet_name = wallet_name.clone();
        move |_| {
            let wallet_name = wallet_name.clone();
            async move {
                *in_operation.write() = true;

                let Ok(mut owned_wallet) = state_management::get_wallet(
                    database_service,
                    service_client_service,
                    blockchain_provider_service,
                    wallet_name,
                )
                .await
                .map_err(log_error) else {
                    alert_error("Failed to load the wallet from the Database");
                    return;
                };
                let ledger_key = match owned_wallet.key_provider_mut() {
                    AnyKeyProvider::Ledger(ledger_key) => ledger_key,
                    _ => {
                        alert_error("Wallet does not use Ledger key provider");
                        return;
                    }
                };
                let policies_cleared_count = ledger_key.clear_registered_policies();

                // Temporarily puting into an Arc to share ownership with the blocking thread
                let owned_wallet = Arc::new(owned_wallet);
                let Ok(_) = state_management::save_wallet(database_service, owned_wallet.clone())
                    .await
                    .map_err(log_error)
                else {
                    alert_error("Failed to save the wallet into the Database");
                    return;
                };
                // Taking it back from the Arc
                let owned_wallet = Arc::into_inner(owned_wallet).expect("save_wallet is finished");
                // Putting it inside the AsyncSignal
                wallet.write().replace(owned_wallet);

                log::info!("Successfully cleared {policies_cleared_count} Ledger policies");
                alert_success("Successfully cleared {policies_cleared_count} Ledger policies");

                *current_policy.write() = None;
                *in_operation.write() = false;
            }
        }
    };

    use_drop(|| log::debug!("LedgerPoliciesConfig Dropped"));

    // Only show if wallet uses Ledger
    if !uses_ledger() {
        return rsx! {};
    }

    rsx! {
        div { class: "rounded-box border border-base-content/5 shadow-md p-4 my-4",
            h2 { class: "text-2xl font-bold mb-4", "Ledger Policies" }

            div { class: "text-sm font-light mb-6",
                "Ledger Policies define the Bitcoin spending conditions that your Ledger device recognizes and can sign for. "
                "Each Heritage Configuration requires its corresponding policy to be registered on your Ledger device before it can be used for signing transactions."
            }

            div { class: "collapse collapse-arrow bg-base-200 mb-4",
                input { r#type: "checkbox", class: "collapse-input" }
                div { class: "collapse-title text-lg font-medium", "Registered Heritage Configurations" }
                div { class: "collapse-content",
                    div { class: "text-sm text-base-content/70 mb-3",
                        "The following Heritage Configurations are already registered on your Ledger device as policies:"
                    }
                    LoadedComponent::<UILedgerPoliciesTable> { input: ledger_registered_policies.into() }
                    button {
                        class: "btn btn-primary btn-xs",
                        disabled: !can_delete() || in_operation(),
                        onclick: delete_registered_policies,
                        if in_operation() {
                            span { class: "loading loading-spinner loading-sm mr-2" }
                            "Registering..."
                        } else {
                            DrawSvg::<Delete> {}
                            "Delete Registered Policies"
                        }
                    }
                }
            }
            div { class: "collapse collapse-arrow bg-base-200 mb-4",
                input { r#type: "checkbox", class: "collapse-input" }
                div { class: "collapse-title text-lg font-medium", "Manual Backup Input" }
                div { class: "collapse-content",
                    div { class: "text-sm text-base-content/70 mb-3",
                        "Manually provide an Online Wallet Backup to register policies from a different device:"
                    }
                    BackupRestoreSection {
                        heritage_wallet_backup_state,
                        expected_fingerprint: fingerprint(),
                    }
                }
            }

            if has_policies_to_register() {
                div { class: "bg-base-100 rounded-lg p-4 mb-4 border border-warning",
                    h3 { class: "text-lg font-semibold mb-3", "Unregistered Heritage Configurations" }
                    div { class: "text-sm text-base-content/70 mb-3",
                        "The following Heritage Configurations need to be registered on your Ledger device as policies:"
                    }
                    LoadedComponent::<UILedgerPoliciesTable> { input: ledger_unregistered_policies.into() }
                }
            }


            div { class: "flex items-center gap-4",
                MaybeHighlight {
                    step: OnboardingStep::ClickRegisterLedgerPolicies,
                    context_filter: consume_onboarding_context(),
                    button {
                        class: "btn btn-primary",
                        disabled: !can_register() || in_operation(),
                        onclick: register_policies,
                        if in_operation() {
                            span { class: "loading loading-spinner loading-sm mr-2" }
                            "Registering..."
                        } else {
                            DrawSvg::<CheckCircle> {}
                            "Register Ledger Policies"
                        }
                    }
                }
            }

            // Registration progress modal
            ConfigModal { is_open: register_modal, title: "Registering Ledger Policies",
                div { class: "w-full flex flex-col gap-4",
                    div { class: "text-center text-xl",
                        span { class: "loading loading-spinner loading-lg text-primary mx-8" }
                        "Registering Ledger Policy "
                        span { class: "font-black text-2xl",
                            "{current_policy_index()}/{policies_to_register_count()}"
                        }
                    }
                    div { class: "text-center text-base italic",
                        "Note that Ledger call it "
                        span { class: "font-bold text-accent", "Account" }
                    }

                    div { class: "alert alert-warning",
                        DrawSvg::<AlertOutline> {}
                        div {
                            div { class: "font-bold", "Verify on Ledger" }
                            div { class: "text-sm",
                                "Make sure the policy details displayed on your Ledger device exactly match the information shown below."
                            }
                        }
                    }

                    div { class: "bg-base-200 rounded-lg p-4",
                        LoadedComponent::<UIWalletPolicy> { input: current_policy().into() }
                    }
                }
            }
        }
    }
}
