use chrono::Utc;
use trace_common::model::{SecurityFinding, SecurityReport, Severity};

#[derive(Default)]
pub struct ReportBuilder {
    findings: Vec<SecurityFinding>,
    package_id: String,
    version: u64,
}

impl ReportBuilder {
    pub fn new(package_id: impl Into<String>, version: u64) -> Self {
        Self {
            findings: Vec::new(),
            package_id: package_id.into(),
            version,
        }
    }

    pub fn push(&mut self, f: SecurityFinding) {
        self.findings.push(f);
    }

    pub fn extend(&mut self, fs: impl IntoIterator<Item = SecurityFinding>) {
        self.findings.extend(fs);
    }

    pub fn finish(self) -> SecurityReport {
        let mut max_severity = Severity::Info;
        let mut score = 0.0f32;
        for f in &self.findings {
            score += f.severity.weight() * f.confidence;
            if (severity_rank(f.severity)) > severity_rank(max_severity) {
                max_severity = f.severity;
            }
        }
        // Cap the score to a 0..=10 range for nicer UX.
        let normalised = (score / 4.0).min(10.0);
        SecurityReport {
            package_id: self.package_id,
            version: self.version,
            score: normalised,
            max_severity,
            findings: self.findings,
            scanned_at: Utc::now(),
        }
    }
}

fn severity_rank(s: Severity) -> u8 {
    match s {
        Severity::Info => 0,
        Severity::Low => 1,
        Severity::Medium => 2,
        Severity::High => 3,
        Severity::Critical => 4,
    }
}
