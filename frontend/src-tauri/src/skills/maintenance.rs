//! Maintenance — re-exports from services::maintenance for backward compatibility.
//!
//! The actual implementation lives in `crate::services::maintenance`.

pub use crate::services::maintenance::{
    repair_sync_health, scan_sync_health, HealthIssue, IssueSummary, RepairOperation, RepairReport,
    SyncHealthReport,
};
