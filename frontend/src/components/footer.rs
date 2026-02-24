use leptos::prelude::*;

#[component]
pub fn Footer() -> impl IntoView {
    view! {
        <footer class="footer">
            <div class="pixel-bar"></div>
            <div class="footer-content">
                <p>"© 2026 GHOSTCHECK PROTOCOL"</p>
                <div class="footer-links">
                    <a href="https://github.com/HZNPrince/GhostCheck" target="_blank" class="footer-link">"Source Code"</a>
                    <span class="footer-sep">"·"</span>
                    <a href="https://x.com/YunoWiz" target="_blank" class="footer-link">"@YunoWiz"</a>
                    <span class="footer-sep">"·"</span>
                    <a href="https://github.com/HZNPrince/GhostCheck" target="_blank" class="footer-link">"Contribute"</a>
                </div>
            </div>
        </footer>
    }
}
