use crate::{errors::errors::AppResult, ports::ports_in::spotify::usecases::sign_in::SignInUseCase};

pub struct SignInInteractor;

impl SignInInteractor {
    pub fn new() -> Self {
        Self
    }
}

impl SignInUseCase for SignInInteractor {
    fn sign_in(&self) -> AppResult<()> {
        println!("Signed in.");
        
        Ok(())
    }
}
