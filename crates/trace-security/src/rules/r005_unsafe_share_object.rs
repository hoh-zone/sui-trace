use trace_common::model::{SecurityFinding, Severity};

use crate::context::ModuleContext;
use crate::rules::{Rule, finding};

pub struct Rule005;

impl Rule for Rule005 {
    fn id(&self) -> &'static str {
        "R005"
    }

    fn name(&self) -> &'static str {
        "Mutable Shared Object Without Lock"
    }

    fn check(&self, module: &ModuleContext) -> Vec<SecurityFinding> {
        let mut out = Vec::new();
        for s in &module.structs {
            let shareable = s.abilities.iter().any(|a| a == "key");
            if !shareable {
                continue;
            }
            let exposes_balance = s
                .fields
                .iter()
                .any(|(_, t)| t.type_name.contains("Balance") || t.type_name.contains("Coin"));
            if !exposes_balance {
                continue;
            }
            // If any public function takes `&mut Self` we flag the type.
            let mutated_publicly = module.functions.iter().any(|f| {
                matches!(
                    f.visibility,
                    crate::context::Visibility::Public | crate::context::Visibility::PublicPackage
                ) && f
                    .parameters
                    .iter()
                    .any(|p| p.is_mut_ref && p.type_name.contains(&s.name))
            });
            if mutated_publicly {
                out.push(finding(
                    self.id(),
                    self.name(),
                    Severity::High,
                    0.55,
                    &module.name,
                    None,
                    &format!("{}::{}", module.name, s.name),
                    "A shared object that holds value (Balance/Coin) is mutated by a public function without an obvious access-control argument.",
                    "Take an `&AdminCap` or assert `tx_context::sender` matches an authorised principal before mutating treasury state.",
                ));
            }
        }
        out
    }
}
