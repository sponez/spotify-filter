use crate::errors::errors::AppResult;

pub trait FilterTrackUseCase {
    fn filter_current_track(&self) -> AppResult<()>;
}
