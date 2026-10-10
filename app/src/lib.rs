pub mod app;
pub mod auth;
// Server-only: needs the HMAC key, so it must never end up in the WASM bundle.
#[cfg(feature = "ssr")]
pub mod registration;
#[cfg(feature = "ssr")]
pub mod session;

#[cfg(feature = "hydrate")]
#[wasm_bindgen::prelude::wasm_bindgen]
pub fn hydrate() {
    use crate::app::*;
    console_error_panic_hook::set_once();
    leptos::mount::hydrate_body(App);
}
