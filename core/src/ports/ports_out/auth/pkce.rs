pub struct PkceChallenge {
    pub verifier: String,
    pub challenge: String,
    pub state: String,
}

pub trait PkceGenerator: Send + Sync {
    fn generate(&self) -> PkceChallenge;
}
