use std::time::Instant;

#[derive(Debug)]
pub struct AppState {
    pub started_at: Instant,
}

impl Default for AppState {
    fn default() -> Self {
        Self {
            started_at: Instant::now(),
        }
    }
}
