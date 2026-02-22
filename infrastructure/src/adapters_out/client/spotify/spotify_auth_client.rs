use domain::{
    errors::errors::AppResult,
    ports::ports_out::client::spotify_auth::{SpotifyAuthClient, TokenResponse},
};
use serde::Deserialize;

pub struct UreqSpotifyAuthClient {
    token_uri: String,
    client_id: String,
    redirect_uri: String,
}

impl UreqSpotifyAuthClient {
    pub fn new(token_uri: String, client_id: String, redirect_uri: String) -> Self {
        Self { token_uri, client_id, redirect_uri }
    }
}

#[derive(Deserialize)]
struct SpotifyTokenResponse {
    access_token: String,
    refresh_token: String,
    expires_in: u64,
}

#[derive(Deserialize)]
struct SpotifyRefreshResponse {
    access_token: String,
    refresh_token: Option<String>,
    expires_in: u64,
}

impl SpotifyAuthClient for UreqSpotifyAuthClient {
    fn exchange_code(&self, code: &str, code_verifier: &str) -> AppResult<TokenResponse> {
        let resp: SpotifyTokenResponse = ureq::post(&self.token_uri)
            .send_form(&[
                ("grant_type", "authorization_code"),
                ("code", code),
                ("redirect_uri", &self.redirect_uri),
                ("client_id", &self.client_id),
                ("code_verifier", code_verifier),
            ])
            .map_err(|e| anyhow::anyhow!("Token exchange request failed: {e}"))?
            .into_json()
            .map_err(|e| anyhow::anyhow!("Failed to parse token response: {e}"))?;

        Ok(TokenResponse {
            access_token: resp.access_token,
            refresh_token: resp.refresh_token,
            expires_in: resp.expires_in,
        })
    }

    fn refresh_token(&self, refresh_token: &str) -> AppResult<TokenResponse> {
        let resp: SpotifyRefreshResponse = ureq::post(&self.token_uri)
            .send_form(&[
                ("grant_type", "refresh_token"),
                ("refresh_token", refresh_token),
                ("client_id", &self.client_id),
            ])
            .map_err(|e| anyhow::anyhow!("Token refresh request failed: {e}"))?
            .into_json()
            .map_err(|e| anyhow::anyhow!("Failed to parse refresh response: {e}"))?;

        Ok(TokenResponse {
            access_token: resp.access_token,
            refresh_token: resp.refresh_token.unwrap_or_else(|| refresh_token.to_string()),
            expires_in: resp.expires_in,
        })
    }
}
