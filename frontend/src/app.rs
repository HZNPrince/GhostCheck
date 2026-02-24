use leptos::prelude::*;
use leptos_router::{components::*, path};
use wasm_bindgen_futures::spawn_local;

use crate::components::{footer::Footer, navbar::Navbar};
use crate::pages::profile::Profile;
use crate::pages::search::Search;
use crate::pages::{connect::Connect, dashboard::Dashboard, landing::Landing};
use crate::services::api;

#[derive(Clone, Copy)]
pub struct WalletState {
    pub address: ReadSignal<Option<String>>,
    pub set_address: WriteSignal<Option<String>>,
}

#[derive(Clone, Copy)]
pub struct GithubState {
    pub username: ReadSignal<Option<String>>,
    pub set_username: WriteSignal<Option<String>>,
}

#[component]
pub fn App() -> impl IntoView {
    // Create wallet signal
    let (address, set_address) = signal(Option::<String>::None);

    // Provide to context
    provide_context(WalletState {
        address,
        set_address,
    });

    // Create username state
    let (gh_username, set_gh_username) = signal(Option::<String>::None);

    provide_context(GithubState {
        username: gh_username,
        set_username: set_gh_username,
    });

    // Auto-check GitHub auth on mount
    spawn_local(async move {
        if let Ok(status) = api::fetch_auth_status().await {
            if status.authenticated {
                set_gh_username.set(status.username);
            }
        }
    });

    // Auto-reconnect wallet on mount (silent, no popup)
    spawn_local(async move {
        match crate::services::wallet::try_reconnect_phantom().await {
            Ok(pubkey) => {
                log::info!("Wallet auto-reconnected: {}", pubkey);
                set_address.set(Some(pubkey));
            }
            Err(_) => {} // Not previously connected, ignore
        }
    });

    view! {
        <Router>
            <Navbar />
                <main>
                    <Routes fallback= || view! {<p>Page Not Found</p>}>
                        <Route path=path!("/") view=Landing />
                        <Route path=path!("/connect") view=Connect/>
                        <Route path=path!("/dashboard") view=Dashboard/>
                        <Route path=path!("/profile") view=Profile/>
                        <Route path=path!("/search") view=Search/>
                    </Routes>
                </main>
            <Footer />
        </Router>
    }
}
