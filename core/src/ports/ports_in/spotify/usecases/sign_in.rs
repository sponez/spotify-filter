use crate::errors::errors::AppResult;

pub trait SignInUseCase: Send + Sync {
    fn sign_in(&self) -> AppResult<()>;
}
