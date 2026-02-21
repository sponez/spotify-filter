use crate::errors::errors::AppResult;

pub trait SignInUseCase {
    fn sign_in(&self) -> AppResult<()>;
}
