use leptos::prelude::*;
use leptos_router::{components::*, path};

use crate::components::{footer::Footer, navbar::Navbar};
use crate::pages::{connect::Connect, dashboard::Dashboard, landing::Landing};

#[derive(Clone, Copy)]
pub struct WalletState {
    pub address: ReadSignal<Option<String>>,
    pub set_address: WriteSignal<Option<String>>,
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

    view! {
        <Router>
            <Navbar />
                <main>
                    <Routes fallback= || view! {<p>Page Not Found</p>}>
                        <Route path=path!("/") view=Landing />
                        <Route path=path!("/connect") view=Connect/>
                        <Route path=path!("/dashboard") view=Dashboard/>
                    </Routes>
                </main>
            <Footer />
        </Router>
    }
}
