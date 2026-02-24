use leptos::prelude::*;
use leptos_router::components::A;
use wasm_bindgen_futures::spawn_local;

use crate::app::WalletState;
use crate::services::api;

#[component]
pub fn Landing() -> impl IntoView {
    let user_wallet = expect_context::<WalletState>();

    // Live protocol stats
    let (devs_verified, set_devs_verified) = signal(String::from("--"));
    let (badges_minted, set_badges_minted) = signal(String::from("--"));
    let (repos_verified, set_repos_verified) = signal(String::from("--"));

    // Fetch on mount
    spawn_local(async move {
        if let Ok(stats) = api::fetch_protocol_stats().await {
            set_devs_verified.set(stats.dev_badges_minted.to_string());
            set_badges_minted
                .set((stats.dev_badges_minted as u32 + stats.repo_badges_minted).to_string());
            set_repos_verified.set(stats.repo_badges_minted.to_string());
        }
    });

    view! {
        <section class="hero">
            <div class="hero-content">
                <div class="ghost-mascot">"👻"</div>
                <h1>
                    "Ghost"
                    <span class="highlight">"Check"</span>
                </h1>
                <p class="tagline">"Anonymous Onchain Reputation"</p>
                <p class="description">
                    "Prove your developer credentials without revealing your identity."
                    "Mint verifiable badges backed by your Github activity."
                </p>
                {move || {
                    if user_wallet.address.get().is_some() {
                        view! {
                            <A href="/dashboard" attr:class="btn-primary">"GO TO DASHBOARD"</A>
                        }.into_any()
                    } else {
                        view! {
                            <A href="/connect" attr:class="btn-primary">"CONNECT WALLET"</A>
                        }.into_any()
                    }
                }}

            </div>
        </section>

        // Features : 3 cards (Dev Badge, Repo Badge, Privacy First)
        <section class="features">
            <div class="features-grid">
                <div class="feature-card">
                    <div class="feature-icon">"🏆"</div>
                    <h3>"Dev Badge"</h3>
                    <p>"Aggregate your entire GitHub profile into a single verifiable credential. Total repos, commits, and languages — all onchain."</p>
                </div>

                <div class="feature-card">
                    <div class="feature-icon">"📦"</div>
                    <h3>"Repo Badge"</h3>
                    <p>"Prove ownership and contribution to specific repositories. Stars, commit count, and primary languages stored as NFT metadata."</p>
                </div>

                <div class="feature-card">
                    <div class="feature-icon">"🔒"</div>
                    <h3>"Privacy First"</h3>
                    <p>"Your GitHub username is hashed onchain. Other protocols can verify your skills without ever knowing your identity."</p>
                </div>
            </div>
        </section>

        <section class="stats-bar">
            <div class="stat">
                <span class="stat-value">{move || devs_verified.get()}</span>
                <span class="stat-label">"DEVS VERIFIED"</span>
            </div>
            <div class="stat">
                <span class="stat-value">{move || badges_minted.get()}</span>
                <span class="stat-label">"BADGES MINTED"</span>
            </div>
            <div class="stat">
                <span class="stat-value">{move || repos_verified.get()}</span>
                <span class="stat-label">"REPO VERIFIED"</span>
            </div>
        </section>
    }
}
