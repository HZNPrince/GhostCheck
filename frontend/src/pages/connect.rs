use leptos::prelude::*;
use leptos_router::components::A;

#[component]
pub fn Connect() -> impl IntoView {
    view! {
        <section class="connect-page">
            <div class="connect-content">
                <div class="ghost-mascot">"👻"</div>
                <h2>"Connect Your Wallet"</h2>
                <p class="connect-subtitle">"Choose a wallet to begin your anonymous reputation journey"</p>
                <div class="wallet-grid">
                    <button class="wallet-option">"Phantom"</button>
                    <button class="wallet-option">"Solflare"</button>
                </div>
                <A href="/" attr:class="back-link">"← BACK TO HOME"</A>
            </div>
        </section>
    }
}
