use trace_common::model::{SecurityFinding, Severity};

use crate::context::ModuleContext;
use crate::rules::{Rule, finding};

pub struct Rule003;

impl Rule for Rule003 {
    fn id(&self) -> &'static str {
        "R003"
    }

    fn name(&self) -> &'static str {
        "Clock Unit Mismatch"
    }

    fn check(&self, module: &ModuleContext) -> Vec<SecurityFinding> {
        let mut out = Vec::new();
        for f in &module.functions {
            if f.calls("clock", "timestamp_ms") && f.has_tag("compares_to_seconds") {
                out.push(finding(
                    self.id(),
                    self.name(),
                    Severity::High,
                    0.8,
                    &module.name,
                    Some(&f.name),
                    &format!("{}::{}", module.name, f.name),
                    "`clock::timestamp_ms` returns milliseconds but the value appears to be compared against a second-based field; off-by-1000 errors are common attack vectors.",
                    "Normalise both sides to milliseconds, or divide by 1000 explicitly with a comment.",
                ));
            }
        }
        out
    }
}
