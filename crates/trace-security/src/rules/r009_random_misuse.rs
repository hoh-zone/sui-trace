use trace_common::model::{SecurityFinding, Severity};

use crate::context::ModuleContext;
use crate::rules::{Rule, finding};

pub struct Rule009;

impl Rule for Rule009 {
    fn id(&self) -> &'static str {
        "R009"
    }

    fn name(&self) -> &'static str {
        "Insecure Randomness"
    }

    fn check(&self, module: &ModuleContext) -> Vec<SecurityFinding> {
        let mut out = Vec::new();
        for f in &module.functions {
            let uses_clock_as_random =
                f.calls("clock", "timestamp_ms") && f.has_tag("derives_random_from_clock");
            let uses_object_id_as_random = f.has_tag("uses_object_id_as_random");
            if uses_clock_as_random || uses_object_id_as_random {
                out.push(finding(
                    self.id(),
                    self.name(),
                    Severity::High,
                    0.7,
                    &module.name,
                    Some(&f.name),
                    &format!("{}::{}", module.name, f.name),
                    "Function appears to derive randomness from a predictable on-chain quantity (clock / object id).",
                    "Use Sui's `0x8::random::Random` shared object via `random::generate_*` for unpredictable randomness.",
                ));
            }
        }
        out
    }
}
