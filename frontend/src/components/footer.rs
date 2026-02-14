use leptos::prelude::*;
use leptos_router::components::A;

#[component]
pub fn Footer() -> impl IntoView {
    view! {
        <footer class="footer">
            <div class="pixel-bar"></div>
            <p>"© 2026 GHOSTCHECK PROTOCOL — ALL RIGHTS RESERVED"</p>
        </footer>
    }
}
