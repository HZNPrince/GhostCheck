use leptos::prelude::*;
use leptos_router::components::A;

#[component]
pub fn Navbar() -> impl IntoView {
    view! {
        <nav class="navbar">
            <A href="/" attr:class="nav-logo">
                <span class="logo-icon">"👾"</span>
                <span class="logo-text">"Ghost"<span class="highlight">"Check"</span></span>
            </A>
            <A href="/connect" attr:class="nav-connect-btn">"Connect Wallet"</A>
        </nav>
    }
}
