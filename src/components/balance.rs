use crate::prelude::*;

use btc_heritage_wallet::{
    bitcoin::{Amount, Denomination, SignedAmount},
    btc_heritage::HeritageWalletBalance,
    heritage_service_api_client::HeritageWalletMeta,
    online_wallet::WalletStatus,
};

use crate::utils::{amount_to_signed, denomination_for_amount};

#[derive(Debug, Copy, Clone, PartialEq)]
pub struct UIBtcAmount {
    amount: Option<SignedAmount>,
    diff_style: bool,
}
impl UIBtcAmount {
    pub fn new(amount: Option<SignedAmount>, diff_style: bool) -> Self {
        Self { amount, diff_style }
    }
}
impl From<Amount> for UIBtcAmount {
    fn from(amount: Amount) -> Self {
        Self::from(Some(amount))
    }
}
impl LoadedSuccessConversionMarker for TypeCouple<Amount, UIBtcAmount> {}
impl FromRef<Amount> for UIBtcAmount {
    fn from_ref(amount: &Amount) -> Self {
        Self::from(Some(*amount))
    }
}
impl From<Option<Amount>> for UIBtcAmount {
    fn from(amount: Option<Amount>) -> Self {
        Self::new(amount.map(amount_to_signed), false)
    }
}

impl LoadedElement for UIBtcAmount {
    type Loader = SkeletonLoader;
    #[inline(always)]
    fn element<M: LoadedComponentInputMapper>(self, _m: M) -> Element {
        let (is_positive, is_negative, amount_s) = match self.amount {
            Some(signed_amount) => {
                let abs_amount = signed_amount
                    .checked_abs()
                    .unwrap_or_default()
                    .to_unsigned()
                    .unwrap();

                let amount_s = match denomination_for_amount(abs_amount) {
                    Denomination::Bitcoin => {
                        format!("{:+} BTC", signed_amount.display_in(Denomination::Bitcoin))
                    }
                    Denomination::MilliBitcoin => format!(
                        "{:+} mBTC",
                        signed_amount.display_in(Denomination::MilliBitcoin)
                    ),
                    Denomination::Satoshi => {
                        format!("{:+} sat", signed_amount.display_in(Denomination::Satoshi))
                    }
                    _ => unreachable!("denomination_for_amount never return another denom"),
                };
                (
                    signed_amount.is_positive() || signed_amount == SignedAmount::ZERO,
                    signed_amount.is_negative(),
                    amount_s,
                )
            }
            None => (false, false, "-".to_owned()),
        };

        let display_text = if !self.diff_style && is_positive {
            &amount_s[1..]
        } else {
            amount_s.as_str()
        };
        rsx! {
            span {
                class: "text-nowrap inline-block",
                class: if self.diff_style && is_positive { "text-green-500" },
                class: if self.diff_style && is_negative { "text-red-500" },
                {display_text}
            }
        }
    }
    fn place_holder() -> Self {
        Self {
            // Because "99.99999 mBTC" is the longuest displayable amount (excluding the unlikely chaps that have more than 1000 BTC)
            amount: Some(SignedAmount::from_btc(0.09999999).expect("valid float")),
            diff_style: true,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
struct UIBalanceInner {
    pub balance: UIBtcAmount,
    pub cur_balance: UIBtcAmount,
    pub obs_balance: UIBtcAmount,
}
impl UIBalanceInner {
    fn place_holder() -> Self {
        Self {
            balance: UIBtcAmount::place_holder(),
            cur_balance: UIBtcAmount::place_holder(),
            obs_balance: UIBtcAmount::place_holder(),
        }
    }
}
impl FromRef<HeritageWalletBalance> for UIBalanceInner {
    fn from_ref(heritage_balance: &HeritageWalletBalance) -> Self {
        let balance = SignedAmount::from_sat(heritage_balance.total_balance().get_total() as i64);
        let cur_balance =
            SignedAmount::from_sat(heritage_balance.uptodate_balance().get_total() as i64);
        let obs_balance =
            SignedAmount::from_sat(heritage_balance.obsolete_balance().get_total() as i64);
        Self {
            balance: UIBtcAmount {
                amount: Some(balance),
                diff_style: false,
            },
            cur_balance: UIBtcAmount {
                amount: Some(cur_balance),
                diff_style: false,
            },
            obs_balance: UIBtcAmount {
                amount: Some(obs_balance),
                diff_style: false,
            },
        }
    }
}

impl FromRef<WalletStatus> for UIBalanceInner {
    fn from_ref(value: &WalletStatus) -> Self {
        Self::from_ref(&value.balance)
    }
}
impl FromRef<HeritageWalletMeta> for UIBalanceInner {
    fn from_ref(value: &HeritageWalletMeta) -> Self {
        if let Some(ref heritage_balance) = value.balance {
            Self::from_ref(heritage_balance)
        } else {
            Self::from_ref(&HeritageWalletBalance::default())
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct UIBalanceSummary(UIBalanceInner);
impl LoadedElement for UIBalanceSummary {
    type Loader = TransparentLoader;
    #[inline(always)]
    fn element<M: LoadedComponentInputMapper>(self, m: M) -> Element {
        rsx! {
            div { class: "text-base", "Balance" }
            div {
                div { class: "text-3xl font-black",
                    LoadedComponent::<UIBtcAmount> { input: m.map(self.0.balance) }
                }
                div { class: "text-nowrap font-light text-sm",
                    "Current: "
                    span { class: "font-bold",
                        LoadedComponent::<UIBtcAmount> { input: m.map(self.0.cur_balance) }
                    }
                }
                div { class: "text-nowrap font-light text-sm",
                    "Obsolete: "
                    span { class: "font-bold",
                        LoadedComponent::<UIBtcAmount> { input: m.map(self.0.obs_balance) }
                    }
                }
            }
        }
    }
    fn place_holder() -> Self {
        Self(UIBalanceInner::place_holder())
    }
}
impl LoadedSuccessConversionMarker for TypeCouple<WalletStatus, UIBalanceSummary> {}
impl FromRef<WalletStatus> for UIBalanceSummary {
    fn from_ref(value: &WalletStatus) -> Self {
        UIBalanceSummary(UIBalanceInner::from_ref(value))
    }
}
impl LoadedSuccessConversionMarker for TypeCouple<HeritageWalletMeta, UIBalanceSummary> {}
impl FromRef<HeritageWalletMeta> for UIBalanceSummary {
    fn from_ref(value: &HeritageWalletMeta) -> Self {
        UIBalanceSummary(UIBalanceInner::from_ref(value))
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct UIWalletBalance(UIBalanceInner);
impl LoadedSuccessConversionMarker for TypeCouple<WalletStatus, UIWalletBalance> {}
impl FromRef<WalletStatus> for UIWalletBalance {
    fn from_ref(value: &WalletStatus) -> Self {
        UIWalletBalance(UIBalanceInner::from_ref(value))
    }
}

impl LoadedElement for UIWalletBalance {
    type Loader = TransparentLoader;
    #[inline(always)]
    fn element<M: LoadedComponentInputMapper>(self, m: M) -> Element {
        rsx! {
            div { class: "card card-xl h-64 bg-base-100 shadow-sm p-6",
                div { class: "card-title", "Balance" }
                div { class: "card-body",
                    div { class: "text-4xl font-black",
                        LoadedComponent::<UIBtcAmount> { input: m.map(self.0.balance) }
                    }
                    div { class: "font-light text-sm",
                        "Current Heritage Config: "
                        span { class: "font-bold",
                            LoadedComponent::<UIBtcAmount> { input: m.map(self.0.cur_balance) }
                        }
                    }
                    div { class: "font-light text-sm",
                        "Previous Heritage Config: "
                        span { class: "font-bold",
                            LoadedComponent::<UIBtcAmount> { input: m.map(self.0.obs_balance) }
                        }
                    }
                }
            }
        }
    }
    fn place_holder() -> Self {
        Self(UIBalanceInner::place_holder())
    }
}
