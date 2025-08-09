use crate::prelude::*;

use crate::{
    components::svg::{
        ArrowRight, ArrowSplitVertical, DrawSvg, LockCheck, SvgSize::Size6, SvgSize::Size8,
    },
    utils::async_sleep,
};

#[component]
pub fn SplashScreenView() -> Element {
    log::debug!("SplashScreenView Rendered");

    let mut show_logo = use_signal(|| false);
    let mut show_title = use_signal(|| false);
    let mut show_features = use_signal(|| false);
    let mut show_button = use_signal(|| false);

    // Animation sequence
    use_future(move || async move {
        // Initial delay
        async_sleep(500).await;

        // Start logo animation
        show_logo.set(true);
        async_sleep(2000).await;

        match *state_management::ONBOARDING_STATUS.read() {
            OnboardingStatus::Pending => {
                // Show title
                show_title.set(true);
                async_sleep(1000).await;

                // Show features
                show_features.set(true);
                async_sleep(1000).await;

                // Show get started button
                show_button.set(true);
            }
            OnboardingStatus::InProgress(ref ob) => {
                let next_route = ob.current_route();
                use_navigator().push(next_route);
            }
            OnboardingStatus::Completed => {
                navigator().push(crate::Route::WalletListView {});
            }
        };
    });

    use_drop(|| log::debug!("SplashScreenView Dropped"));

    rsx! {
        div {
            class: "hero min-h-screen overflow-hidden",
            class: if show_logo() { "transition-all duration-2000" } else { "items-start" },
            div { class: "hero-content flex-col text-center",

                // Logo section with scale animation
                div { class: if show_logo() { "transition-all transform duration-2000 ease-out" } else { "translate-y-[calc(50vh-50%)]" },
                    img {
                        src: asset!("/assets/crypto7world-logo.png"),
                        class: "mx-auto drop-shadow-2xl",
                    }
                }

                // Title with typewriter effect
                AppearFrom { show: show_title,
                    h1 { class: "text-5xl lg:text-7xl font-bold bg-gradient-to-r from-primary to-base-content bg-clip-text text-transparent pb-4",
                        "Heritage Wallet"
                    }
                    // Subtitle with slide up animation
                    p { class: "text-xl lg:text-2xl text-base-content/80 leading-relaxed",
                        "Secure your Bitcoin legacy with advanced inheritance planning"
                    }
                }

                // Feature highlights with staggered animation

                AppearFrom { show: show_features,
                    div { class: "grid grid-cols-1 md:grid-cols-3 gap-8 mb-8",

                        // Feature 1
                        FeatureBox {
                            div { class: "p-4 rounded-full text-green-600 bg-green-100 mb-4",
                                DrawSvg::<LockCheck> { size: Size8 }
                            }
                            h3 { class: "text-lg font-semibold mb-2", "Secure Inheritance" }
                            p { class: "text-base-content/70 text-center",
                                "Advanced cryptographic protection for your Bitcoin legacy"
                            }
                        }

                        // Feature 2
                        FeatureBox {
                            div { class: "p-4 rounded-full text-orange-600 bg-orange-100 mb-4",
                                DrawSvg::<ArrowSplitVertical> { size: Size8 }
                            }
                            h3 { class: "text-lg font-semibold mb-2", "Split Wallet Architecture" }
                            p { class: "text-base-content/70 text-center",
                                "Separate online and offline components for maximum security"
                            }
                        }

                        // Feature 3
                        FeatureBox {
                            div { class: "p-4 rounded-full text-blue-600 bg-blue-100 mb-4",
                                DrawSvg::<ArrowRight> { size: Size8 }
                            }
                            h3 { class: "text-lg font-semibold mb-2", "Easy Setup" }
                            p { class: "text-base-content/70 text-center",
                                "Intuitive interface for complex inheritance planning"
                            }
                        }
                    }
                }


                // Call to action button with bounce animation
                div { class: if show_button() { "transform transition-all duration-1000 ease-out scale-100 opacity-100" } else { "scale-0 opacity-0" },
                    button {
                        class: "btn btn-primary btn-lg px-12 py-4 text-lg font-semibold shadow-2xl hover:shadow-primary/25 hover:scale-105 group animate-pulse",
                        onclick: move |_| {
                            navigator().push(crate::Route::OnboardingWhoView {});
                        },
                        span { "Get Started" }
                        DrawSvg::<ArrowRight> {
                            size: Size6,
                            base_class: "ml-2 group-hover:translate-x-4 transition-transform duration-300 fill-current",
                        }
                    }
                }
                div { class: if show_button() { "transform transition-all duration-1000 ease-out scale-100 opacity-100" } else { "scale-0 opacity-0" },
                    div { class: "text-sm",
                        p { "Bitcoin Heritage Wallet • Secure • Private • Reliable" }
                    }
                }
            }
        }
    }
}

#[component]
fn AppearFrom(show: ReadOnlySignal<bool>, children: Element) -> Element {
    let mut animate = use_signal(|| false);
    use_effect(move || animate.set(show()));
    rsx! {
        div { class: if animate() { "transform transition-all duration-1000 ease-out opacity-100 overflow-y-visible" } else { "translate-y-32 opacity-0" },
            {children}
        }
    }
}

#[component]
fn FeatureBox(children: Element) -> Element {
    rsx! {
        div { class: "flex flex-col items-center p-6 rounded-2xl backdrop-blur-sm border border-base-300",
            {children}
        }
    }
}
