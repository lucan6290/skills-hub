use std::sync::Arc;
use std::time::Instant;

use crate::db::Database;
use crate::tasks::TaskManager;

pub struct AppState {
    pub started_at: Instant,
    pub db: Arc<Database>,
    pub task_manager: Arc<TaskManager>,
}

impl std::fmt::Debug for AppState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("AppState")
            .field("started_at", &self.started_at)
            .field("db", &"<Database>")
            .field("task_manager", &"<TaskManager>")
            .finish()
    }
}

impl AppState {
    pub fn new(db: Database) -> Self {
        Self {
            started_at: Instant::now(),
            db: Arc::new(db),
            task_manager: Arc::new(TaskManager::new()),
        }
    }
}

impl Default for AppState {
    fn default() -> Self {
        let db_path = crate::config::default_db_path();
        let db = Database::new(&db_path).expect("failed to initialize database");
        Self::new(db)
    }
}
