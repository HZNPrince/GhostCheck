use leptos::prelude::*;
use leptos_router::{components::*, path};

use crate::components::{footer::Footer, navbar::Navbar};
use crate::pages::{connect::Connect, dashboard::Dashboard, landing::Landing};

#[component]
pub fn App() -> impl IntoView {
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
