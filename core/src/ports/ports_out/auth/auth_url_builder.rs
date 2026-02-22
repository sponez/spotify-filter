pub trait AuthUrlBuilder: Send + Sync {
    fn build_authorize_url(&self, code_challenge: &str, state: &str) -> String;
}
