use trace_common::model::{SecurityFinding, Severity};

use crate::context::{ModuleContext, Visibility};
use crate::rules::{Rule, finding};

pub struct Rule006;

impl Rule for Rule006 {
    fn id(&self) -> &'static str {
        "R006"
    }

    fn name(&self) -> &'static str {
        "Capability Leak"
    }

    fn check(&self, module: &ModuleContext) -> Vec<SecurityFinding> {
        let mut out = Vec::new();
        for f in &module.functions {
            if !matches!(f.visibility, Visibility::Public) {
                continue;
            }
            for ret in &f.return_types {
                let looks_like_cap = ret.type_name.ends_with("Cap")
                    || ret.type_name.ends_with("Capability")
                    || ret.type_name.ends_with("Witness");
                if looks_like_cap {
                    out.push(finding(
                        self.id(),
                        self.name(),
                        Severity::Critical,
                        0.75,
                        &module.name,
                        Some(&f.name),
                        &format!("{}::{}", module.name, f.name),
                        &format!("Public function returns `{}`, allowing arbitrary callers to acquire a privileged capability.", ret.type_name),
                        "Use `transfer::transfer` to a specific recipient at module init, or restrict callers via friend visibility.",
                    ));
                }
            }
        }
        out
    }
}
