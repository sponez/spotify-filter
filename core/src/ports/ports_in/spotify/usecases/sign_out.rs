use crate::errors::errors::AppResult;

pub trait SignOutUseCase {
    fn sign_out(&self) -> AppResult<()>;
}
