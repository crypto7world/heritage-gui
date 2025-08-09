use crate::components::onboarding::MaybeOnPathHighlight;
use crate::prelude::*;

use crate::{
    components::{
        app_config::AppConfig,
        onboarding::OnboardingMessage,
        svg::{Alert, DrawSvg, InfoCircle, Moon, Sun, SvgSize::Size10},
    },
    onboarding::OnboardingStep,
    Route,
};

#[component]
pub fn MainLayout() -> Element {
    log::debug!("MainLayout reload");

    let service_client_service = state_management::use_service_client_service();
    let resource_service_client_config = use_resource(move || async move {
        state_management::get_service_config(service_client_service).await
    });
    let heritage_service_website_url = use_memo(move || {
        // The API URL is https://api.{website_domain}/v1
        let ref_service_client_config = resource_service_client_config.read();
        let host = ref_service_client_config
            .as_ref()
            .map(|hsc| {
                hsc.service_api_url
                    .split("/")
                    .filter_map(|p| {
                        p.starts_with("api.")
                            .then(|| p.strip_prefix("api.").unwrap())
                    })
                    .next()
            })
            .flatten()
            .unwrap_or("btcherit.com");
        format!("https://{host}/")
    });

    use_drop(|| log::debug!("MainLayout Dropped"));

    rsx! {
        div { class: "relative min-h-dvh",
            OnboardingMessage {}
            header { class: "bg-base-100 fixed top-0 w-full z-20 shadow-lg shadow-base-content/10",
                NavBar {}
            }
            main { class: "pt-12 pb-16 mx-8 text-justify", Outlet::<Route> {} }
            footer { class: "absolute bottom-px w-full h-12 px-8 z-0",
                div { class: "h-px border-t border-solid border-gray-500" }
                Footer {}
            }

            OnboardingInfoModal {
                step: OnboardingStep::ModalCreateAccountOnTheService,
                btn_text: "I am logged-in the Heritage Service website",
                div { class: "flex flex-col gap-4 max-w-xl text-base",
                    p {
                        "In order to proceed, you must first create an account on the
                        Heritage Service. Make sure you are registered and logged-in
                        at the following URL before continuing the onboarding process:"
                    }
                    div { class: "p-4 bg-base-200 rounded-lg border",
                        a {
                            href: "{heritage_service_website_url.read()}",
                            target: "_blank",
                            rel: "noopener noreferrer",
                            class: "text-xl font-mono text-primary hover:text-primary-focus break-all",
                            "{heritage_service_website_url.read()}"
                        }
                    }
                }
            }

            OnboardingInfoModal {
                step: OnboardingStep::ModalInstallBlockchainProviderNode,
                btn_text: "My Bitcoin node is up and running",
                div { class: "flex flex-col gap-4 max-w-xl text-base",
                    p {
                        "To access the Bitcoin blockchain, Heritage Wallet requires a connection
                        to your own Bitcoin or Electrum node. This ensures maximum privacy and
                        security for your wallet operations."
                    }
                    p {
                        "Before continuing the onboarding process, please ensure that your Bitcoin or Electrum node is:"
                    }
                    ul { class: "list-disc list-inside ml-4 space-y-2",
                        li { "Installed and running on your system" }
                        li { "Fully synchronized with the Bitcoin network" }
                        li { "Configured to accept RPC connections (for Bitcoin Core)" }
                        li { "Ready to provide blockchain data to external applications" }
                    }
                    div { class: "flex flex-col gap-3 mt-4",
                        div { class: "p-4 bg-base-200 rounded-lg border",
                            h4 { class: "font-semibold mb-2", "Installation Guides:" }
                            div { class: "flex flex-col gap-2",
                                div {
                                    strong { "Bitcoin Core: " }
                                    a {
                                        href: "https://bitcoin.org/en/full-node#setup-a-full-node",
                                        target: "_blank",
                                        rel: "noopener noreferrer",
                                        class: "text-primary hover:text-primary-focus underline",
                                        "Official Bitcoin Core Setup Guide"
                                    }
                                }
                                div {
                                    strong { "Electrum: " }
                                    a {
                                        href: "https://electrum.org/#download",
                                        target: "_blank",
                                        rel: "noopener noreferrer",
                                        class: "text-primary hover:text-primary-focus underline",
                                        "Official Electrum Download"
                                    }
                                }
                                div {
                                    strong { "Electrs: " }
                                    a {
                                        href: "https://github.com/romanz/electrs/blob/master/doc/install.md",
                                        target: "_blank",
                                        rel: "noopener noreferrer",
                                        class: "text-primary hover:text-primary-focus underline",
                                        "Installation Readme"
                                    }
                                }
                            }
                        }
                    }
                    p {
                        "We suggest using Electrs, since that is what the Heritage Service use
                        internally, but Electrum RPC server shoud work as well. In the next steps,
                        you will be able to configure the connection details including the node
                        URI and authentication credentials if using a Bitcoin Core node."
                    }
                }
            }

            OnboardingInfoModal { step: OnboardingStep::ModalLocalInheritance,
                div { class: "flex flex-col gap-4 max-w-xl text-base",
                    p {
                        "You are attempting to retrieve an inheritance without the Heritage
                        Service. This requires both your heir seed mnemonic phrase and the "
                        span { class: "font-bold", "complete Heritage Wallet backup" }
                        " from the original wallet owner."
                    }
                    div { class: "alert alert-info text-base font-semibold",
                        DrawSvg::<InfoCircle> {}
                        p {
                            "The Heritage Wallet Descriptors backup is a JSON file containing
                            Bitcoin descriptors and it should have been provided to you by
                            the original wallet owner."
                        }
                    }
                    div { class: "alert alert-warning text-base font-semibold",
                        DrawSvg::<Alert> {}
                        p {
                            "Without the Heritage Wallet Descriptors backup file, "
                            span { class: "font-black bg-error uppercase",
                                "accessing your inheritance is impossible"
                            }
                            ", even with the correct seed mnemonic phrase."
                        }
                    }
                    LocalWalletMoreInfo {}
                }
            }

            OnboardingInfoModal {
                step: OnboardingStep::ModalExplainHeritageBackup,
                btn_text: "I understand",
                div { class: "flex flex-col gap-4 max-w-2xl text-base",
                    div { class: "flex flex-col gap-4",
                        h3 { class: "text-lg font-semibold text-warning flex items-center gap-2",
                            DrawSvg::<Alert> {}
                            "Critical: Heritage Wallet Descritpors Backups"
                        }
                        p {
                            "You have created a local Heritage Wallet. This means you are solely
                            responsible for ensuring the safety of your bitcoins and that your
                            heirs can access their inheritance."
                        }
                        p {
                            "An "
                            span { class: "font-bold", "Heritage Wallet Descritpors Backup" }
                            " contains all the "
                            span { class: "italic", "Bitcoin Descriptors" }
                            " ever used in your Heritage Wallet, and their knowledge is critical
                            to be able to spend from your wallet."
                        }
                        p {
                            "Heritage Wallets use TapRoot technology and advanced Bitcoin scripts to
                            enable inheritance while preserving your full control over your coins.
                            However, this technology requires complete Bitcoin descriptors to
                            reconstruct spending conditions - "
                            span { class: "text-error font-black uppercase",
                                "private keys alone are insufficient"
                            }
                            " to spend bitcoins from your wallet."
                        }
                        div { class: "text-lg p-4 bg-base-200 rounded-lg border",

                            "If you loose these Bitcoin descriptors, you "
                            span { class: "text-error font-black uppercase", "will" }
                            " loose all your bitcoins and "
                            span { class: "text-error font-bold", "no heir will be able to access them" }
                            ". Your heirs will need "
                            span { class: "text-error font-black uppercase", "both" }
                            " their seed mnemonic phrase "
                            span { class: "text-error font-black uppercase", "and" }
                            " your complete Heritage Wallet backup to claim their inheritance."
                        }
                        div { class: "alert alert-warning text-base font-semibold",
                            DrawSvg::<Alert> {}
                            p {
                                "Each time you update your Heritage Configuration, you "
                                span { class: "bg-error font-black uppercase", "must" }
                                " re-generate an "
                                span { class: "font-bold", "Heritage Wallet Backup" }
                                " and provide it to your heirs through a secure
                                method of your choosing."
                            }
                        }
                        div { class: "alert alert-info text-base font-semibold",
                            DrawSvg::<InfoCircle> {}
                            div {
                                p { class: "px-2",
                                    "The Heritage Service exists specifically to solve this challenge
                                by securely and durably storing descriptors and making inheritance
                                seamless for heirs."
                                }
                                p { class: "text-xl bg-base-300 text-warning border-2 rounded-box border-warning px-2",
                                    "If you're uncertain about managing backups yourself,
                                    consider using the Heritage Service instead."
                                }
                            }
                        }
                        LocalWalletMoreInfo {}
                    }
                }
            }
        
        }
    }
}

#[component]
fn LocalWalletMoreInfo() -> Element {
    rsx! {
        details { class: "collapse collapse-arrow bg-base-200 mt-2",
            summary { class: "collapse-title font-medium",
                "Learn more about Heritage Wallets and the underlying technology"
            }
            div { class: "collapse-content",
                div { class: "flex flex-col gap-3 pt-2",
                    div {
                        strong { "Why Heritage Wallets? " }
                        a {
                            href: "https://heritage.develop.dev.crypto7.world/docs/why-heritage",
                            target: "_blank",
                            rel: "noopener noreferrer",
                            class: "text-primary hover:text-primary-focus underline",
                            "Understanding Bitcoin inheritance challenges"
                        }
                    }
                    div {
                        strong { "TapRoot: " }
                        a {
                            href: "https://en.bitcoin.it/wiki/Taproot",
                            target: "_blank",
                            rel: "noopener noreferrer",
                            class: "text-primary hover:text-primary-focus underline",
                            "What is TapRoot?"
                        }
                    }
                    div {
                        strong { "Bitcoin Descriptors: " }
                        div { class: "flex flex-col gap-1 ml-4",
                            a {
                                href: "https://github.com/bitcoin/bips/blob/master/bip-0379.md",
                                target: "_blank",
                                rel: "noopener noreferrer",
                                class: "text-primary hover:text-primary-focus underline",
                                "BIP 379: Miniscript"
                            }
                            a {
                                href: "https://github.com/bitcoin/bips/blob/master/bip-0380.mediawiki",
                                target: "_blank",
                                rel: "noopener noreferrer",
                                class: "text-primary hover:text-primary-focus underline",
                                "BIP 380: Output Script Descriptors General Operation"
                            }
                            a {
                                href: "https://github.com/bitcoin/bips/blob/master/bip-0386.mediawiki",
                                target: "_blank",
                                rel: "noopener noreferrer",
                                class: "text-primary hover:text-primary-focus underline",
                                "BIP 386: tr() Output Script Descriptors"
                            }
                        }
                    }
                }
            }
        }
    }
}

#[component]
fn NavBar() -> Element {
    log::debug!("NavBar reload");

    let router = try_consume_context::<RouterContext>().expect("Router is present");
    let current_route = use_memo(move || router.current::<crate::Route>());
    let is_wallet_list = use_memo(move || matches!(current_route(), Route::WalletListView {}));
    let is_heir_list = use_memo(move || matches!(current_route(), Route::HeirListView {}));
    let is_heirwallet_list =
        use_memo(move || matches!(current_route(), Route::HeirWalletListView {}));

    use_drop(|| log::debug!("NavBar Dropped"));

    rsx! {
        nav { class: "h-12 px-2 flex flex-row gap-2",
            div { class: "h-full flex flex-none gap-2",
                img {
                    src: asset!("/assets/crypto7world-logo.png"),
                    class: "self-center flex-none h-11",
                }
                div {
                    div { class: "text-lg font-black text-nowrap", "Heritage Wallet" }
                    div { class: "text-xs text-primary italic", "by Crypto7.world" }
                }
            }
            div { class: "basis-10" }
            MaybeOnPathHighlight {
                steps: &[
                    OnboardingStep::ClickCreateWalletCard,
                    OnboardingStep::ClickWalletCardAfterHeirsCreation,
                ],
                progress: MaybeHighlightProgressType::Signal(is_wallet_list.into()),
                NavLink { route: Route::WalletListView {}, "Wallets" }
            }
            MaybeOnPathHighlight {
                steps: &[
                    OnboardingStep::ModalExplainHeirs,
                    OnboardingStep::ClickCreateHeirCard,
                    OnboardingStep::ClickHeirCard,
                    OnboardingStep::ModalMoreHeirOrWallet,
                ],
                progress: MaybeHighlightProgressType::Signal(is_heir_list.into()),
                NavLink { route: Route::HeirListView {}, "Heirs" }
            }
            MaybeOnPathHighlight {
                steps: &[OnboardingStep::ClickCreateHeirWalletCard, OnboardingStep::ClickHeirWalletCard],
                progress: MaybeHighlightProgressType::Signal(is_heirwallet_list.into()),
                NavLink { route: Route::HeirWalletListView {}, "Inheritances" }
            }
            div { class: "grow" }
            DarkModeToggle {}
            AppConfig {}
        }
    }
}

#[component]
fn NavLink(route: Route, children: Element) -> Element {
    rsx! {
        div { class: "basis-10 content-center flex",
            Link {
                class: "h-full px-4 content-center text-lg text-nowrap font-bold uppercase hover:bg-primary/10",
                active_class: "bg-primary/10 text-primary",
                to: route,
                {children}
            }
        }
    }
}
#[component]
fn Footer() -> Element {
    rsx! {
        div { class: "h-full text-primary text-right content-center", "2025 — Crypto7.world" }
    }
}

#[component]
fn DarkModeToggle() -> Element {
    rsx! {
        input {
            r#type: "checkbox",
            name: "theme",
            class: "theme-controller hidden",
            value: match state_management::THEME() {
                Theme::Light => "light",
                Theme::Dark => "dark",
            },
            tabindex: "-1",
            checked: true,
        }
        label { class: "swap swap-rotate",
            input {
                r#type: "checkbox",
                name: "theme",
                tabindex: "-1",
                oninput: move |event| {
                    *state_management::THEME.write() = match event.checked() {
                        true => Theme::Dark,
                        false => Theme::Light,
                    };
                },
                checked: matches!(state_management::THEME(), Theme::Dark),
            }
            DrawSvg::<Sun> { base_class: "swap-off fill-current", size: Size10 }
            DrawSvg::<Moon> { base_class: "swap-on fill-current", size: Size10 }
        }
    }
}
