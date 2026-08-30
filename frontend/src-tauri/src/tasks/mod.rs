//! Task manager.
//!
//! Provides a lightweight in-process task system for long-running operations.
//! Tasks run on background threads and report progress via events.

use serde::Serialize;
use std::sync::{Arc, Mutex};
use std::time::{SystemTime, UNIX_EPOCH};

fn now_ms() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis() as i64
}

/// Task status values.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum TaskStatus {
    Pending,
    Running,
    Succeeded,
    Failed,
    Canceled,
}

impl std::fmt::Display for TaskStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TaskStatus::Pending => write!(f, "pending"),
            TaskStatus::Running => write!(f, "running"),
            TaskStatus::Succeeded => write!(f, "succeeded"),
            TaskStatus::Failed => write!(f, "failed"),
            TaskStatus::Canceled => write!(f, "canceled"),
        }
    }
}

/// A task record tracking state and progress.
#[derive(Debug, Clone, Serialize)]
pub struct TaskRecord {
    pub id: String,
    pub kind: String,
    pub status: TaskStatus,
    pub progress: u32,
    pub message: String,
    pub result: Option<serde_json::Value>,
    pub error: Option<String>,
    pub logs: Vec<String>,
    pub cancel_requested: bool,
    pub created_at: i64,
    pub started_at: Option<i64>,
    pub finished_at: Option<i64>,
}

/// Context passed to task functions for progress reporting.
pub struct TaskContext {
    manager: Arc<TaskManagerInner>,
    pub task_id: String,
}

impl TaskContext {
    /// Log a message for this task.
    pub fn log(&self, message: &str) {
        self.manager.log(&self.task_id, message);
    }

    /// Set progress (0-100) with an optional message.
    pub fn set_progress(&self, progress: u32, message: &str) {
        self.manager.set_progress(&self.task_id, progress, message);
    }

    /// Check if cancellation has been requested.
    pub fn is_cancelled(&self) -> bool {
        self.manager.is_cancelled(&self.task_id)
    }

    /// Raise-style check: returns Err if cancelled.
    pub fn check_cancelled(&self) -> Result<(), String> {
        if self.is_cancelled() {
            Err("任务已取消 / task cancelled".to_string())
        } else {
            Ok(())
        }
    }
}

/// Error type for cancelled tasks.
#[derive(Debug)]
pub struct TaskCancelled;

impl std::fmt::Display for TaskCancelled {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "task cancelled")
    }
}

/// Type alias for task functions.
pub type TaskFn = Box<dyn FnOnce(&TaskContext) -> Result<serde_json::Value, String> + Send>;

struct TaskManagerInner {
    tasks: Mutex<std::collections::HashMap<String, TaskRecord>>,
}

impl TaskManagerInner {
    fn new() -> Self {
        Self {
            tasks: Mutex::new(std::collections::HashMap::new()),
        }
    }

    fn log(&self, task_id: &str, message: &str) {
        if let Ok(mut tasks) = self.tasks.lock() {
            if let Some(task) = tasks.get_mut(task_id) {
                task.logs.push(message.to_string());
                task.message = message.to_string();
                // Keep only last 200 log entries
                if task.logs.len() > 200 {
                    task.logs.drain(..task.logs.len() - 200);
                }
            }
        }
    }

    fn set_progress(&self, task_id: &str, progress: u32, message: &str) {
        if let Ok(mut tasks) = self.tasks.lock() {
            if let Some(task) = tasks.get_mut(task_id) {
                task.progress = progress.clamp(0, 100);
                if !message.is_empty() {
                    task.message = message.to_string();
                }
            }
        }
    }

    fn is_cancelled(&self, task_id: &str) -> bool {
        if let Ok(tasks) = self.tasks.lock() {
            tasks
                .get(task_id)
                .map(|t| t.cancel_requested)
                .unwrap_or(false)
        } else {
            false
        }
    }
}

/// The task manager handles submission and lifecycle of background tasks.
pub struct TaskManager {
    inner: Arc<TaskManagerInner>,
}

impl TaskManager {
    pub fn new() -> Self {
        Self {
            inner: Arc::new(TaskManagerInner::new()),
        }
    }

    /// Submit a new task for background execution.
    pub fn submit(&self, kind: &str, task_fn: TaskFn) -> TaskRecord {
        let task_id = uuid::Uuid::new_v4().to_string();
        let record = TaskRecord {
            id: task_id.clone(),
            kind: kind.to_string(),
            status: TaskStatus::Pending,
            progress: 0,
            message: String::new(),
            result: None,
            error: None,
            logs: Vec::new(),
            cancel_requested: false,
            created_at: now_ms(),
            started_at: None,
            finished_at: None,
        };

        {
            let mut tasks = self.inner.tasks.lock().unwrap();
            tasks.insert(task_id.clone(), record.clone());
        }

        let inner = self.inner.clone();
        let tid = task_id.clone();

        std::thread::Builder::new()
            .name(format!("skills-hub-task-{}", &tid[..8]))
            .spawn(move || {
                Self::run_task(inner, tid, task_fn);
            })
            .expect("failed to spawn task thread");

        record
    }

    fn run_task(inner: Arc<TaskManagerInner>, task_id: String, task_fn: TaskFn) {
        // Mark as running and capture kind for error logging
        let kind = {
            let mut tasks = inner.tasks.lock().unwrap();
            if let Some(task) = tasks.get_mut(&task_id) {
                task.status = TaskStatus::Running;
                task.started_at = Some(now_ms());
                task.message = "running".to_string();
                task.kind.clone()
            } else {
                return;
            }
        };

        let ctx = TaskContext {
            manager: inner.clone(),
            task_id: task_id.clone(),
        };

        match task_fn(&ctx) {
            Ok(result) => {
                let mut tasks = inner.tasks.lock().unwrap();
                if let Some(task) = tasks.get_mut(&task_id) {
                    if task.cancel_requested {
                        task.status = TaskStatus::Canceled;
                        task.message = "cancelled".to_string();
                    } else {
                        task.status = TaskStatus::Succeeded;
                        task.progress = 100;
                        task.message = "completed".to_string();
                        task.result = Some(result);
                    }
                    task.finished_at = Some(now_ms());
                }
            }
            Err(err) => {
                log::error!("[TASK_FAILED] task_id={} kind={} error={}", task_id, kind, err);
                let mut tasks = inner.tasks.lock().unwrap();
                if let Some(task) = tasks.get_mut(&task_id) {
                    if task.cancel_requested || err.contains("cancelled") {
                        task.status = TaskStatus::Canceled;
                        task.message = err;
                    } else {
                        task.status = TaskStatus::Failed;
                        task.error = Some(err.clone());
                        task.message = "failed".to_string();
                        task.logs.push(err);
                    }
                    task.finished_at = Some(now_ms());
                }
            }
        }
    }

    /// Get a task record by ID.
    pub fn get(&self, task_id: &str) -> Option<TaskRecord> {
        let tasks = self.inner.tasks.lock().ok()?;
        tasks.get(task_id).cloned()
    }

    /// List all tasks, sorted by creation time (newest first).
    pub fn list(&self) -> Vec<TaskRecord> {
        let tasks = self.inner.tasks.lock().unwrap();
        let mut records: Vec<_> = tasks.values().cloned().collect();
        records.sort_by(|a, b| b.created_at.cmp(&a.created_at));
        records
    }

    /// Request cancellation of a task.
    pub fn cancel(&self, task_id: &str) -> bool {
        let mut tasks = self.inner.tasks.lock().unwrap();
        if let Some(task) = tasks.get_mut(task_id) {
            match task.status {
                TaskStatus::Succeeded | TaskStatus::Failed | TaskStatus::Canceled => true,
                _ => {
                    task.cancel_requested = true;
                    task.message = "cancellation requested".to_string();
                    true
                }
            }
        } else {
            false
        }
    }

    /// Cancel all running/pending tasks.
    pub fn cancel_all_running(&self) -> usize {
        let mut tasks = self.inner.tasks.lock().unwrap();
        let mut count = 0;
        for task in tasks.values_mut() {
            if task.status == TaskStatus::Pending || task.status == TaskStatus::Running {
                task.cancel_requested = true;
                task.message = "cancellation requested".to_string();
                count += 1;
            }
        }
        count
    }
}

impl Default for TaskManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_task_submit_and_complete() {
        let mgr = TaskManager::new();
        let record = mgr.submit(
            "test",
            Box::new(|ctx| {
                ctx.set_progress(50, "halfway");
                ctx.log("working...");
                Ok(serde_json::json!({"done": true}))
            }),
        );

        assert_eq!(record.status, TaskStatus::Pending);

        // Wait for completion
        std::thread::sleep(std::time::Duration::from_millis(200));

        let updated = mgr.get(&record.id).unwrap();
        assert_eq!(updated.status, TaskStatus::Succeeded);
        assert_eq!(updated.progress, 100);
        assert!(updated.result.is_some());
    }

    #[test]
    fn test_task_failure() {
        let mgr = TaskManager::new();
        let record = mgr.submit(
            "fail_test",
            Box::new(|_ctx| Err("something went wrong".to_string())),
        );

        std::thread::sleep(std::time::Duration::from_millis(200));

        let updated = mgr.get(&record.id).unwrap();
        assert_eq!(updated.status, TaskStatus::Failed);
        assert!(updated.error.is_some());
    }

    #[test]
    fn test_task_cancel() {
        let mgr = TaskManager::new();
        let record = mgr.submit(
            "cancel_test",
            Box::new(|ctx| {
                // Simulate long work
                for _ in 0..100 {
                    if ctx.is_cancelled() {
                        return Err("任务已取消 / task cancelled".to_string());
                    }
                    std::thread::sleep(std::time::Duration::from_millis(10));
                }
                Ok(serde_json::json!(null))
            }),
        );

        std::thread::sleep(std::time::Duration::from_millis(50));
        assert!(mgr.cancel(&record.id));

        std::thread::sleep(std::time::Duration::from_millis(200));

        let updated = mgr.get(&record.id).unwrap();
        assert_eq!(updated.status, TaskStatus::Canceled);
    }

    #[test]
    fn test_task_list() {
        let mgr = TaskManager::new();
        mgr.submit("a", Box::new(|_| Ok(serde_json::json!(null))));
        mgr.submit("b", Box::new(|_| Ok(serde_json::json!(null))));

        std::thread::sleep(std::time::Duration::from_millis(200));

        let list = mgr.list();
        assert_eq!(list.len(), 2);
    }

    #[test]
    fn test_cancel_all_running() {
        let mgr = TaskManager::new();
        mgr.submit(
            "long1",
            Box::new(|ctx| {
                for _ in 0..100 {
                    if ctx.is_cancelled() {
                        return Err("cancelled".to_string());
                    }
                    std::thread::sleep(std::time::Duration::from_millis(10));
                }
                Ok(serde_json::json!(null))
            }),
        );
        mgr.submit(
            "long2",
            Box::new(|ctx| {
                for _ in 0..100 {
                    if ctx.is_cancelled() {
                        return Err("cancelled".to_string());
                    }
                    std::thread::sleep(std::time::Duration::from_millis(10));
                }
                Ok(serde_json::json!(null))
            }),
        );

        std::thread::sleep(std::time::Duration::from_millis(50));
        let count = mgr.cancel_all_running();
        assert!(count >= 1);
    }

    #[test]
    fn test_task_context_progress() {
        let mgr = TaskManager::new();
        let record = mgr.submit(
            "progress_test",
            Box::new(|ctx| {
                ctx.set_progress(25, "quarter");
                ctx.set_progress(50, "half");
                ctx.set_progress(75, "three quarters");
                ctx.log("step 1");
                ctx.log("step 2");
                Ok(serde_json::json!({"steps": 3}))
            }),
        );

        std::thread::sleep(std::time::Duration::from_millis(200));

        let updated = mgr.get(&record.id).unwrap();
        assert_eq!(updated.status, TaskStatus::Succeeded);
        assert_eq!(updated.progress, 100); // Set to 100 on success
        assert!(updated.logs.len() >= 2);
    }
}
