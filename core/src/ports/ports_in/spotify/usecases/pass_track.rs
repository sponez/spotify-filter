use crate::errors::errors::AppResult;

pub trait PassTrackUseCase {
    fn pass_current_track(&self) -> AppResult<()>;
}
