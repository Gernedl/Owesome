use base64::engine::general_purpose::URL_SAFE_NO_PAD;
use base64::Engine;
use hmac::{Hmac, KeyInit, Mac};
use sha2::Sha256;
use std::time::{SystemTime, UNIX_EPOCH};

type HmacSha256 = Hmac<Sha256>;

// Reads the key file that secrets-init generated (hex text) and turns it back into bytes.
pub fn secret() -> Vec<u8> {
    let path = std::env::var("HMAC_KEY_FILE").expect("HMAC_KEY_FILE is not set");
    let hex_text = std::fs::read_to_string(path).expect("cannot read HMAC key file");
    hex::decode(hex_text.trim()).expect("HMAC key file is not valid hex")
}

pub fn get_code() -> String {
    let start = SystemTime::now();
    let binding = start
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_millis()
        .to_string();
    let since_the_epoch = binding;
    let secret = secret();
    let mut key = HmacSha256::new_from_slice(&secret).unwrap();
    key.update(since_the_epoch.as_bytes());
    let res = key.finalize();
    URL_SAFE_NO_PAD.encode(
        format!(
            "{}.{}",
            since_the_epoch,
            URL_SAFE_NO_PAD.encode(res.into_bytes())
        )
        .into_bytes(),
    )
}

pub fn check_code(code: &str) -> bool {
    let Ok(inner) = URL_SAFE_NO_PAD.decode(code) else {
        return false;
    };
    let Ok(inner) = String::from_utf8(inner) else {
        return false;
    };
    let Some((timestamp, signature)) = inner.split_once('.') else {
        return false;
    };
    let Ok(signature) = URL_SAFE_NO_PAD.decode(signature) else {
        return false;
    };
    let secret = secret();
    let mut key = HmacSha256::new_from_slice(&secret).unwrap();
    key.update(timestamp.as_bytes());
    if key.verify_slice(&signature).is_ok() {
        let start = SystemTime::now();
        let binding = start.duration_since(UNIX_EPOCH).unwrap().as_millis();
        let Ok(timestamp) = u128::from_str_radix(timestamp, 10) else {
            return false;
        };
        let diff = binding.saturating_sub(timestamp);
        diff < 1000 * 60 * 15 // 15 minutes old max
    } else {
        false
    }
}
