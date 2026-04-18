//! LiveKit access token issuer. Stub until the real HMAC signing is wired up:
//! return an error rather than a blank success so callers can surface the
//! failure instead of handing out an unusable connect token.

use anyhow::{Result, bail};

pub fn issue_token(_user_id: &str, _room: &str) -> Result<String> {
    bail!("livekit token issuance is not implemented")
}

pub fn server_url(cfg_url: &str) -> String {
    cfg_url.to_string()
}
