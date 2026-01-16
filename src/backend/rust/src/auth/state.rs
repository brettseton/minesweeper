use crate::error::{AppError, AppResult};
use crate::settings::Settings;
use base64::engine::general_purpose::URL_SAFE_NO_PAD;
use base64::Engine;
use chacha20poly1305::aead::{Aead, KeyInit};
use chacha20poly1305::{ChaCha20Poly1305, Nonce};
use serde::{Deserialize, Serialize};
use sha2::Digest;
use std::time::{SystemTime, UNIX_EPOCH};
use rand::RngCore;

const STATE_TTL_SECS: i64 = 10 * 60;
const STATE_VERSION: u8 = 1;

#[derive(Debug, Serialize, Deserialize)]
struct OAuthState {
    v: u8,
    exp_unix: i64,
    nonce: String,
    pkce_verifier: String,
}

fn now_unix() -> AppResult<i64> {
    let secs = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(|e| AppError::Internal(e.to_string()))?
        .as_secs();
    Ok(secs as i64)
}

fn derive_key(settings: &Settings) -> [u8; 32] {
    // Derive a dedicated key from the session secret; avoids key reuse across concerns.
    // SESSION_SECRET_KEY is already required to be >= 64 chars.
    let mut hasher = sha2::Sha256::new();
    hasher.update(settings.server.session_secret_key.as_bytes());
    hasher.update(b"minesweeper.oauth_state.v1");
    let out = hasher.finalize();
    let mut key = [0u8; 32];
    key.copy_from_slice(&out[..32]);
    key
}

fn aead(settings: &Settings) -> ChaCha20Poly1305 {
    let key = derive_key(settings);
    ChaCha20Poly1305::new((&key).into())
}

pub fn build_state(settings: &Settings, nonce: &str, pkce_verifier: &str) -> AppResult<String> {
    let exp_unix = now_unix()? + STATE_TTL_SECS;
    let payload = OAuthState {
        v: STATE_VERSION,
        exp_unix,
        nonce: nonce.to_string(),
        pkce_verifier: pkce_verifier.to_string(),
    };

    let plaintext =
        serde_json::to_vec(&payload).map_err(|e| AppError::Internal(e.to_string()))?;

    let mut nonce_bytes = [0u8; 12];
    rand::rngs::OsRng.fill_bytes(&mut nonce_bytes);
    let nonce = Nonce::from_slice(&nonce_bytes);

    let ciphertext = aead(settings)
        .encrypt(nonce, plaintext.as_ref())
        .map_err(|_| AppError::Internal("Failed to encrypt OAuth state".to_string()))?;

    // Token format: base64url( nonce(12) || ciphertext )
    let mut token = Vec::with_capacity(12 + ciphertext.len());
    token.extend_from_slice(&nonce_bytes);
    token.extend_from_slice(&ciphertext);
    Ok(URL_SAFE_NO_PAD.encode(token))
}

pub fn parse_state(settings: &Settings, token: &str) -> AppResult<(String, String)> {
    let bytes = URL_SAFE_NO_PAD
        .decode(token)
        .map_err(|_| AppError::BadRequest("Invalid OAuth state".to_string()))?;
    if bytes.len() < 12 {
        return Err(AppError::BadRequest("Invalid OAuth state".to_string()));
    }
    let (nonce_bytes, ciphertext) = bytes.split_at(12);
    let nonce = Nonce::from_slice(nonce_bytes);

    let plaintext = aead(settings)
        .decrypt(nonce, ciphertext)
        .map_err(|_| AppError::BadRequest("Invalid OAuth state".to_string()))?;

    let state: OAuthState =
        serde_json::from_slice(&plaintext).map_err(|_| AppError::BadRequest("Invalid OAuth state".to_string()))?;

    if state.v != STATE_VERSION {
        return Err(AppError::BadRequest("Unsupported OAuth state version".to_string()));
    }

    let now = now_unix()?;
    if state.exp_unix < now {
        return Err(AppError::BadRequest("OAuth state expired".to_string()));
    }
    if state.exp_unix > now + STATE_TTL_SECS {
        // Reject abnormally long expiries (defense in depth against odd clocks/tampering).
        return Err(AppError::BadRequest("OAuth state invalid".to_string()));
    }

    Ok((state.nonce, state.pkce_verifier))
}
