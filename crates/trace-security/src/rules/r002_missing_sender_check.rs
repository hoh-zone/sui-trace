use trace_common::model::{SecurityFinding, Severity};

use crate::context::{ModuleContext, Visibility};
use crate::rules::{Rule, finding};

pub struct Rule002;

impl Rule for Rule002 {
    fn id(&self) -> &'static str {
        "R002"
    }

    fn name(&self) -> &'static str {
        "Missing Sender Authorization"
    }

    fn check(&self, module: &ModuleContext) -> Vec<SecurityFinding> {
        let mut out = Vec::new();
        for f in &module.functions {
            let is_admin = f.name.contains("admin")
                || f.name.contains("set_")
                || f.name.contains("withdraw")
                || f.name.contains("transfer_owner");
            let is_callable =
                matches!(f.visibility, Visibility::Public | Visibility::PublicPackage)
                    || f.is_entry;
            let checks_sender = f.calls("tx_context", "sender")
                || f.has_tag("checks_admin_cap")
                || f.parameters
                    .iter()
                    .any(|p| p.type_name.ends_with("AdminCap"));

            if is_admin && is_callable && !checks_sender {
                out.push(finding(
                    self.id(),
                    self.name(),
                    Severity::Critical,
                    0.7,
                    &module.name,
                    Some(&f.name),
                    &format!("{}::{}", module.name, f.name),
                    "An administrative-looking function is publicly callable without a sender check or capability parameter.",
                    "Either accept a `&AdminCap` parameter, assert the caller via `tx_context::sender(ctx)`, or restrict visibility.",
                ));
            }
        }
        out
    }
}
