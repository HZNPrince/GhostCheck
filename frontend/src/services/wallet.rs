use js_sys::Reflect;
use wasm_bindgen::prelude::*;
use wasm_bindgen_futures::JsFuture;

pub async fn connect_phantom() -> Result<String, String> {
    // Get Browers window Object
    let window = web_sys::window().ok_or("No Window Object - are we in a browser?")?;

    // Access window.phantom()
    let phantom = Reflect::get(&window, &"phantom".into())
        .map_err(|_| "Phantom Wallet not found, is the extension installed?".to_string())?;

    // Access window.phantom.solana
    let solana = Reflect::get(&phantom, &"solana".into())
        .map_err(|_| "Phantom Solana provider not found".to_string())?;

    // Get the connect function
    let connect_fn: js_sys::Function = Reflect::get(&solana, &"connect".into())
        .map_err(|_| "connect method not found".to_string())?
        .dyn_into()
        .map_err(|_| "connect is not a function".to_string())?;

    // Call solana.connect() function : Returns a js promise
    let promise: js_sys::Promise = connect_fn
        .call0(&solana)
        .map_err(|_| "failed to connect".to_string())?
        .dyn_into()
        .map_err(|_| "connect() didn't return a Promise".to_string())?;

    // await the jspromise to get the result
    let result = JsFuture::from(promise)
        .await
        .map_err(|_| "User rejected the connection".to_string())?;

    // Get publicKey from response
    let public_key = Reflect::get(&result, &"publicKey".into())
        .map_err(|_| "No publicKey in response".to_string())?;

    // get the publicKey.toString() function
    let to_string_fn: js_sys::Function = Reflect::get(&public_key, &"toString".into())
        .map_err(|_| "toString not found on publicKey".to_string())?
        .dyn_into()
        .map_err(|_| "toString is not a function".to_string())?;

    // call the function
    let pubkey_str = to_string_fn
        .call0(&public_key)
        .map_err(|_| "Failed to call toString()".to_string())?;

    Ok(pubkey_str.as_string().unwrap_or_default())
}

pub async fn disconnect_phantom() -> Result<(), String> {
    let window = web_sys::window().ok_or("No Window")?;
    let phantom =
        Reflect::get(&window, &"phantom".into()).map_err(|_| "Phantom not found".to_string())?;
    let solana = Reflect::get(&phantom, &"solana".into())
        .map_err(|_| "Solana Provider not found".to_string())?;

    let disconnect_fn: js_sys::Function = Reflect::get(&solana, &"disconnect".into())
        .map_err(|_| "Disconnect function not found".to_string())?
        .dyn_into()
        .map_err(|_| "disconnect is not a function".to_string())?;

    let promise: js_sys::Promise = disconnect_fn
        .call0(&solana)
        .map_err(|_| "Failed to call disconnect".to_string())?
        .dyn_into()
        .map_err(|_| "Did not return promise")?;

    JsFuture::from(promise)
        .await
        .map_err(|_| "Disconnect Failed".to_string())?;

    Ok(())
}
