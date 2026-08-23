//! Onboarding — re-exports from services::onboarding for backward compatibility.
//!
//! The actual implementation lives in `crate::services::onboarding`.

pub use crate::services::onboarding::{
    build_onboarding_plan, OnboardingGroup, OnboardingPlan, OnboardingVariant,
};
