use leptos::prelude::*;
use leptos_router::components::A;

#[component]
pub fn Dashboard() -> impl IntoView {
    let (active_tab, set_active_tab) = signal(String::from("dev"));

    view! {
        <section class="dashboard">
            <h2 class="dashboard-title">"Dashboard"</h2>
            <div class="tabs">
                <button class=move || if active_tab.get() == "dev" {"tab_active"} else {"tab"} on:click=move |_| set_active_tab.set(String::from("dev"))>"Dev Badge"</button>
                <button class=move || if active_tab.get() == "repo" {"tab_active"} else {"tab"} on:click=move |_| set_active_tab.set(String::from("repo"))>"Repo Badge"</button>
                <button class=move || if active_tab.get() == "badges" {"tab_active"} else {"tab"} on:click=move |_| set_active_tab.set(String::from("badges"))>"My Badges"</button>
            </div>

            <div class="tab-content">
                {move|| match active_tab.get().as_str() {
                    "dev" => view! {
                        <div class="tab-panel">
                            <div class="ghost-mascot">"🐙"</div>
                            <h3>"Authorize Github"</h3>
                            <p>"Connect your Github to generate your anonymous Dev Badge"</p>
                            <button class="btn-primary">"🔑 Authorize Github"</button>
                        </div>
                    }.into_any(),
                    "repo" => view! {
                        <div class="tab-panel">
                            <h3>"Verify Repository"</h3>
                            <div class="repo-input-group">
                                <input type="text" placeholder="https://github.com/user/repo" class="repo-input" />
                                <button class="btn-verify">"Verify"</button>
                            </div>
                        </div>
                    }.into_any(),
                    "badges" => view! {
                        <div class="tab-panel">
                            <p class="empty-state">"No badges minted yet. Start by getting your Dev Badge!"</p>
                        </div>
                    }.into_any(),
                    _ => view! { <p>"Unknown Tab"</p>}.into_any()
                }}
            </div>
        </section>
    }
}
