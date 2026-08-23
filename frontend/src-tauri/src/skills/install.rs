//! Skill installation — re-exports from services::install for backward compatibility.
//!
//! The actual implementation lives in `crate::services::install`.

pub use crate::services::install::{
    build_skill_record, compute_skill_file_stats, dedupe_install_result, install_local_skill,
    install_local_skill_from_selection, list_local_skills, parse_skill_md,
    upsert_skill_from_install, InstallResult, LocalSkillCandidate, SkillFrontmatter,
};
