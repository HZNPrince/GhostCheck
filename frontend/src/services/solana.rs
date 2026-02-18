use wasm_bindgen::prelude::*;

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
        owned_repo_count: u32,
        total_stars: u32,
        total_commits: u32,
        prs_merged: u32,
        issues_closed: u32,
        followers: u32,
        account_age_days: u32,
        reputation_level: u8,
    ) -> Result<JsValue, JsValue>;

    // Calls window.buildAndSendRepoBadgeTx() defined in js/solana.js
    #[wasm_bindgen(js_name = buildAndSendRepoBadgeTx, catch)]
    pub async fn build_and_send_repo_badge_tx(
        signature: Vec<u8>,
        message: Vec<u8>,
        public_key: Vec<u8>,
        repo_name_padded: Vec<u8>,
        username_hashed: Vec<u8>,
        stars: u32,
        commits: u32,
        forks: u32,
        open_issues: u32,
        is_fork: u8,
        lang1: Vec<u8>,
        lang2: Vec<u8>,
    ) -> Result<JsValue, JsValue>;

    #[wasm_bindgen(js_name = buildAndSendUpdateDevBadgeTx, catch)]
    pub async fn build_and_send_update_dev_badge_tx(
        signature: Vec<u8>,
        message: Vec<u8>,
        public_key: Vec<u8>,
        username: Vec<u8>,
        repo_count: u32,
        owned_repo_count: u32,
        total_stars: u32,
        total_commits: u32,
        prs_merged: u32,
        issues_closed: u32,
        followers: u32,
        account_age_days: u32,
        reputation_level: u8,
    ) -> Result<JsValue, JsValue>;

    #[wasm_bindgen(js_name = buildAndSendUpdateRepoBadgeTx, catch)]
    pub async fn build_and_send_update_repo_badge_tx(
        signature: Vec<u8>,
        message: Vec<u8>,
        public_key: Vec<u8>,
        repo_name_padded: Vec<u8>,
        username_hashed: Vec<u8>,
        stars: u32,
        commits: u32,
        forks: u32,
        open_issues: u32,
        lang1: Vec<u8>,
        lang2: Vec<u8>,
    ) -> Result<JsValue, JsValue>;

    #[wasm_bindgen(js_name = buildAndSendVouchTx, catch)]
    pub async fn build_and_send_vouch_tx(target_addr: Vec<u8>) -> Result<JsValue, JsValue>;
}
