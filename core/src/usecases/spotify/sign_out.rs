use crate::ports::ports_in::spotify::usecases::sign_out::SignOutUseCase;

pub struct SignOutInteractor;

impl SignOutInteractor {
    pub fn new() -> Self {
        Self
    }
}

impl SignOutUseCase for SignOutInteractor {
    fn sign_out(&self) {
        println!("Signed out.")
    }
}
