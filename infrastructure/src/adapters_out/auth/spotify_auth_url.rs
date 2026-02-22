use domain::ports::ports_out::auth::auth_url_builder::AuthUrlBuilder;
use url::Url;

pub struct SpotifyAuthUrlBuilder {
    auth_uri: String,
    client_id: String,
    redirect_uri: String,
    scopes: Vec<String>,
}

impl SpotifyAuthUrlBuilder {
    pub fn new(
        auth_uri: String,
        client_id: String,
        redirect_uri: String,
        scopes: Vec<String>,
    ) -> Self {
        Self { auth_uri, client_id, redirect_uri, scopes }
    }
}

impl AuthUrlBuilder for SpotifyAuthUrlBuilder {
    fn build_authorize_url(&self, code_challenge: &str, state: &str) -> String {
        let mut url = Url::parse(&self.auth_uri)
            .expect("invalid auth_uri in configuration");

        url.query_pairs_mut()
            .append_pair("client_id", &self.client_id)
            .append_pair("response_type", "code")
            .append_pair("redirect_uri", &self.redirect_uri)
            .append_pair("code_challenge_method", "S256")
            .append_pair("code_challenge", code_challenge)
            .append_pair("scope", &self.scopes.join(" "))
            .append_pair("state", state);

        url.to_string()
    }
}
