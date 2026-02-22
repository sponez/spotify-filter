use crate::errors::errors::AppResult;

pub struct TokenResponse {
    pub access_token: String,
    pub refresh_token: String,
    pub expires_in: u64,
}

pub trait SpotifyAuthClient: Send + Sync {
    fn exchange_code(
        &self,
        code: &str,
        code_verifier: &str,
    ) -> AppResult<TokenResponse>;

    fn refresh_token(
        &self,
        refresh_token: &str,
    ) -> AppResult<TokenResponse>;
}
