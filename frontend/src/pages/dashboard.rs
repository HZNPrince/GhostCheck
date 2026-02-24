use leptos::prelude::*;
use wasm_bindgen_futures::spawn_local;

use crate::{
    app::{GithubState, WalletState},
    services::{
        api::{self, DevMetrics, OnChainDevBadge, RepoMetrics},
        solana,
    },
};

#[component]
pub fn Dashboard() -> impl IntoView {
    // Context
    let github = expect_context::<GithubState>();
    let wallet = expect_context::<WalletState>();

    // Badge detection signals
    let (badge_loading, set_badge_loading) = signal(true);
    let (on_chain_badge, set_on_chain_badge) = signal(Option::<OnChainDevBadge>::None);

    // DEV_BADGE mint flow signals
    let (dev_metrics, set_dev_metrics) = signal(Option::<DevMetrics>::None);
    let (loading, set_loading) = signal(false);
    let (error, set_error) = signal(Option::<String>::None);

    // REPO_BADGE signals
    let (repo_input, set_repo_input) = signal(String::new());
    let (repo_metrics, set_repo_metrics) = signal(Option::<RepoMetrics>::None);
    let (repo_loading, set_repo_loading) = signal(false);
    let (repo_error, set_repo_error) = signal(Option::<String>::None);

    // MINTING signals
    let (minting, set_minting) = signal(false);
    let (mint_result, set_mint_result) = signal(Option::<String>::None);
    let (mint_error, set_mint_error) = signal(Option::<String>::None);

    // Watch wallet signal — when wallet connects, check for existing badge
    Effect::new(move |_| {
        let addr = wallet.address.get();
        if let Some(wallet_str) = addr {
            spawn_local(async move {
                match api::check_dev_badge(&wallet_str).await {
                    Ok(badge) => {
                        if badge.exists {
                            set_on_chain_badge.set(Some(badge));
                        }
                    }
                    Err(e) => log::warn!("Badge check failed: {}", e),
                }
                set_badge_loading.set(false);
            });
        } else {
            set_badge_loading.set(false);
        }
    });

    // Fetch dev metrics from GitHub (for new users who need to mint)
    let fetch_dev = move |_| {
        set_loading.set(true);
        set_error.set(None);

        spawn_local(async move {
            match api::fetch_github_metrics().await {
                Ok(dev_metrics) => {
                    log::info!(
                        "Got metrics: {} repos, {} commits",
                        dev_metrics.repo_count,
                        dev_metrics.total_commit
                    );
                    set_dev_metrics.set(Some(dev_metrics));
                }
                Err(e) => {
                    log::info!("Error fetching dev metrics {}", e);
                    set_error.set(Some(e));
                }
            }
            set_loading.set(false);
        });
    };

    // Mint dev badge (new users only)
    let mint_dev = move |_| {
        let metrics = dev_metrics.get();
        if metrics.is_none() {
            set_mint_error.set(Some("Fetch metrics first".to_string()));
            return;
        }
        let m = metrics.unwrap();
        set_minting.set(true);
        set_mint_error.set(None);
        set_mint_result.set(None);

        spawn_local(async move {
            match solana::build_and_send_dev_badge_tx(
                m.signature.clone(),
                m.signed_message.clone(),
                m.public_key_bytes.clone(),
                m.hashed_username.clone(),
                m.repo_count,
                m.owned_repo_count,
                m.total_stars,
                m.total_commit,
                m.prs_merged,
                m.issues_closed,
                m.followers,
                m.account_age_days,
                m.reputation_level,
            )
            .await
            {
                Ok(sig) => {
                    let tx_sig = sig.as_string().unwrap_or("unknown".to_string());
                    log::info!("Dev badge minted! Tx: {}", tx_sig);
                    set_mint_result.set(Some(tx_sig));

                    // Show the badge card now (use the GitHub metrics as a proxy for display)
                    set_on_chain_badge.set(Some(OnChainDevBadge {
                        exists: true,
                        repo_counts: m.repo_count,
                        owned_repo_counts: m.owned_repo_count,
                        total_stars: m.total_stars,
                        total_commits: m.total_commit,
                        prs_merged: m.prs_merged,
                        issues_closed: m.issues_closed,
                        followers: m.followers,
                        account_age_days: m.account_age_days,
                        reputation_level: m.reputation_level,
                        verified_repos: 0,
                        vouch_count: 0,
                    }));
                }
                Err(e) => {
                    let err_msg = format!("{:?}", e);
                    log::error!("Mint failed: {}", err_msg);
                    set_mint_error.set(Some(err_msg));
                }
            }
            set_minting.set(false);
        });
    };

    // Fetch repo metrics from GitHub
    let fetch_repo = move |_| {
        set_repo_loading.set(true);
        set_repo_error.set(None);

        let repo = repo_input.get();
        if repo.is_empty() {
            set_repo_error.set(Some(String::from("Enter a repo name")));
            set_repo_loading.set(false);
            return;
        }

        spawn_local(async move {
            match api::fetch_repo_metrics(&repo).await {
                Ok(repo_metrics) => {
                    log::info!(
                        "Repo: {:?}, commits: {}, stars: {}",
                        String::from_utf8(repo_metrics.repo_name_bytes.clone()),
                        repo_metrics.commits,
                        repo_metrics.stars
                    );
                    set_repo_metrics.set(Some(repo_metrics));
                }
                Err(e) => {
                    log::info!("Error fetching repo_metrics: {}", e);
                    set_repo_error.set(Some(e));
                }
            }
            set_repo_loading.set(false);
        });
    };

    // Mint repo badge + save to DB + reset form for another
    let wallet_for_mint = wallet.address;
    let mint_repo = move |_| {
        let metrics = repo_metrics.get();
        if metrics.is_none() {
            set_mint_error.set(Some("Fetch repo metrics first".to_string()));
            return;
        }
        let m = metrics.unwrap();
        let current_wallet = wallet_for_mint.get().unwrap_or_default();
        let repo_name_str = repo_input.get();

        set_minting.set(true);
        set_mint_error.set(None);
        set_mint_result.set(None);

        spawn_local(async move {
            // Pad repo_name_bytes to 32 bytes for the PDA seed
            let mut repo_name_padded = m.repo_name_bytes.clone();
            repo_name_padded.resize(32, 0);

            match solana::build_and_send_repo_badge_tx(
                m.signature.clone(),
                m.signed_message.clone(),
                m.public_key_bytes.clone(),
                repo_name_padded,
                m.hashed_username.clone(),
                m.stars,
                m.commits,
                m.fork_count,
                m.issues_open_count,
                m.is_fork,
                m.lang1_bytes.clone(),
                m.lang2_bytes.clone(),
            )
            .await
            {
                Ok(sig) => {
                    let tx_sig = sig.as_string().unwrap_or("unknown".to_string());
                    log::info!("Repo badge minted! Tx: {}", tx_sig);
                    set_mint_result.set(Some(tx_sig));

                    // Save to DB for future lookups
                    if let Err(e) = api::save_repo_mint(&current_wallet, &repo_name_str).await {
                        log::warn!("Failed to save to DB: {}", e);
                    }

                    // Reset form for another repo mint
                    set_repo_input.set(String::new());
                    set_repo_metrics.set(None);
                }
                Err(e) => {
                    let err_msg = format!("{:?}", e);
                    log::error!("Repo mint failed: {}", err_msg);
                    set_mint_error.set(Some(err_msg));
                }
            }
            set_minting.set(false);
        });
    };

    // View
    view! {
    <section class="dashboard">
        <h2 class="dashboard-title">"Dashboard"</h2>

        // Dev badge section
        {move || {
            if badge_loading.get() {
                view! {
                    <div class="tab-panel">
                        <div class="ghost-mascot">"⏳"</div>
                        <h3>"Checking badge status..."</h3>
                    </div>
                }.into_any()
            } else if let Some(badge) = on_chain_badge.get() {
                // Returning user — simple verified banner
                view! {
                    <div class="badge-verified-banner">
                        <div class="banner-left">
                            <span class="banner-icon">"✅"</span>
                            <div>
                                <span class="banner-title">"Dev Badge Verified"</span>
                                <span class="banner-sub">"Lv."{badge.reputation_level.to_string()}" · "{badge.verified_repos.to_string()}" repos verified"</span>
                            </div>
                        </div>
                        <a href="/profile" class="banner-link">"View Profile →"</a>
                    </div>
                    {move || mint_result.get().map(|sig| view! {
                        <p class="success-msg">"Minted! Tx: "{sig}</p>
                    })}
                }.into_any()
            } else if github.username.get().is_none() {
                view! {
                    <div class="tab-panel">
                        <div class="ghost-mascot">"👻"</div>
                        <h3>"Connect GitHub First"</h3>
                        <p>"Authorize GitHub from the navbar to get started"</p>
                    </div>
                }.into_any()
            } else if let Some(metrics) = dev_metrics.get() {
                // Pre-mint: show stats in dev-card style
                view! {
                    <div class="dev-card">
                        <div class="dev-card-accent"></div>
                        <div class="dev-card-header">
                            <div class="dev-card-identity">
                                <span class="dev-card-icon">"📊"</span>
                                <div>
                                    <h3 class="dev-card-name">"Your Dev Stats"</h3>
                                    <span class="dev-card-level">"Ready to mint"</span>
                                </div>
                            </div>
                            <span class="dev-card-badge">"PREVIEW"</span>
                        </div>
                        <div class="dev-card-stats">
                            <div class="dev-stat">
                                <span class="dev-stat-val">{metrics.repo_count.to_string()}</span>
                                <span class="dev-stat-lbl">"Repos"</span>
                            </div>
                            <div class="dev-stat">
                                <span class="dev-stat-val">{metrics.total_commit.to_string()}</span>
                                <span class="dev-stat-lbl">"Commits"</span>
                            </div>
                            <div class="dev-stat">
                                <span class="dev-stat-val">{metrics.total_stars.to_string()}</span>
                                <span class="dev-stat-lbl">"Stars"</span>
                            </div>
                            <div class="dev-stat">
                                <span class="dev-stat-val">{metrics.prs_merged.to_string()}</span>
                                <span class="dev-stat-lbl">"PRs"</span>
                            </div>
                            <div class="dev-stat">
                                <span class="dev-stat-val">{metrics.issues_closed.to_string()}</span>
                                <span class="dev-stat-lbl">"Issues"</span>
                            </div>
                            <div class="dev-stat">
                                <span class="dev-stat-val">{metrics.followers.to_string()}</span>
                                <span class="dev-stat-lbl">"Followers"</span>
                            </div>
                            <div class="dev-stat">
                                <span class="dev-stat-val">{metrics.account_age_days.to_string()}</span>
                                <span class="dev-stat-lbl">"Days"</span>
                            </div>
                            <div class="dev-stat">
                                <span class="dev-stat-val">{metrics.owned_repo_count.to_string()}</span>
                                <span class="dev-stat-lbl">"Owned"</span>
                            </div>
                        </div>
                        <div class="dev-card-actions">
                            <button class="btn-primary"
                                on:click=mint_dev
                                disabled=move || minting.get()
                            >
                                {move || if minting.get() { "MINTING..." } else { "MINT DEV BADGE" }}
                            </button>
                        </div>
                        {move || mint_result.get().map(|sig| view! {
                            <p class="success-msg">"Minted! Tx: "{sig}</p>
                        })}
                        {move || mint_error.get().map(|e| view! {
                            <p class="error-msg">{e}</p>
                        })}
                        <div class="dev-card-watermark">"GHOSTCHECK"</div>
                    </div>
                }.into_any()
            } else {
                view! {
                    <div class="tab-panel">
                        <div class="ghost-mascot">"🐙"</div>
                        <h3>"GitHub Connected!"</h3>
                        <p>"Fetch your stats to mint your Developer Badge"</p>
                        <button class="btn-primary" on:click=fetch_dev disabled=move || loading.get()>
                            {move || if loading.get() { "FETCHING..." } else { "GET DEV STATS" }}
                        </button>
                    </div>
                }.into_any()
            }
        }}

        // Repo verification section
        {move || {
            if github.username.get().is_none() {
                return view! { <div></div> }.into_any();
            }
            if let Some(metrics) = repo_metrics.get() {
                // Repo stats in compact card style
                let repo_name = String::from_utf8(metrics.repo_name_bytes.clone()).unwrap_or_default();
                let lang1 = String::from_utf8(metrics.lang1_bytes.clone()).unwrap_or_default();
                let lang2 = String::from_utf8(metrics.lang2_bytes.clone()).unwrap_or_default();
                view! {
                    <div class="dev-card" style="margin-top: 2rem;">
                        <div class="dev-card-accent" style="background: linear-gradient(90deg, #3b82f6, #8b5cf6, #3b82f6);"></div>
                        <div class="dev-card-header">
                            <div class="dev-card-identity">
                                <span class="dev-card-icon">"📦"</span>
                                <div>
                                    <h3 class="dev-card-name">{repo_name}</h3>
                                    <span class="dev-card-level">"Repository Badge"</span>
                                </div>
                            </div>
                            <div class="repo-card-langs">
                                {if !lang1.is_empty() {
                                    Some(view! { <span class="lang-tag">{lang1}</span> })
                                } else { None }}
                                {if !lang2.is_empty() {
                                    Some(view! { <span class="lang-tag">{lang2}</span> })
                                } else { None }}
                            </div>
                        </div>
                        <div class="dev-card-stats">
                            <div class="dev-stat">
                                <span class="dev-stat-val">{metrics.stars.to_string()}</span>
                                <span class="dev-stat-lbl">"Stars"</span>
                            </div>
                            <div class="dev-stat">
                                <span class="dev-stat-val">{metrics.commits.to_string()}</span>
                                <span class="dev-stat-lbl">"Commits"</span>
                            </div>
                            <div class="dev-stat">
                                <span class="dev-stat-val">{metrics.fork_count.to_string()}</span>
                                <span class="dev-stat-lbl">"Forks"</span>
                            </div>
                            <div class="dev-stat">
                                <span class="dev-stat-val">{metrics.issues_open_count.to_string()}</span>
                                <span class="dev-stat-lbl">"Issues"</span>
                            </div>
                        </div>
                        <div class="dev-card-actions">
                            <button class="btn-primary"
                                on:click=mint_repo
                                disabled=move || minting.get()
                            >
                                {move || if minting.get() { "MINTING..." } else { "MINT REPO BADGE" }}
                            </button>
                        </div>
                        {move || mint_result.get().map(|sig| view! {
                            <p class="success-msg">"Minted! Tx: "{sig}</p>
                        })}
                        {move || mint_error.get().map(|e| view! {
                            <p class="error-msg">{e}</p>
                        })}
                        <div class="dev-card-watermark">"GHOSTCHECK"</div>
                    </div>
                }.into_any()
            } else {
                view! {
                    <div class="profile-section" style="margin-top: 2rem;">
                        <h3 class="section-title">"Verify Repository"</h3>
                        <p class="section-sub">"Enter your repository name to verify and mint on-chain"</p>
                        <div class="repo-input-group">
                            <input
                                type="text"
                                placeholder="repo-name"
                                class="repo-input"
                                prop:value=move || repo_input.get()
                                on:input=move |ev| set_repo_input.set(event_target_value(&ev))
                            />
                            <button class="btn-verify"
                                on:click=fetch_repo
                                disabled=move || repo_loading.get()
                            >
                                {move || if repo_loading.get() { "VERIFYING..." } else { "VERIFY" }}
                            </button>
                        </div>
                        {move || repo_error.get().map(|e| view! { <p class="error-msg">{e}</p> })}
                    </div>
                }.into_any()
            }
        }}
        {move || error.get().map(|e| view! { <p class="error-msg">{e}</p> })}
    </section>
    }
}
