//! The orchestration layer that loads modules and applies every rule.

use std::sync::Arc;

use trace_common::{error::Result, model::SecurityReport};
use trace_storage::Db;
use trace_storage::repo::security::SecurityRepo;

use crate::context::ModuleContext;
use crate::report::ReportBuilder;
use crate::rules::{Rule, all_rules};

#[derive(Clone)]
pub struct SecurityEngine {
    db: Db,
    rules: Vec<Arc<dyn Rule>>,
}

impl SecurityEngine {
    pub fn new(db: Db) -> Self {
        Self {
            db,
            rules: all_rules(),
        }
    }

    pub fn rule_count(&self) -> usize {
        self.rules.len()
    }

    pub fn analyse(
        &self,
        package_id: &str,
        version: u64,
        modules: &[ModuleContext],
    ) -> SecurityReport {
        let mut builder = ReportBuilder::new(package_id, version);
        for m in modules {
            for r in &self.rules {
                let findings = r.check(m);
                builder.extend(findings);
            }
        }
        builder.finish()
    }

    pub async fn analyse_and_save(
        &self,
        package_id: &str,
        version: u64,
        modules: &[ModuleContext],
    ) -> Result<SecurityReport> {
        let report = self.analyse(package_id, version, modules);
        SecurityRepo::new(&self.db).save_report(&report).await?;
        Ok(report)
    }
}
