use crate::errors::errors::AppResult;

pub trait SignOutUseCase: Send + Sync {
    fn sign_out(&self) -> AppResult<()>;
}
