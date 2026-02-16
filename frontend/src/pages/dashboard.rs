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

    // Signal for which tab is open (dev, repo or my badges)
    let (active_tab, set_active_tab) = signal(String::from("dev"));

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

    // Mint dev badge — calls JS bridge via wasm-bindgen
    let mint_dev = move |_| {
        let metrics = dev_metrics.get();
        if metrics.is_none() {
            set_mint_error.set(Some("Fetch metrics first".to_string()));
            return;
        }
        let metrics = metrics.unwrap();
        set_minting.set(true);
        set_mint_error.set(None);
        set_mint_result.set(None);

        spawn_local(async move {
            match solana::build_and_send_dev_badge_tx(
                metrics.signature.clone(),
                metrics.signed_message.clone(),
                metrics.public_key_bytes.clone(),
                metrics.hashed_username.clone(),
                metrics.repo_count,
                metrics.total_commit,
            )
            .await
            {
                Ok(sig) => {
                    let tx_sig = sig.as_string().unwrap_or("unknown".to_string());
                    log::info!("Dev badge minted! Tx: {}", tx_sig);
                    set_mint_result.set(Some(tx_sig));
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

    // Mint repo badge
    let mint_repo = move |_| {
        let metrics = repo_metrics.get();
        if metrics.is_none() {
            set_mint_error.set(Some("Fetch repo metrics first".to_string()));
            return;
        }
        let metrics = metrics.unwrap();
        set_minting.set(true);
        set_mint_error.set(None);
        set_mint_result.set(None);

        spawn_local(async move {
            // Pad repo_name_bytes to 32 bytes for the PDA seed
            let mut repo_name_padded = metrics.repo_name_bytes.clone();
            repo_name_padded.resize(32, 0);

            match solana::build_and_send_repo_badge_tx(
                metrics.signature.clone(),
                metrics.signed_message.clone(),
                metrics.public_key_bytes.clone(),
                repo_name_padded,
                metrics.hashed_username.clone(),
                metrics.stars,
                metrics.commits,
                metrics.lang1_bytes.clone(),
                metrics.lang2_bytes.clone(),
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
            <div class="tabs">
                <button class=move || if active_tab.get() == "dev" {"tab_active"} else {"tab"} on:click=move |_| set_active_tab.set(String::from("dev"))>"Dev Badge"</button>
                <button class=move || if active_tab.get() == "repo" {"tab_active"} else {"tab"} on:click=move |_| set_active_tab.set(String::from("repo"))>"Repo Badge"</button>
                <button class=move || if active_tab.get() == "badges" {"tab_active"} else {"tab"} on:click=move |_| set_active_tab.set(String::from("badges"))>"My Badges"</button>
            </div>

            <div class="tab-content">
                {move|| match active_tab.get().as_str() {
                    "dev" => {
                        if let Some(metrics) = dev_metrics.get() {
                            // Already fetched — show stats
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
                                    <p class="hint-text">"Signature ready — click to mint your badge onchain"</p>
                                </div>
                            }.into_any()
                        } else if github.username.get().is_some(){
                            // Haven't fetched yet — show both options
                            view! {
                                <div class="tab-panel">
                                    <div class="ghost-mascot">"🐙"</div>
                                    <h3>"GitHub Connected!"</h3>
                                    <p>"Click below to fetch your developer metrics"</p>
                                    <button class="btn-primary" on:click=fetch_dev disabled=move || loading.get()>
                                        {move || if loading.get() { "⏳ FETCHING..." } else { "📊 GET DEV STATS" }}
                                    </button>
                                </div>
                            }.into_any()
                        } else {
                            view! {
                                <div class="tab-panel">
                                    <div class="ghost-mascot">"🐙"</div>
                                    <h3>"GitHub Not Connected"</h3>
                                    <p>"Click 'Authorize GitHub' in the navigation bar to connect your account"</p>
                                </div>
                            }.into_any()
                        }
                    },
                    "repo" => if let Some(metrics) = repo_metrics.get() {
                            view! {
                                <div class="tab-panel">
                                    <h3>"Repo Stats"</h3>
                                    <div class="metrics-grid">
                                        <div class="metric-card">
                                            <span class="metric-value">{metrics.stars.to_string()}</span>
                                            <span class="metric-label">"STARS"</span>
                                        </div>
                                        <div class="metric-card">
                                            <span class="metric-value">{metrics.commits.to_string()}</span>
                                            <span class="metric-label">"COMMITS"</span>
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
                                <div class="tab-panel">
                                    <h3>"Verify Repository"</h3>
                                    <p>"Enter your repository name (e.g. GhostCheck)"</p>
                                    <div class="repo-input-group">
                                        <input
                                            type="text"
                                            placeholder="repo-name"
                                            class="repo-input"
                                            on:input=move |ev| {
                                                set_repo_input.set(event_target_value(&ev));
                                            }
                                        />
                                        <button
                                            class="btn-verify"
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

                    "badges" => view! {
                        <div class="tab-panel">
                            <p class="empty-state">"No badges minted yet. Start by getting your Dev Badge!"</p>
                        </div>
                    }.into_any(),
                    _ => view! { <p>"Unknown Tab"</p>}.into_any()
                }}

                // Show errors
                {
                    move || {
                        error.get().map(|e| view! {
                            <p class="error-msg">{e}</p>
                        })
                    }
                }

            </div>
        </section>
    }
}
