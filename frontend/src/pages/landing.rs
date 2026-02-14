use leptos::prelude::*;
use leptos_router::components::A;

#[component]
pub fn Landing() -> impl IntoView {
    view! {
        <section class="hero">
            <div class="hero-content">
                <div class="ghost-mascot">"👻"</div>
                <h1>
                    "Ghost"
                    <span class="highlight">"Check"</span>
                </h1>
                <p class="tagline">"Anonymous Onchain Reputation"</p>
                <p class="description">
                    "Prove your developer credentials without revealing your identity."
                    "Mint verifiable badges backed by your Github activity."
                </p>
                <A href="/connect" attr:class="btn-primary">
                    "Connect Wallet"
                </A>
            </div>

        </section>
    }
}
