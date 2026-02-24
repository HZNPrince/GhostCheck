use leptos::prelude::*;
use wasm_bindgen_futures::spawn_local;

use crate::{
    app::WalletState,
    services::{
        api::{self, OnChainDevBadge, OnchainRepoBadgeData},
        solana,
    },
};

#[component]
pub fn Search() -> impl IntoView {
    let wallet_state = expect_context::<WalletState>();

    let (query, set_query) = signal(String::new());
    let (searching, set_searching) = signal(false);
    let (search_error, set_search_error) = signal(Option::<String>::None);
    let (dev_badge, set_dev_badge) = signal(Option::<OnChainDevBadge>::None);
    let (repos, set_repos) = signal(Vec::<OnchainRepoBadgeData>::new());
    let (searched, set_searched) = signal(false);

    // Vouch signals
    let (vouching, set_vouching) = signal(false);
    let (vouch_result, set_vouch_result) = signal(Option::<String>::None);
    let (vouch_error, set_vouch_error) = signal(Option::<String>::None);

    let do_search = move || {
        let wallet = query.get().trim().to_string();
        if wallet.is_empty() {
            set_search_error.set(Some("Enter a wallet address".into()));
            return;
        }
        set_searching.set(true);
        set_search_error.set(None);
        set_dev_badge.set(None);
        set_repos.set(vec![]);
        set_searched.set(false);
        // Reset vouch state on new search
        set_vouch_result.set(None);
        set_vouch_error.set(None);

        spawn_local(async move {
            match api::search_dev_badge(&wallet).await {
                Ok(badge) => {
                    if badge.exists {
                        set_dev_badge.set(Some(badge));
                    }
                }
                Err(e) => log::warn!("Dev badge search: {}", e),
            }

            match api::search_repos(&wallet).await {
                Ok(r) => set_repos.set(r),
                Err(e) => log::warn!("Repo search: {}", e),
            }

            set_searched.set(true);
            set_searching.set(false);
        });
    };

    let do_vouch = move || {
        let target = query.get().trim().to_string();
        if target.is_empty() {
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

    let on_click_search = move |_: web_sys::MouseEvent| {
        do_search();
    };
    let on_key_search = move |ev: web_sys::KeyboardEvent| {
        if ev.key() == "Enter" {
            do_search();
        }
    };
    let on_click_vouch = move |_: web_sys::MouseEvent| {
        do_vouch();
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
        <section class="search-page">
            <h2 class="search-title">"Search Developer"</h2>
            <p class="search-sub">"Look up any wallet to view their on-chain reputation"</p>

            <div class="search-bar">
                <input
                    type="text"
                    placeholder="Enter SOL wallet address"
                    class="search-input"
                    prop:value=move || query.get()
                    on:input=move |ev| set_query.set(event_target_value(&ev))
                    on:keydown=on_key_search
                />
                <button
                    class="btn-primary"
                    on:click=on_click_search
                    disabled=move || searching.get()
                >
                    {move || if searching.get() { "SEARCHING..." } else { "SEARCH" }}
                </button>
            </div>

            {move || search_error.get().map(|e| view! {
                <p class="error-msg">{e}</p>
            })}

            // Results
            {move || {
                if searching.get() {
                    return view! {
                        <div class="search-loading">
                            <div class="ghost-mascot">"🔍"</div>
                            <p>"Searching on-chain data..."</p>
                        </div>
                    }.into_any();
                }

                if !searched.get() {
                    return view! { <div></div> }.into_any();
                }

                let badge = dev_badge.get();
                let found_repos = repos.get();

                if badge.is_none() && found_repos.is_empty() {
                    return view! {
                        <div class="search-empty">
                            <div class="ghost-mascot">"👻"</div>
                            <h3>"No On-Chain Data Found"</h3>
                            <p>"This wallet has no GhostCheck badges yet"</p>
                        </div>
                    }.into_any();
                }

                view! {
                    <div class="search-results">
                        // Dev Badge Card
                        {badge.map(|b| view! {
                            <div class="dev-card">
                                <div class="dev-card-accent"></div>
                                <div class="dev-card-header">
                                    <div class="dev-card-identity">
                                        <span class="dev-card-icon">{level_icon(b.reputation_level)}</span>
                                        <div>
                                            <h3 class="dev-card-name">"Developer Badge"</h3>
                                            <span class="dev-card-level">{level_name(b.reputation_level)}" · Lv."{b.reputation_level.to_string()}</span>
                                        </div>
                                    </div>
                                    <span class="dev-card-badge">"VERIFIED"</span>
                                </div>
                                <div class="dev-card-stats">
                                    <div class="dev-stat">
                                        <span class="dev-stat-val">{b.repo_counts.to_string()}</span>
                                        <span class="dev-stat-lbl">"Repos"</span>
                                    </div>
                                    <div class="dev-stat">
                                        <span class="dev-stat-val">{b.total_commits.to_string()}</span>
                                        <span class="dev-stat-lbl">"Commits"</span>
                                    </div>
                                    <div class="dev-stat">
                                        <span class="dev-stat-val">{b.total_stars.to_string()}</span>
                                        <span class="dev-stat-lbl">"Stars"</span>
                                    </div>
                                    <div class="dev-stat">
                                        <span class="dev-stat-val">{b.prs_merged.to_string()}</span>
                                        <span class="dev-stat-lbl">"PRs"</span>
                                    </div>
                                    <div class="dev-stat">
                                        <span class="dev-stat-val">{b.issues_closed.to_string()}</span>
                                        <span class="dev-stat-lbl">"Issues"</span>
                                    </div>
                                    <div class="dev-stat">
                                        <span class="dev-stat-val">{b.followers.to_string()}</span>
                                        <span class="dev-stat-lbl">"Followers"</span>
                                    </div>
                                    <div class="dev-stat">
                                        <span class="dev-stat-val">{b.vouch_count.to_string()}</span>
                                        <span class="dev-stat-lbl">"Vouches"</span>
                                    </div>
                                    <div class="dev-stat">
                                        <span class="dev-stat-val">{b.verified_repos.to_string()}</span>
                                        <span class="dev-stat-lbl">"Verified"</span>
                                    </div>
                                </div>
                                // Vouch action (only if wallet connected)
                                <div class="dev-card-actions">
                                    {move || {
                                        if wallet_state.address.get().is_some() {
                                            if let Some(ref tx) = vouch_result.get() {
                                                view! {
                                                    <p class="success-msg">"✅ Vouched! Tx: "{tx.clone()}</p>
                                                }.into_any()
                                            } else {
                                                view! {
                                                    <button
                                                        class="btn-vouch"
                                                        on:click=on_click_vouch
                                                        disabled=move || vouching.get()
                                                    >
                                                        {move || if vouching.get() { "VOUCHING..." } else { "🤝 VOUCH FOR THIS DEV" }}
                                                    </button>
                                                }.into_any()
                                            }
                                        } else {
                                            view! {
                                                <p class="search-sub" style="font-size: .7rem; padding: 0;">"Connect wallet to vouch"</p>
                                            }.into_any()
                                        }
                                    }}
                                    {move || vouch_error.get().map(|e| view! {
                                        <p class="error-msg">{e}</p>
                                    })}
                                </div>
                                <div class="dev-card-watermark">"GHOSTCHECK"</div>
                            </div>
                        })}

                        // Repo Cards
                        {if !found_repos.is_empty() {
                            Some(view! {
                                <div class="search-repos-section">
                                    <h3 class="section-title">"Verified Repositories ("{found_repos.len().to_string()}")"</h3>
                                    <div class="repos-grid">
                                        {found_repos.into_iter().map(|repo| view! {
                                            <div class="repo-badge-card">
                                                <div class="repo-badge-accent"></div>
                                                <div class="repo-badge-header">
                                                    <div class="repo-badge-identity">
                                                        <span class="repo-badge-icon">"📦"</span>
                                                        <div>
                                                            <h3 class="repo-badge-name">{repo.repo_name.clone()}</h3>
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
                                            </div>
                                        }).collect_view()}
                                    </div>
                                </div>
                            })
                        } else { None }}
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
