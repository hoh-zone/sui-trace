use trace_common::model::{SecurityFinding, Severity};

use crate::context::ModuleContext;
use crate::rules::{Rule, finding};

pub struct Rule007;

impl Rule for Rule007 {
    fn id(&self) -> &'static str {
        "R007"
    }

    fn name(&self) -> &'static str {
        "Untrusted External Call"
    }

    fn check(&self, module: &ModuleContext) -> Vec<SecurityFinding> {
        let mut out = Vec::new();
        for f in &module.functions {
            for callee in &f.callees {
                let is_external = callee.package_id != module.package_id
                    && !callee.package_id.starts_with("0x1")
                    && !callee.package_id.starts_with("0x2")
                    && !callee.package_id.starts_with("0x3");
                if is_external && callee.function == "transfer" {
                    out.push(finding(
                        self.id(),
                        self.name(),
                        Severity::Medium,
                        0.5,
                        &module.name,
                        Some(&f.name),
                        &format!("{}::{}", module.name, f.name),
                        &format!(
                            "Function calls `{}::{}` in an untrusted external package `{}`.",
                            callee.module, callee.function, callee.package_id
                        ),
                        "Pin the dependency to a known package version and document the trust boundary, or replace with a vetted helper.",
                    ));
                }
            }
        }
        out
    }
}
