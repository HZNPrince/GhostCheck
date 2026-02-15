use leptos::prelude::*;
use leptos_router::components::A;
use wasm_bindgen_futures::spawn_local;

use crate::{app::WalletState, services::wallet};

#[component]
pub fn Connect() -> impl IntoView {
    let user_wallet = expect_context::<WalletState>();
    let (error_msg, set_error_msg) = signal(Option::<String>::None);
    let (connecting, set_connecting) = signal(false);

    // Runs when phantom button is clicked
    let on_phantom_click = move |_| {
        set_connecting.set(true);
        set_error_msg.set(None);

        spawn_local(async move {
            match wallet::connect_phantom().await {
                Ok(pubkey) => {
                    log::info!("Connected ! Pubkey: {}", pubkey);
                    user_wallet.set_address.set(Some(pubkey));
                }
                Err(e) => {
                    log::info!("Connection failed: {}", e);
                    set_error_msg.set(Some(e));
                }
            }
            set_connecting.set(false);
        });
    };

    let on_disconnect_click = move |_| {
        spawn_local(async move {
            match wallet::disconnect_phantom().await {
                Ok(()) => {
                    user_wallet.set_address.set(None);
                    log::info!("Disconnected.");
                }
                Err(e) => {
                    log::error!("Disconnect failed : {}", e);
                }
            }
        });
    };

    view! {
        <section class="connect-page">
            <div class="connect-content">
                <div class="ghost-mascot">"👻"</div>
                <h2>"Connect Your Wallet"</h2>
                <p class="connect_subtitle">"Choose a wallet to begin your anonymous reputation journey"</p>

                {
                    move || {
                        if let Some(addr) = user_wallet.address.get() {
                            view! {
                                <div class="wallet-connected">
                                    <p class="connected-label">"✅ CONNECTED"</p>
                                    <p class="wallet-addr">{format!("{}...{}", &addr[..4], &addr[addr.len()-4 ..])}</p>
                                    <A href="/dashboard" attr:class="btn-primary">"GO TO DASHBOARD →"</A>
                                    <button class="btn-disconnect" on:click=on_disconnect_click>"DISCONNECT"</button>

                                </div>
                            }.into_any()
                        } else {
                               view! {
                                   <div class="wallet-grid">
                                       <button
                                           class="wallet-option"
                                           on:click=on_phantom_click
                                           disabled=move || connecting.get()
                                       >
                                           {move || if connecting.get() { "CONNECTING..." } else { "👻 Phantom" }}
                                       </button>
                                       <button class="wallet-option" disabled=true>
                                           "☀️ Solflare (coming soon)"
                                       </button>
                                   </div>
                               }.into_any()
                           }

                    }

                }
                {move ||{
                    error_msg.get().map(|e| view! {
                        <p class="error-msg">{e}</p>
                    })
                }}

                 <A href="/" attr:class="back-link">"← BACK TO HOME"</A>
            </div>
        </section>
    }
}
