use leptos::prelude::*;
use leptos_router::{components::A, hooks::use_query_map};
use wasm_bindgen_futures::spawn_local;

use crate::services::api::{self, DevMetrics};

#[component]
pub fn Dashboard() -> impl IntoView {
    let (active_tab, set_active_tab) = signal(String::from("dev"));

    // Read session_id from the url query
    let query = use_query_map();
    let session_id = move || query.read().get("session_id").unwrap_or_default();

    // Store fetched metrics (set signals)
    let (dev_metrics, set_dev_metrics) = signal(Option::<DevMetrics>::None);
    let (loading, set_loading) = signal(false);
    let (error, set_error) = signal(Option::<String>::None);

    // Fetch Dev Metrics when the button is clicked
    let fetch_dev = move |_| {
        let sid = session_id();
        if sid.is_empty() {
            set_error.set(Some("No Session: Authorize your github first".to_string()));
            return;
        }
        set_loading.set(true);
        set_error.set(None);

        spawn_local(async move {
            match api::fetch_github_metrics(&sid).await {
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
                        // Check for session id
                        let has_session = !session_id().is_empty();
                        if let Some(metrics) = dev_metrics.get() {
                            view! {
                                <div class="tap-panel">
                                    <h3>"Your Dev Stats"</h3>
                                    <div class="metrics-grid">
                                        <div class="metric-card">
                                            <span class="metrics-value">{metrics.repo_count.to_string()}</span>
                                            <span class="metrics-label">"REPOS"</span>
                                        </div>
                                        <div class="metric-card">
                                            <span class="metrics-value">{metrics.total_commit.to_string()}</span>
                                            <span class="metrics-label">"COMMITS"</span>
                                        </div>
                                    </div>
                                    <button class="btn-primary">"🏆 Mint DEV_BADGE"</button>
                                    <p class="hint-text">"Signature ready — click to mint your badge onchain"</p>
                                </div>
                            }.into_any()
                        } else if has_session {
                            // has session but havent fetched yet
                            view! {
                                <div class="tab-panel">
                                    <div class="ghost-mascot">"🐙"</div>
                                    <h3>"Github CONNECTED!"</h3>
                                    <p>"Click below to fetch your developer metrics"</p>
                                    <button class="btn-primary" on:click=fetch_dev disabled = move || loading.get() >{move || if loading.get() {"⏳ FETCHING ..."} else {"📊 GET DEV STATS"}}</button>
                                </div>
                            }.into_any()
                        } else {
                            view! {
                                <div class="tab-panel">
                                    <div class="ghost-mascot">"🐙"</div>
                                    <h3>"Authorize GitHub"</h3>
                                    <p>"Connect your GitHub to generate your anonymous Dev Badge"</p>
                                    <a href="http://localhost:3000/auth/github">"🔑 AUTHORIZE GITHUB"</a>
                                </div>
                            }.into_any()
                        }
                    },
                    "repo" => view! {
                        <div class="tab-panel">
                            <h3>"Verify Repository"</h3>
                            <div class="repo-input-group">
                                <input type="text" placeholder="https://github.com/user/repo" class="repo-input" />
                                <button class="btn-verify">"Verify"</button>
                            </div>
                        </div>
                    }.into_any(),
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
