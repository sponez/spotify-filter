use crate::errors::errors::AppResult;

pub trait PassTrackUseCase: Send + Sync {
    fn pass_current_track(&self) -> AppResult<()>;
}
