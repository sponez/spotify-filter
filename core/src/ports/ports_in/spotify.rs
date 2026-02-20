pub trait SignInUseCase {
    fn sign_in(&self);
}

pub trait SignOutUseCase {
    fn sign_out(&self);
}

pub trait PlayerUseCase {
    fn discard_track(&self);
    fn like_and_discard_track(&self);
}

