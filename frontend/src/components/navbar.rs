use leptos::prelude::*;
use leptos_router::components::A;
use wasm_bindgen_futures::spawn_local;

use crate::{app::WalletState, services::wallet};

#[component]
pub fn Navbar() -> impl IntoView {
    // Grab global wallet state
    let user_wallet = expect_context::<WalletState>();

    let on_wallet_click = move |_| {
        if user_wallet.address.get().is_some() {
            // Already connected → disconnect
            spawn_local(async move {
                if let Ok(()) = wallet::disconnect_phantom().await {
                    user_wallet.set_address.set(None);
                    log::info!("Disconnected from navbar");
                }
            });
        } else {
            // Not connected → connect
            spawn_local(async move {
                match wallet::connect_phantom().await {
                    Ok(pubkey) => {
                        log::info!("Connected! {}", pubkey);
                        user_wallet.set_address.set(Some(pubkey));
                    }
                    Err(e) => log::error!("Failed: {}", e),
                }
            });
        }
    };

    view! {
        <nav class="navbar">
            <A href="/" attr:class="nav-logo">
                <span class="logo-icon">"👾"</span>
                <span class="logo-text">"Ghost"<span class="highlight">"Check"</span></span>
            </A>
            <A href="/connect" attr:class="nav-connect-btn" on:click= on_wallet_click>{
                move || match user_wallet.address.get() {
                    Some(addr) => format!("{}...{}", &addr[..4], &addr[addr.len()-4 ..]),
                    None => "CONNECT WALLET".to_string(),
                }
            }</A>
        </nav>
    }
}
