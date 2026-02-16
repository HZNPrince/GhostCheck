use wasm_bindgen::prelude::*;

// These bindings tell Rust: "there are JS functions with these names available globally.
// Let me call them from Rust and they return a Promise (async)."

#[wasm_bindgen]
extern "C" {
    // Calls window.buildAndSendDevBadgeTx() defined in js/solana.js
    #[wasm_bindgen(js_name = buildAndSendDevBadgeTx, catch)]
    pub async fn build_and_send_dev_badge_tx(
        signature: Vec<u8>,
        message: Vec<u8>,
        public_key: Vec<u8>,
        username: Vec<u8>,
        repo_count: u32,
        total_commits: u32,
    ) -> Result<JsValue, JsValue>;

    // Calls window.buildAndSendRepoBadgeTx() defined in js/solana.js
    #[wasm_bindgen(js_name = buildAndSendRepoBadgeTx, catch)]
    pub async fn build_and_send_repo_badge_tx(
        signature: Vec<u8>,
        message: Vec<u8>,
        public_key: Vec<u8>,
        repo_name_padded: Vec<u8>,
        username_padded: Vec<u8>,
        stars: u32,
        commits: u32,
        lang1: Vec<u8>,
        lang2: Vec<u8>,
    ) -> Result<JsValue, JsValue>;
}
