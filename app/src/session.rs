use base64::{engine::general_purpose::URL_SAFE_NO_PAD, Engine};
use hmac::{Hmac, KeyInit, Mac};
use sha2::Sha256;

use crate::registration::secret;

type HmacSha256 = Hmac<Sha256>;

pub fn get_id(value: &str) -> Option<i64> {
    let inner = URL_SAFE_NO_PAD.decode(value);
    let Ok(inner) = inner else {
        return None;
    };
    let Ok(inner) = String::from_utf8(inner) else {
        return None;
    };
    let tmp = inner.split_once('.');
    let Some((payload, signature)) = tmp else {
        return None;
    };
    let signature = URL_SAFE_NO_PAD.decode(signature).ok()?;
    let secret = secret();
    let mut key = HmacSha256::new_from_slice(&secret).unwrap();
    key.update(payload.as_bytes());
    if key.verify_slice(&signature).is_ok() {
        let payload = URL_SAFE_NO_PAD.decode(payload).ok()?;
        let payload = String::from_utf8(payload).ok()?;
        let (id, _username) = payload.split_once('.')?;
        i64::from_str_radix(id, 10).ok()
    } else {
        None
    }
}

pub fn get_username(value: &str) -> Option<String> {
    let inner = URL_SAFE_NO_PAD.decode(value).ok()?;
    let inner = String::from_utf8(inner).ok()?;
    let (payload, signature) = inner.split_once('.')?;
    let signature = URL_SAFE_NO_PAD.decode(signature).ok()?;
    let secret = secret();
    let mut key = HmacSha256::new_from_slice(&secret).unwrap();
    key.update(payload.as_bytes());
    if key.verify_slice(&signature).is_ok() {
        let payload = URL_SAFE_NO_PAD.decode(payload).ok()?;
        let payload = String::from_utf8(payload).ok()?;
        let (_id, username) = payload.split_once('.')?;
        Some(username.to_string())
    } else {
        None
    }
}

pub fn sign(id: i64, username: &str) -> String {
    let payload = URL_SAFE_NO_PAD.encode(format!("{}.{}", id.to_string(), username).into_bytes());
    let secret = secret();
    let mut key = HmacSha256::new_from_slice(&secret).unwrap();
    key.update(payload.as_bytes());
    let res = key.finalize();
    URL_SAFE_NO_PAD
        .encode(format!("{}.{}", payload, URL_SAFE_NO_PAD.encode(res.into_bytes())).into_bytes())
}
