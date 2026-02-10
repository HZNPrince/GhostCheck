use once_cell::sync::Lazy;
use rand::{Rng, distr::Alphanumeric};
use std::{collections::HashMap, sync::Mutex};

static SESSIONS: Lazy<Mutex<HashMap<String, String>>> = Lazy::new(|| Mutex::new(HashMap::new()));

pub fn create_session(token_access: String) -> String {
    let session_id: String = rand::rng()
        .sample_iter(&Alphanumeric)
        .take(16)
        .map(char::from)
        .collect();

    SESSIONS
        .lock()
        .unwrap()
        .insert(session_id.clone(), token_access);

    session_id
}

pub fn get_token(session_id: String) -> Option<String> {
    SESSIONS.lock().unwrap().get(&session_id).cloned()
}
