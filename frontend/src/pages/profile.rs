use leptos::prelude::*;
use wasm_bindgen_futures::spawn_local;

use crate::{
    app::{GithubState, WalletState},
    services::{
        api::{self, OnChainDevBadge, OnchainRepoBadgeData},
        solana,
    },
};

#[component]
pub fn Profile() -> impl IntoView {
    let github = expect_context::<GithubState>();
    let wallet = expect_context::<WalletState>();

    // On-chain badge (determines if update is available)
    let (on_chain_badge, set_on_chain_badge) = signal(Option::<OnChainDevBadge>::None);
    let (badge_loading, set_badge_loading) = signal(true);

    // Minted repos from on-chain
    let (minted_repos, set_minted_repos) = signal(Vec::<OnchainRepoBadgeData>::new());
    let (repos_loading, set_repos_loading) = signal(false);

    // Vouch signals
    let (vouch_target, set_vouch_target) = signal(String::new());
    let (vouching, set_vouching) = signal(false);
    let (vouch_result, set_vouch_result) = signal(Option::<String>::None);
    let (vouch_error, set_vouch_error) = signal(Option::<String>::None);

    // Watch wallet — check on-chain badge + fetch minted repos
    Effect::new(move |_| {
        let addr = wallet.address.get();
        if let Some(wallet_str) = addr {
            let ws = wallet_str.clone();
            // Check on-chain badge
            spawn_local(async move {
                match api::check_dev_badge(&ws).await {
                    Ok(badge) => {
                        if badge.exists {
                            set_on_chain_badge.set(Some(badge));
                        }
                    }
                    Err(e) => log::warn!("Badge check failed: {}", e),
                }
                set_badge_loading.set(false);
            });
            // Fetch minted repos
            set_repos_loading.set(true);
            spawn_local(async move {
                match api::fetch_minted_repos(&wallet_str).await {
                    Ok(repos) => set_minted_repos.set(repos),
                    Err(e) => log::warn!("Failed to load minted repos: {}", e),
                }
                set_repos_loading.set(false);
            });
        } else {
            set_badge_loading.set(false);
        }
    });

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

    fn level_name(lvl: u8) -> &'static str {
        match lvl {
            1 => "Ghost",
            2 => "Coder",
            3 => "Builder",
            4 => "Architect",
            5 => "Legend",
            _ => "Unknown",
        }
    }

    fn level_icon(lvl: u8) -> &'static str {
        match lvl {
            1 => "👻",
            2 => "💻",
            3 => "🔨",
            4 => "🏛️",
            5 => "🏆",
            _ => "❓",
        }
    }

    view! {
        <section class="profile-page">
            // Header
            <div class="profile-header">
                <h2 class="profile-title">"Developer Profile"</h2>
                {move || github.username.get().map(|u| view! {
                    <span class="profile-username">"@"{u}</span>
                })}
            </div>

            // Dev Badge Card
            {move || {
                if github.username.get().is_none() {
                    return view! {
                        <div class="profile-empty">
                            <div class="ghost-mascot">"👻"</div>
                            <h3>"Connect GitHub to view your profile"</h3>
                            <p>"Your developer reputation starts here"</p>
                        </div>
                    }.into_any();
                }

                if badge_loading.get() {
                    return view! {
                        <div class="profile-empty">
                            <div class="ghost-mascot">"⏳"</div>
                            <h3>"Loading profile..."</h3>
                        </div>
                    }.into_any();
                }

                // If on-chain badge exists — show the premium dev card
                if let Some(badge) = on_chain_badge.get() {
                    view! {
                        <div class="dev-card">
                            <div class="dev-card-accent"></div>
                            <div class="dev-card-header">
                                <div class="dev-card-identity">
                                    <span class="dev-card-icon">{level_icon(badge.reputation_level)}</span>
                                    <div>
                                        <h3 class="dev-card-name">"Developer Badge"</h3>
                                        <span class="dev-card-level">{level_name(badge.reputation_level)}" · Lv."{badge.reputation_level.to_string()}</span>
                                    </div>
                                </div>
                                <span class="dev-card-badge">"ON-CHAIN"</span>
                            </div>
                            <div class="dev-card-stats">
                                <div class="dev-stat">
                                    <span class="dev-stat-val">{badge.repo_counts.to_string()}</span>
                                    <span class="dev-stat-lbl">"Repos"</span>
                                </div>
                                <div class="dev-stat">
                                    <span class="dev-stat-val">{badge.total_commits.to_string()}</span>
                                    <span class="dev-stat-lbl">"Commits"</span>
                                </div>
                                <div class="dev-stat">
                                    <span class="dev-stat-val">{badge.total_stars.to_string()}</span>
                                    <span class="dev-stat-lbl">"Stars"</span>
                                </div>
                                <div class="dev-stat">
                                    <span class="dev-stat-val">{badge.prs_merged.to_string()}</span>
                                    <span class="dev-stat-lbl">"PRs"</span>
                                </div>
                                <div class="dev-stat">
                                    <span class="dev-stat-val">{badge.issues_closed.to_string()}</span>
                                    <span class="dev-stat-lbl">"Issues"</span>
                                </div>
                                <div class="dev-stat">
                                    <span class="dev-stat-val">{badge.followers.to_string()}</span>
                                    <span class="dev-stat-lbl">"Followers"</span>
                                </div>
                                <div class="dev-stat">
                                    <span class="dev-stat-val">{badge.vouch_count.to_string()}</span>
                                    <span class="dev-stat-lbl">"Vouches"</span>
                                </div>
                                <div class="dev-stat">
                                    <span class="dev-stat-val">{badge.verified_repos.to_string()}</span>
                                    <span class="dev-stat-lbl">"Verified"</span>
                                </div>
                            </div>
                            <div class="dev-card-watermark">"GHOSTCHECK"</div>
                        </div>
                    }.into_any()
                } else {
                    // No on-chain badge — prompt to mint from dashboard
                    view! {
                        <div class="profile-empty">
                            <div class="ghost-mascot">"🎯"</div>
                            <h3>"No Dev Badge Yet"</h3>
                            <p>"Mint your developer badge from the Dashboard to see your on-chain stats here"</p>
                        </div>
                    }.into_any()
                }
            }}

            // Verified Repos section
            {move || {
                if github.username.get().is_none() {
                    return view! { <div></div> }.into_any();
                }

                let repos = minted_repos.get();

                if repos_loading.get() {
                    return view! {
                        <div class="profile-section">
                            <h3 class="section-title">"Verified Repositories"</h3>
                            <p class="section-sub">"Loading..."</p>
                        </div>
                    }.into_any();
                }

                if repos.is_empty() {
                    return view! {
                        <div class="profile-section">
                            <h3 class="section-title">"Verified Repositories"</h3>
                            <div class="profile-empty-small">
                                <p>"No repos verified yet"</p>
                                <p class="section-sub">"Mint repo badges from the Dashboard"</p>
                            </div>
                        </div>
                    }.into_any();
                }

                view! {
                    <div class="profile-section">
                        <h3 class="section-title">"Verified Repositories ("{repos.len().to_string()}")"</h3>
                        <div class="repos-grid">
                            {repos.into_iter().map(|repo| {
                                let repo_name = repo.repo_name.clone();
                                let repo_name_for_update = repo.repo_name.clone();
                                let (repo_updating, set_repo_updating) = signal(false);
                                let (repo_update_msg, set_repo_update_msg) = signal(Option::<String>::None);
                                let (repo_update_err, set_repo_update_err) = signal(Option::<String>::None);

                                let on_update_repo = move |_| {
                                    let name = repo_name_for_update.clone();
                                    set_repo_updating.set(true);
                                    set_repo_update_msg.set(None);
                                    set_repo_update_err.set(None);
                                    spawn_local(async move {
                                        match api::fetch_repo_metrics(&name).await {
                                            Ok(m) => {
                                                // Pad repo_name_bytes to 32 bytes for PDA
                                                let mut repo_name_padded = m.repo_name_bytes.clone();
                                                repo_name_padded.resize(32, 0);

                                                match solana::build_and_send_update_repo_badge_tx(
                                                    m.signature.clone(),
                                                    m.signed_message.clone(),
                                                    m.public_key_bytes.clone(),
                                                    repo_name_padded,
                                                    m.hashed_username.clone(),
                                                    m.stars,
                                                    m.commits,
                                                    m.fork_count,
                                                    m.issues_open_count,
                                                    m.lang1_bytes.clone(),
                                                    m.lang2_bytes.clone(),
                                                ).await {
                                                    Ok(sig) => {
                                                        let tx = sig.as_string().unwrap_or("unknown".into());
                                                        set_repo_update_msg.set(Some(format!("Updated! Tx: {}", tx)));
                                                    }
                                                    Err(e) => set_repo_update_err.set(Some(format!("{:?}", e))),
                                                }
                                            }
                                            Err(e) => set_repo_update_err.set(Some(e)),
                                        }
                                        set_repo_updating.set(false);
                                    });
                                };

                                view! {
                                    <div class="repo-badge-card">
                                        <div class="repo-badge-accent"></div>
                                        <div class="repo-badge-header">
                                            <div class="repo-badge-identity">
                                                <span class="repo-badge-icon">"📦"</span>
                                                <div>
                                                    <h3 class="repo-badge-name">{repo_name.clone()}</h3>
                                                    <div class="repo-badge-langs">
                                                        {if !repo.lang1.is_empty() {
                                                            Some(view! { <span class="lang-tag">{repo.lang1.clone()}</span> })
                                                        } else { None }}
                                                        {if !repo.lang2.is_empty() {
                                                            Some(view! { <span class="lang-tag">{repo.lang2.clone()}</span> })
                                                        } else { None }}
                                                    </div>
                                                </div>
                                            </div>
                                        </div>
                                        <div class="repo-badge-stats">
                                            <div class="dev-stat">
                                                <span class="dev-stat-val">{repo.stars.to_string()}</span>
                                                <span class="dev-stat-lbl">"Stars"</span>
                                            </div>
                                            <div class="dev-stat">
                                                <span class="dev-stat-val">{repo.commits.to_string()}</span>
                                                <span class="dev-stat-lbl">"Commits"</span>
                                            </div>
                                            <div class="dev-stat">
                                                <span class="dev-stat-val">{repo.forks.to_string()}</span>
                                                <span class="dev-stat-lbl">"Forks"</span>
                                            </div>
                                            <div class="dev-stat">
                                                <span class="dev-stat-val">{repo.open_issues.to_string()}</span>
                                                <span class="dev-stat-lbl">"Issues"</span>
                                            </div>
                                        </div>
                                        <div class="repo-badge-actions">
                                            <button class="btn-update-sm"
                                                on:click=on_update_repo
                                                disabled=move || repo_updating.get()
                                            >
                                                {move || if repo_updating.get() { "UPDATING..." } else { "UPDATE REPO" }}
                                            </button>
                                        </div>
                                        {move || repo_update_msg.get().map(|msg| view! {
                                            <p class="success-msg">{msg}</p>
                                        })}
                                        {move || repo_update_err.get().map(|e| view! {
                                            <p class="error-msg">{e}</p>
                                        })}
                                    </div>
                                }
                            }).collect_view()}
                        </div>
                    </div>
                }.into_any()
            }}

            // Vouch section
            {move || {
                if github.username.get().is_none() {
                    return view! { <div></div> }.into_any();
                }
                view! {
                    <div class="profile-section">
                        <h3 class="section-title">"Vouch for a Developer"</h3>
                        <p class="section-sub">"Enter their SOL wallet address"</p>
                        <div class="vouch-input-row">
                            <input
                                type="text"
                                placeholder="Wallet address (base58)"
                                class="vouch-input"
                                on:input=move |ev| set_vouch_target.set(event_target_value(&ev))
                            />
                            <button class="btn-vouch"
                                on:click=vouch
                                disabled=move || vouching.get()
                            >
                                {move || if vouching.get() { "..." } else { "VOUCH" }}
                            </button>
                        </div>
                        {move || vouch_result.get().map(|sig| view! {
                            <p class="success-msg">"Vouched! Tx: "{sig}</p>
                        })}
                        {move || vouch_error.get().map(|e| view! {
                            <p class="error-msg">{e}</p>
                        })}
                    </div>
                }.into_any()
            }}


        </section>
    }
}

// Simple base58 decoder for wallet addresses
fn bs58_decode(input: &str) -> Option<Vec<u8>> {
    const ALPHABET: &[u8] = b"123456789ABCDEFGHJKLMNPQRSTUVWXYZabcdefghijkmnopqrstuvwxyz";
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
