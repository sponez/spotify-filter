use crate::errors::errors::AppResult;

pub trait TrySignInUseCase: Send + Sync {
    /// Attempts silent sign-in using a stored refresh token.
    /// Returns `true` if signed in, `false` if no token available.
    fn try_sign_in(&self) -> AppResult<bool>;
}
