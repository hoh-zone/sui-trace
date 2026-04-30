use trace_common::model::{SecurityFinding, Severity};

use crate::context::ModuleContext;
use crate::rules::{Rule, finding};

pub struct Rule004;

impl Rule for Rule004 {
    fn id(&self) -> &'static str {
        "R004"
    }

    fn name(&self) -> &'static str {
        "Unsafe Arithmetic"
    }

    fn check(&self, module: &ModuleContext) -> Vec<SecurityFinding> {
        let mut out = Vec::new();
        for f in &module.functions {
            if f.has_tag("uses_mul") && !f.has_tag("uses_safe_math") {
                out.push(finding(
                    self.id(),
                    self.name(),
                    Severity::Medium,
                    0.6,
                    &module.name,
                    Some(&f.name),
                    &format!("{}::{}", module.name, f.name),
                    "Function performs `*` on user-controlled values without using a checked-math helper.",
                    "Use the `sui::math` helpers (`mul_div_round`) or assert that operands fit in u64 before multiplication.",
                ));
            }
        }
        out
    }
}
