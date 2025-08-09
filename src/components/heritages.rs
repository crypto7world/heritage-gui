use crate::prelude::*;

use std::collections::HashMap;

use btc_heritage_wallet::{bitcoin::Amount, btc_heritage::utils::timestamp_now};

use crate::{
    components::{
        balance::UIBtcAmount,
        svg::{DrawSvg, Spend},
        timestamp::UITimestamp,
    },
    utils::{heir_config_type_to_string, CCStr},
    Route,
};

#[derive(Debug, Clone, PartialEq)]
pub struct UIHeritage {
    spend_infos: Option<(CCStr, CCStr)>,
    heir_config_type: &'static str,
    heir_config_fingerprint: CCStr,
    from_service: bool,
    owner_email: Option<CCStr>,
    total_amount: Option<UIBtcAmount>,
    spendable_amount: Option<UIBtcAmount>,
    spend_disabled: bool,
    heritage_lines: Vec<UIHeritageLine>,
}
impl LoadedElement for UIHeritage {
    type Loader = TransparentLoader;
    #[inline(always)]
    fn element<M: LoadedComponentInputMapper>(self, m: M) -> Element {
        rsx! {
            div { class: "card card-lg border shadow-xl w-full",
                div { class: "card-body",


                    div { class: "grid grid-cols-2 lg:grid-cols-4 flex-row gap-2",
                        div { class: "col-span-2 grid grid-cols-subgrid",
                            div { class: "flex flex-col",
                                div { class: "font-light", "Type" }
                                div { class: "text-lg font-bold text-nowrap",
                                    LoadedComponent { input: m.map(self.heir_config_type) }
                                }
                            }
                            div { class: "flex flex-col",
                                div { class: "font-light text-nowrap", "Key Fingerprint" }
                                div { class: "text-lg font-bold",
                                    LoadedComponent { input: m.map(self.heir_config_fingerprint) }
                                }
                            }
                        }
                        div { class: "col-span-2 grid grid-cols-subgrid",
                            div { class: "flex flex-col",
                                div { class: "font-light", "Total" }
                                div { class: "text-lg font-bold text-nowrap",
                                    if let Some(amount) = self.total_amount {
                                        LoadedComponent { input: m.map(amount) }
                                    } else {
                                        "-"
                                    }
                                }
                            }
                            div { class: "flex flex-col",
                                div { class: "font-light text-nowrap", "Spendable" }
                                div { class: "text-lg font-bold",
                                    if let Some(amount) = self.spendable_amount {
                                        LoadedComponent { input: m.map(amount) }
                                    } else {
                                        "-"
                                    }
                                }
                            }
                        }
                    }
                    if self.from_service {
                        div { class: "flex flex-col",
                            div { class: "font-light text-nowrap", "Owner Email" }
                            div { class: "text-lg font-bold",
                                if let Some(owner_email) = self.owner_email {
                                    LoadedComponent { input: m.map(owner_email) }
                                } else {
                                    "-"
                                }
                            }
                        }
                    }


                    table { class: "table w-full",
                        thead {
                            tr {
                                th { "Maturity" }
                                th { "Amount" }
                                th { "Position" }
                                th { "Expiration" }
                            }
                        }
                        tbody {
                            LoadedComponent { input: m.map(self.heritage_lines) }
                        }
                    }

                    div { class: "grow" }

                    if let Some((heirwallet_name, heritage_id)) = self.spend_infos {
                        div { class: "card-actions justify-center mt-6",
                            MaybeHighlight {
                                step: OnboardingStep::ClickInheritanceSpendButton,
                                context_filter: consume_onboarding_context(),
                                button {
                                    class: "btn btn-primary",
                                    disabled: self.spend_disabled,
                                    onclick: move |_| {
                                        let heirwallet_name = heirwallet_name.clone();
                                        let heritage_id = heritage_id.clone();
                                        if let OnboardingStatus::InProgress(ref mut onboarding) = *state_management::ONBOARDING_STATUS
                                            .write()
                                        {
                                            if onboarding
                                                .context()
                                                .get_first_context(OnboardingContextItemId::HeirWalletName)
                                                .is_some_and(|val| val == heirwallet_name.as_ref())
                                            {
                                                onboarding
                                                    .add_context(
                                                        OnboardingContextItemId::HeritageId
                                                            .item(heritage_id.to_string()),
                                                        true,
                                                    );
                                            }
                                        }
                                        navigator()
                                            .push(Route::HeirWalletSpendView {
                                                heirwallet_name,
                                                heritage_id,
                                            });
                                    },
                                    DrawSvg::<Spend> {}
                                    "Spend"
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    fn place_holder() -> Self {
        Self {
            spend_infos: None,
            heir_config_type: <&str>::place_holder(),
            heir_config_fingerprint: CCStr::place_holder(),
            from_service: false,
            owner_email: None,
            total_amount: Some(UIBtcAmount::place_holder()),
            spendable_amount: Some(UIBtcAmount::place_holder()),
            spend_disabled: true,
            heritage_lines: vec![UIHeritageLine::place_holder()],
        }
    }
}

impl LoadedSuccessConversionMarker for TypeCouple<ContextualizedHeritages, UIHeritage> {}
impl FromRef<ContextualizedHeritages> for UIHeritage {
    fn from_ref(ContextualizedHeritages { context, heritages }: &ContextualizedHeritages) -> Self {
        let (spend_infos, from_service, owner_email) = match context {
            HeritageContext::WalletService { spend_infos, owner } => {
                (Some(spend_infos.clone()), true, owner.clone())
            }
            HeritageContext::Service { owner } => (None, true, owner.clone()),
            HeritageContext::WalletLocal { spend_infos } => {
                (Some(spend_infos.clone()), false, None)
            }
        };

        // We use some functional garantees in the following code, so start with assertions
        assert!(!heritages.is_empty());
        fn none_or_some<T>(f1: Option<T>, f2: Option<T>) -> bool {
            match (f1, f2) {
                (None, None) | (Some(_), Some(_)) => true,
                _ => false,
            }
        }
        assert!(heritages.windows(2).all(|pair| {
            let h1 = &pair[0];
            let h2 = &pair[1];
            h1.heritage_id == h2.heritage_id && h1.heir_config == h2.heir_config
            // Inside an Heritage group, it is a functionnal guarantee that each fields are either all None, or all Some
            // because it represent the permissions of the user when Heritages come from the Heritage Service
            // and the user's permission cannot change mid-heritages generation.
            && none_or_some(h1.value, h2.value)
            && none_or_some(h1.maturity, h2.maturity)
            && none_or_some(h1.next_heir_maturity, h2.next_heir_maturity)
            && none_or_some(h1.heir_position, h2.heir_position)
            && none_or_some(h1.heirs_count, h2.heirs_count)
        }));

        let now = timestamp_now();

        // All heritages have the same heritage_id and heir_config
        let first_heritage = &heritages[0];
        let spend_disabled = !heritages
            .iter()
            .any(|h| h.maturity.is_some_and(|ts| ts < now));

        let heir_config_type = heir_config_type_to_string(&first_heritage.heir_config);

        let heir_config_fingerprint =
            CCStr::from(first_heritage.heir_config.fingerprint().to_string());

        // Calculate total and spendable amounts
        let total_amount = heritages.iter().map(|h| h.value).sum::<Option<Amount>>();

        let spendable_amount = heritages
            .iter()
            .filter_map(|h| h.maturity.is_some_and(|ts| ts < now).then_some(h.value))
            .sum::<Option<Amount>>();

        // Group heritages by (maturity, next_heir_maturity, heir_position)
        let grouped_heritages = heritages.iter().fold(HashMap::new(), |mut acc, heritage| {
            let key = (
                heritage.maturity,
                heritage.next_heir_maturity,
                heritage.heir_position,
            );
            acc.entry(key).or_insert_with(Vec::new).push(heritage);
            acc
        });

        // Create heritage lines from grouped heritages
        let mut heritage_lines = Vec::new();
        for ((maturity, next_heir_maturity, heir_position), group) in grouped_heritages {
            let total_amount = group.iter().map(|h| h.value).sum::<Option<Amount>>();

            let amount = total_amount.map(UIBtcAmount::from);

            // Determine if this heritage line is spendable (maturity has passed)
            let spendable = maturity.is_some_and(|ts| ts < now);

            let maturity = match maturity {
                Some(ts) => UITimestamp::new_date_only(ts),
                None => UITimestamp::none(),
            };

            let expiration = match next_heir_maturity {
                Some(Some(ts)) => UITimestamp::new_date_only(ts),
                Some(None) => UITimestamp::never(),
                None => UITimestamp::none(),
            };

            let position = UIHeirPosition {
                pos: heir_position,
                count: first_heritage.heirs_count,
            };

            heritage_lines.push(UIHeritageLine {
                spendable,
                maturity,
                amount,
                position,
                expiration,
            });
        }

        let total_amount = total_amount.map(UIBtcAmount::from);
        let spendable_amount = spendable_amount.map(UIBtcAmount::from);

        Self {
            spend_infos,
            heir_config_type,
            heir_config_fingerprint,
            from_service,
            owner_email,
            total_amount,
            spendable_amount,
            spend_disabled,
            heritage_lines,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
struct UIHeritageLine {
    spendable: bool,
    maturity: UITimestamp,
    amount: Option<UIBtcAmount>,
    position: UIHeirPosition,
    expiration: UITimestamp,
}
impl LoadedElement for UIHeritageLine {
    type Loader = TransparentLoader;
    #[inline(always)]
    fn element<M: LoadedComponentInputMapper>(self, m: M) -> Element {
        rsx! {
            tr { class: if self.spendable { "text-base-content" } else { "text-(--color-base-content)/60" },
                td {
                    LoadedComponent { input: m.map(self.maturity) }
                }
                td {
                    if let Some(amount) = self.amount {
                        LoadedComponent { input: m.map(amount) }
                    } else {
                        "-"
                    }
                }
                td {
                    LoadedComponent { input: m.map(self.position) }
                }
                td {
                    LoadedComponent { input: m.map(self.expiration) }
                }
            }
        }
    }
    fn place_holder() -> Self {
        Self {
            spendable: true,
            maturity: UITimestamp::place_holder(),
            amount: Some(UIBtcAmount::place_holder()),
            position: UIHeirPosition::place_holder(),
            expiration: UITimestamp::place_holder(),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
struct UIHeirPosition {
    pos: Option<u8>,
    count: Option<u8>,
}
impl LoadedElement for UIHeirPosition {
    type Loader = SkeletonLoader;
    #[inline(always)]
    fn element<M: LoadedComponentInputMapper>(self, _m: M) -> Element {
        match (self.pos, self.count) {
            (Some(pos), Some(count)) => rsx! {
                span { "{pos}/{count}" }
            },
            (Some(pos), None) => rsx! {
                span { "{pos}/?" }
            },
            (None, Some(count)) => rsx! {
                span { "?/{count}" }
            },
            (None, None) => rsx! {
                span { "-" }
            },
        }
    }
    fn place_holder() -> Self {
        Self {
            pos: Some(1),
            count: Some(3),
        }
    }
}
