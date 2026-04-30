use trace_common::model::{SecurityFinding, Severity};

use crate::context::ModuleContext;
use crate::rules::{Rule, finding};

pub struct Rule008;

impl Rule for Rule008 {
    fn id(&self) -> &'static str {
        "R008"
    }

    fn name(&self) -> &'static str {
        "Unbounded Loop / DoS Vector"
    }

    fn check(&self, module: &ModuleContext) -> Vec<SecurityFinding> {
        let mut out = Vec::new();
        for f in &module.functions {
            let has_loop = f.has_tag("has_loop");
            let bounded = f.has_tag("loop_bound_constant");
            let user_input = f
                .parameters
                .iter()
                .any(|p| p.type_name.contains("vector") || p.type_name.contains("Table"));
            if has_loop && !bounded && user_input {
                out.push(finding(
                    self.id(),
                    self.name(),
                    Severity::Medium,
                    0.6,
                    &module.name,
                    Some(&f.name),
                    &format!("{}::{}", module.name, f.name),
                    "Loop body iterates over a user-supplied vector without an upper bound, allowing the caller to pay through the gas roof and brick the transaction.",
                    "Cap the iteration with `assert!(vector::length(&v) <= MAX_LEN, ...)` or paginate the operation.",
                ));
            }
        }
        out
    }
}
