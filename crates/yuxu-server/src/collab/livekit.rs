//! Stub — returns a random placeholder. Real impl would sign a LiveKit JWT
//! (identity, room, metadata, grants) with the configured shared secret.

pub fn issue_token(_user_id: &str, _room: &str) -> String {
    String::new()
}

pub fn server_url(cfg_url: &str) -> String {
    cfg_url.to_string()
}
