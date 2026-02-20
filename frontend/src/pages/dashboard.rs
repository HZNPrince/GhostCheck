use leptos::prelude::*;
use wasm_bindgen_futures::spawn_local;

use crate::{
    app::GithubState,
    services::{
        api::{self, DevMetrics, RepoMetrics},
        solana,
    },
};

#[component]
pub fn Dashboard() -> impl IntoView {
    // Github context
    let github = expect_context::<GithubState>();

    // Track if user already has a dev badge (hides mint section)
    let (has_dev_badge, set_has_dev_badge) = signal(false);

    // DEV_BADGE fetch metrics signals
    let (dev_metrics, set_dev_metrics) = signal(Option::<DevMetrics>::None);
    let (loading, set_loading) = signal(false);
    let (error, set_error) = signal(Option::<String>::None);

    // REPO_BADGE fetch metrics signals
    let (repo_input, set_repo_input) = signal(String::new());
    let (repo_metrics, set_repo_metrics) = signal(Option::<RepoMetrics>::None);
    let (repo_loading, set_repo_loading) = signal(false);
    let (repo_error, set_repo_error) = signal(Option::<String>::None);

    // MINTING signals
    let (minting, set_minting) = signal(false);
    let (mint_result, set_mint_result) = signal(Option::<String>::None);
    let (mint_error, set_mint_error) = signal(Option::<String>::None);

    // Fetch Dev Metrics when the button is clicked
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

    // Mint dev badge — calls JS bridge via wasm-bindgen
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
                    set_has_dev_badge.set(true); //Hide Mint section
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

    // Mint repo badge
    let mint_repo = move |_| {
        let metrics = repo_metrics.get();
        if metrics.is_none() {
            set_mint_error.set(Some("Fetch repo metrics first".to_string()));
            return;
        }
        let m = metrics.unwrap();
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

    view! {
    <section class="dashboard">
        <h2 class="dashboard-title">"Dashboard"</h2>

        {
            move || {
                if has_dev_badge.get() {
                    // Already minted — don't show
                    view!{<div></div>}.into_any()
                }else if github.username.get().is_none() {
                    view! {
                        <div class="tab-panel">
                                                    <div class="ghost-mascot">"👻"</div>
                                                    <h3>"Connect GitHub First"</h3>
                                                    <p>"Authorize GitHub from the navbar to get started"</p>
                                                </div>
                    }.into_any()
                } else if let Some(metrics) = dev_metrics.get() {
                    // Fetched — show stats + mint button
                    view! {
                        <div class="tab-panel">
                            <h3>"Your Dev Stats"</h3>
                            <div class="metrics-grid">
                                <div class="metric-card">
                                    <span class="metric-value">{metrics.repo_count.to_string()}</span>
                                    <span class="metric-label">"REPOS"</span>
                                </div>
                                <div class="metric-card">
                                    <span class="metric-value">{metrics.total_commit.to_string()}</span>
                                    <span class="metric-label">"COMMITS"</span>
                                </div>
                                <div class="metric-card">
                                    <span class="metric-value">{metrics.total_stars.to_string()}</span>
                                    <span class="metric-label">"STARS"</span>
                                </div>
                                <div class="metric-card">
                                    <span class="metric-value">{metrics.prs_merged.to_string()}</span>
                                    <span class="metric-label">"PRs MERGED"</span>
                                </div>
                            </div>
                            <button class="btn-primary"
                                on:click=mint_dev
                                disabled=move || minting.get()
                            >
                                {move || if minting.get() { "⏳ MINTING..." } else { "🏆 MINT DEV BADGE" }}
                            </button>
                            {move || mint_result.get().map(|sig| view! {
                                <p class="success-msg">"✅ Minted! Tx: "{sig}</p>
                            })}
                            {move || mint_error.get().map(|e| view! {
                                <p class="error-msg">"❌ "{e}</p>
                            })}
                        </div>
                    }.into_any()
                } else {
                    // Connected but not fetched
                    view! {
                        <div class="tab-panel">
                            <div class="ghost-mascot">"🐙"</div>
                            <h3>"GitHub Connected!"</h3>
                            <p>"Fetch your stats to mint your Developer Badge"</p>
                            <button class="btn-primary" on:click=fetch_dev disabled=move || loading.get()>
                                {move || if loading.get() { "⏳ FETCHING..." } else { "📊 GET DEV STATS" }}
                            </button>
                        </div>
                    }.into_any()
                }
            }
        }

        // ══════ REPO VERIFICATION SECTION (always visible after GitHub connected) ══════
                   {move || {
                       if github.username.get().is_none() {
                           return view! { <div></div> }.into_any();
                       }
                       if let Some(metrics) = repo_metrics.get() {
                           view! {
                               <div class="tab-panel" style="margin-top: 2rem;">
                                   <h3>"Repo: " {String::from_utf8(metrics.repo_name_bytes.clone()).unwrap_or_default()}</h3>
                                   <div class="metrics-grid">
                                       <div class="metric-card">
                                           <span class="metric-value">{metrics.stars.to_string()}</span>
                                           <span class="metric-label">"STARS"</span>
                                       </div>
                                       <div class="metric-card">
                                           <span class="metric-value">{metrics.commits.to_string()}</span>
                                           <span class="metric-label">"COMMITS"</span>
                                       </div>
                                       <div class="metric-card">
                                           <span class="metric-value">{metrics.fork_count.to_string()}</span>
                                           <span class="metric-label">"FORKS"</span>
                                       </div>
                                       <div class="metric-card">
                                           <span class="metric-value">{metrics.issues_open_count.to_string()}</span>
                                           <span class="metric-label">"ISSUES"</span>
                                       </div>
                                   </div>
                                   <button class="btn-primary"
                                       on:click=mint_repo
                                       disabled=move || minting.get()
                                   >
                                       {move || if minting.get() { "⏳ MINTING..." } else { "📦 MINT REPO BADGE" }}
                                   </button>
                                   {move || mint_result.get().map(|sig| view! {
                                       <p class="success-msg">"✅ Minted! Tx: "{sig}</p>
                                   })}
                                   {move || mint_error.get().map(|e| view! {
                                       <p class="error-msg">"❌ "{e}</p>
                                   })}
                               </div>
                           }.into_any()
                       } else {
                           view! {
                               <div class="tab-panel" style="margin-top: 2rem;">
                                   <h3>"Verify Repository"</h3>
                                   <p>"Enter your repository name to verify and mint on-chain"</p>
                                   <div class="repo-input-group">
                                       <input
                                           type="text"
                                           placeholder="repo-name"
                                           class="repo-input"
                                           on:input=move |ev| set_repo_input.set(event_target_value(&ev))
                                       />
                                       <button class="btn-verify"
                                           on:click=fetch_repo
                                           disabled=move || repo_loading.get()
                                       >
                                           {move || if repo_loading.get() { "⏳ VERIFYING..." } else { "VERIFY" }}
                                       </button>
                                   </div>
                                   {move || repo_error.get().map(|e| view! { <p class="error-msg">{e}</p> })}
                               </div>
                           }.into_any()
                       }
                   }}
                   // Global error
                   {move || error.get().map(|e| view! { <p class="error-msg">{e}</p> })}
               </section>
           }
}
// Show errors
