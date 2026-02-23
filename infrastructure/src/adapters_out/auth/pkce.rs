use base64::{Engine, engine::general_purpose::URL_SAFE_NO_PAD};
use domain::ports::ports_out::auth::pkce::{PkceChallenge, PkceGenerator};
use rand::Rng;
use sha2::{Digest, Sha256};
use tracing::debug;

pub struct Sha256PkceGenerator;

impl PkceGenerator for Sha256PkceGenerator {
    fn generate(&self) -> PkceChallenge {
        debug!("Generating PKCE challenge");
        let verifier = generate_random_string(128, PKCE_CHARSET);
        let challenge = generate_challenge(&verifier);
        let state = generate_random_string(32, ALPHANUMERIC_CHARSET);
        PkceChallenge { verifier, challenge, state }
    }
}

const PKCE_CHARSET: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789-._~";
const ALPHANUMERIC_CHARSET: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789";

fn generate_random_string(len: usize, charset: &[u8]) -> String {
    let mut rng = rand::thread_rng();
    (0..len)
        .map(|_| {
            let idx = rng.gen_range(0..charset.len());
            charset[idx] as char
        })
        .collect()
}

fn generate_challenge(verifier: &str) -> String {
    let hash = Sha256::digest(verifier.as_bytes());
    URL_SAFE_NO_PAD.encode(hash)
}
