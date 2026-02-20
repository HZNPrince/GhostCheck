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
pub fn Profile() -> impl IntoView {
    let github = expect_context::<GithubState>();

    // Dev data
    let (dev_metrics, set_dev_metrics) = signal(Option::<DevMetrics>::None);
    let (loading, set_loading) = signal(false);
    let (error, set_error) = signal(Option::<String>::None);

    // Minted repos list (track what user has minted)
    let (minted_repos, set_minted_repos) = signal(Vec::<RepoMetrics>::new());

    // Update signals
    let (updating, set_updating) = signal(false);
    let (update_result, set_update_result) = signal(Option::<String>::None);
    let (update_error, set_update_error) = signal(Option::<String>::None);

    // Vouch signals
    let (vouch_target, set_vouch_target) = signal(String::new());
    let (vouching, set_vouching) = signal(false);
    let (vouch_result, set_vouch_result) = signal(Option::<String>::None);
    let (vouch_error, set_vouch_error) = signal(Option::<String>::None);

    // Fetch dev metrics on load
    let fetch_dev = move |_| {
        set_loading.set(true);
        set_error.set(None);
        spawn_local(async move {
            match api::fetch_github_metrics().await {
                Ok(m) => set_dev_metrics.set(Some(m)),
                Err(e) => set_error.set(Some(e)),
            }
            set_loading.set(false);
        });
    };

    // Update dev badge
    let update_dev = move |_| {
        let metrics = dev_metrics.get();
        if metrics.is_none() {
            return;
        }
        let m = metrics.unwrap();
        set_updating.set(true);
        set_update_error.set(None);
        set_update_result.set(None);

        spawn_local(async move {
            match solana::build_and_send_update_dev_badge_tx(
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
                    let tx = sig.as_string().unwrap_or("unknown".into());
                    set_update_result.set(Some(tx));
                }
                Err(e) => set_update_error.set(Some(format!("{:?}", e))),
            }
            set_updating.set(false);
        });
    };

    // Vouch for dev
    let vouch = move |_| {
        let target = vouch_target.get();
        if target.is_empty() {
            set_vouch_error.set(Some("Enter a wallet address".into()));
            return;
        }
        set_vouching.set(true);
        set_vouch_error.set(None);
        set_vouch_result.set(None);

        spawn_local(async move {
            // Convert base58 string to bytes
            // For now we'll pass the raw string bytes — you can improve this later
            let addr_bytes = bs58_decode(&target);
            match addr_bytes {
                None => set_vouch_error.set(Some("Invalid wallet address".into())),
                Some(bytes) => match solana::build_and_send_vouch_tx(bytes).await {
                    Ok(sig) => {
                        let tx = sig.as_string().unwrap_or("unknown".into());
                        set_vouch_result.set(Some(tx));
                    }
                    Err(e) => set_vouch_error.set(Some(format!("{:?}", e))),
                },
            }
            set_vouching.set(false);
        });
    };

    // Helper: get level name
    fn level_name(lvl: u8) -> &'static str {
        match lvl {
            1 => "👻 Ghost",
            2 => "💻 Coder",
            3 => "🔨 Builder",
            4 => "🏛️ Architect",
            5 => "🏆 Legend",
            _ => "❓ Unknown",
        }
    }

    view! {
        <section class="dashboard">
            <h2 class="dashboard-title">"Profile"</h2>

            // ══════ DEV BADGE CARD ══════
            {move || {
                if github.username.get().is_none() {
                    return view! {
                        <div class="tab-panel">
                            <div class="ghost-mascot">"👻"</div>
                            <h3>"Connect GitHub to view your profile"</h3>
                        </div>
                    }.into_any();
                }

                if let Some(m) = dev_metrics.get() {
                    view! {
                        <div class="badge-card">
                            <div class="badge-header">
                                <h3>"🏅 Developer Badge"</h3>
                                <span class="level-badge">{level_name(m.reputation_level)}</span>
                            </div>
                            <div class="badge-stats">
                                <div class="stat-row">
                                    <span class="stat-item">"📦 "{m.repo_count.to_string()}" Repos"</span>
                                    <span class="stat-item">"⭐ "{m.total_stars.to_string()}" Stars"</span>
                                </div>
                                <div class="stat-row">
                                    <span class="stat-item">"📝 "{m.total_commit.to_string()}" Commits"</span>
                                    <span class="stat-item">"🔀 "{m.prs_merged.to_string()}" PRs Merged"</span>
                                </div>
                                <div class="stat-row">
                                    <span class="stat-item">"🐛 "{m.issues_closed.to_string()}" Issues Closed"</span>
                                    <span class="stat-item">"👥 "{m.followers.to_string()}" Followers"</span>
                                </div>
                                <div class="stat-row">
                                    <span class="stat-item">"📅 "{m.account_age_days.to_string()}" Days Active"</span>
                                    <span class="stat-item">"🗂️ "{m.owned_repo_count.to_string()}" Owned Repos"</span>
                                </div>
                            </div>
                            <button class="btn-update"
                                on:click=update_dev
                                disabled=move || updating.get()
                            >
                                {move || if updating.get() { "⏳ UPDATING..." } else { "🔄 UPDATE DEV BADGE" }}
                            </button>
                            {move || update_result.get().map(|sig| view! {
                                <p class="success-msg">"✅ Updated! Tx: "{sig}</p>
                            })}
                            {move || update_error.get().map(|e| view! {
                                <p class="error-msg">"❌ "{e}</p>
                            })}
                        </div>
                    }.into_any()
                } else {
                    view! {
                        <div class="tab-panel">
                            <button class="btn-primary" on:click=fetch_dev disabled=move || loading.get()>
                                {move || if loading.get() { "⏳ LOADING..." } else { "📊 LOAD PROFILE" }}
                            </button>
                        </div>
                    }.into_any()
                }
            }}

            // ══════ VOUCH SECTION ══════
            {move || {
                if github.username.get().is_none() {
                    return view! { <div></div> }.into_any();
                }
                view! {
                    <div class="tab-panel" style="margin-top: 2rem;">
                        <h3>"🤝 Vouch for a Developer"</h3>
                        <p>"Enter their SOL wallet address to vouch"</p>
                        <div class="repo-input-group">
                            <input
                                type="text"
                                placeholder="Wallet address (base58)"
                                class="repo-input"
                                on:input=move |ev| set_vouch_target.set(event_target_value(&ev))
                            />
                            <button class="btn-verify"
                                on:click=vouch
                                disabled=move || vouching.get()
                            >
                                {move || if vouching.get() { "⏳..." } else { "VOUCH" }}
                            </button>
                        </div>
                        {move || vouch_result.get().map(|sig| view! {
                            <p class="success-msg">"✅ Vouched! Tx: "{sig}</p>
                        })}
                        {move || vouch_error.get().map(|e| view! {
                            <p class="error-msg">"❌ "{e}</p>
                        })}
                    </div>
                }.into_any()
            }}

            {move || error.get().map(|e| view! { <p class="error-msg">{e}</p> })}
        </section>
    }
}

// Simple base58 decoder for wallet addresses
fn bs58_decode(input: &str) -> Option<Vec<u8>> {
    const ALPHABET: &[u8] = b"123456789ABCDEFGHJKLMNPQRSTUVWXYZabcdefghijkmnopqrstuvwxyz";
    let mut result = vec![0u8; 32];
    let mut scratch = Vec::new();

    for c in input.bytes() {
        let mut carry = match ALPHABET.iter().position(|&x| x == c) {
            Some(idx) => idx as u32,
            None => return None,
        };
        for byte in scratch.iter_mut() {
            carry += 58 * (*byte as u32);
            *byte = (carry & 0xff) as u8;
            carry >>= 8;
        }
        while carry > 0 {
            scratch.push((carry & 0xff) as u8);
            carry >>= 8;
        }
    }

    // Add leading zeros
    for c in input.bytes() {
        if c == b'1' {
            scratch.push(0);
        } else {
            break;
        }
    }

    scratch.reverse();
    if scratch.len() != 32 {
        return None;
    }
    Some(scratch)
}
