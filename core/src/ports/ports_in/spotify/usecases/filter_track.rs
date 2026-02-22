use crate::errors::errors::AppResult;

pub trait FilterTrackUseCase: Send + Sync {
    fn filter_current_track(&self) -> AppResult<()>;
}
