use trace_common::model::{SecurityFinding, Severity};

use crate::context::ModuleContext;
use crate::rules::{Rule, finding};

pub struct Rule010;

impl Rule for Rule010 {
    fn id(&self) -> &'static str {
        "R010"
    }

    fn name(&self) -> &'static str {
        "Loose Upgrade Policy"
    }

    fn check(&self, module: &ModuleContext) -> Vec<SecurityFinding> {
        let mut out = Vec::new();
        // Heuristic: an `init` function that calls `package::make_immutable` is
        // safe; one that does not, *and* exposes admin capabilities is risky.
        for f in &module.functions {
            if f.name == "init" {
                let immutable = f.calls("package", "make_immutable");
                let exposes_cap = module.structs.iter().any(|s| s.is_capability);
                if !immutable && exposes_cap {
                    out.push(finding(
                        self.id(),
                        self.name(),
                        Severity::Medium,
                        0.6,
                        &module.name,
                        Some(&f.name),
                        &format!("{}::init", module.name),
                        "Module `init` keeps the upgrade cap mutable while exposing administrative capabilities; a compromised admin can ship a malicious upgrade in one transaction.",
                        "Either call `package::make_immutable(upgrade_cap)`, place the cap behind a multisig, or document the upgrade governance in `OnChain Governance`.",
                    ));
                }
            }
        }
        out
    }
}
