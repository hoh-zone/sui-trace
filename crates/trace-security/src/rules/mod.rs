//! Rule library. Rules run independently and produce findings; the engine
//! aggregates and scores them.

use std::sync::Arc;

use trace_common::model::SecurityFinding;

use crate::context::ModuleContext;

mod r001_visibility_confusion;
mod r002_missing_sender_check;
mod r003_clock_unit_mismatch;
mod r004_arithmetic_overflow;
mod r005_unsafe_share_object;
mod r006_capability_leak;
mod r007_untrusted_external_call;
mod r008_dos_unbounded_loop;
mod r009_random_misuse;
mod r010_upgrade_policy_loose;

pub trait Rule: Send + Sync {
    fn id(&self) -> &'static str;
    fn name(&self) -> &'static str;
    fn check(&self, module: &ModuleContext) -> Vec<SecurityFinding>;
}

pub fn all_rules() -> Vec<Arc<dyn Rule>> {
    vec![
        Arc::new(r001_visibility_confusion::Rule001),
        Arc::new(r002_missing_sender_check::Rule002),
        Arc::new(r003_clock_unit_mismatch::Rule003),
        Arc::new(r004_arithmetic_overflow::Rule004),
        Arc::new(r005_unsafe_share_object::Rule005),
        Arc::new(r006_capability_leak::Rule006),
        Arc::new(r007_untrusted_external_call::Rule007),
        Arc::new(r008_dos_unbounded_loop::Rule008),
        Arc::new(r009_random_misuse::Rule009),
        Arc::new(r010_upgrade_policy_loose::Rule010),
    ]
}

#[allow(clippy::too_many_arguments)]
pub(crate) fn finding(
    rule_id: &str,
    rule_name: &str,
    severity: trace_common::model::Severity,
    confidence: f32,
    module: &str,
    function: Option<&str>,
    location: &str,
    message: &str,
    suggestion: &str,
) -> SecurityFinding {
    SecurityFinding {
        rule_id: rule_id.to_string(),
        rule_name: rule_name.to_string(),
        severity,
        confidence,
        module: module.to_string(),
        function: function.map(|s| s.to_string()),
        location: location.to_string(),
        message: message.to_string(),
        suggestion: suggestion.to_string(),
    }
}
