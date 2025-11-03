use serde::Deserialize;

#[derive(Deserialize)]
pub struct TokenData {
    pub access_token: String,
    pub expires_in: u64,
    pub refresh_token_expires_in: u64,
    pub refresh_token_absolute_expires_in: u64,
    pub token_type: String,
    pub refresh_token: String,
}