use trace_common::model::{SecurityFinding, Severity};

use crate::context::{ModuleContext, Visibility};
use crate::rules::{Rule, finding};

pub struct Rule001;

impl Rule for Rule001 {
    fn id(&self) -> &'static str {
        "R001"
    }

    fn name(&self) -> &'static str {
        "Visibility Confusion"
    }

    fn check(&self, module: &ModuleContext) -> Vec<SecurityFinding> {
        let mut out = Vec::new();
        for f in &module.functions {
            // `public(package) entry` exposes package-private logic to anyone
            // who can craft a tx — almost always a bug.
            if matches!(f.visibility, Visibility::PublicPackage) && f.is_entry {
                out.push(finding(
                    self.id(),
                    self.name(),
                    Severity::High,
                    0.85,
                    &module.name,
                    Some(&f.name),
                    &format!("{}::{}", module.name, f.name),
                    "Function declared `public(package) entry` is callable by anyone via a transaction; the package qualifier provides no protection at the transaction boundary.",
                    "Either drop `entry`, change visibility to `public(friend)` and use a friend cap, or assert the caller package_id explicitly.",
                ));
            }
        }
        out
    }
}
