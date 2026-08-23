pub mod install;
pub mod maintenance;
pub mod onboarding;

pub use install::{
    install_local_skill, install_local_skill_from_selection, list_local_skills, InstallResult,
    LocalSkillCandidate, SkillFrontmatter,
};
pub use maintenance::{repair_sync_health, scan_sync_health, SyncHealthReport};
pub use onboarding::{build_onboarding_plan, OnboardingGroup, OnboardingPlan, OnboardingVariant};
