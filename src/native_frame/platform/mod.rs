use iced::{Task, window};

use std::sync::Arc;

use super::Shared;

#[cfg(windows)]
mod windows;

pub(crate) fn install(
    window_id: window::Id,
    shared: Arc<Shared>,
) -> Task<Result<(), String>> {
    #[cfg(windows)]
    {
        windows::install(window_id, shared)
    }

    #[cfg(not(windows))]
    {
        let _ = (window_id, shared);
        Task::done(Ok(()))
    }
}
