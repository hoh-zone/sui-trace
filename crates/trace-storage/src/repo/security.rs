use chrono::{DateTime, Utc};
use sqlx::Row;
use trace_common::{
    Error,
    error::Result,
    model::{SecurityFinding, SecurityReport, Severity},
};

use crate::Db;

pub struct SecurityRepo<'a> {
    db: &'a Db,
}

impl<'a> SecurityRepo<'a> {
    pub fn new(db: &'a Db) -> Self {
        Self { db }
    }

    pub async fn save_report(&self, report: &SecurityReport) -> Result<()> {
        let mut tx = self
            .db
            .pool()
            .begin()
            .await
            .map_err(|e| Error::Database(e.to_string()))?;

        sqlx::query(
            r#"
            INSERT INTO security_reports
                (package_id, version, score, max_severity, findings_count, scanned_at)
            VALUES ($1, $2, $3, $4, $5, $6)
            ON CONFLICT (package_id, version) DO UPDATE SET
                score = EXCLUDED.score,
                max_severity = EXCLUDED.max_severity,
                findings_count = EXCLUDED.findings_count,
                scanned_at = EXCLUDED.scanned_at
            "#,
        )
        .bind(&report.package_id)
        .bind(report.version as i64)
        .bind(report.score)
        .bind(severity_to_str(report.max_severity))
        .bind(report.findings.len() as i32)
        .bind(report.scanned_at)
        .execute(&mut *tx)
        .await
        .map_err(|e| Error::Database(e.to_string()))?;

        sqlx::query("DELETE FROM security_findings WHERE package_id = $1 AND version = $2")
            .bind(&report.package_id)
            .bind(report.version as i64)
            .execute(&mut *tx)
            .await
            .map_err(|e| Error::Database(e.to_string()))?;

        for f in &report.findings {
            sqlx::query(
                r#"
                INSERT INTO security_findings
                    (package_id, version, rule_id, rule_name, severity, confidence, module, function, location, message, suggestion)
                VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11)
                "#,
            )
            .bind(&report.package_id)
            .bind(report.version as i64)
            .bind(&f.rule_id)
            .bind(&f.rule_name)
            .bind(severity_to_str(f.severity))
            .bind(f.confidence)
            .bind(&f.module)
            .bind(&f.function)
            .bind(&f.location)
            .bind(&f.message)
            .bind(&f.suggestion)
            .execute(&mut *tx)
            .await
            .map_err(|e| Error::Database(e.to_string()))?;
        }

        tx.commit()
            .await
            .map_err(|e| Error::Database(e.to_string()))?;
        Ok(())
    }

    pub async fn get_report(&self, package_id: &str) -> Result<Option<SecurityReport>> {
        let row = sqlx::query(
            r#"SELECT package_id, version, score, max_severity, scanned_at
               FROM security_reports WHERE package_id = $1
               ORDER BY version DESC LIMIT 1"#,
        )
        .bind(package_id)
        .fetch_optional(self.db.pool())
        .await
        .map_err(|e| Error::Database(e.to_string()))?;

        let Some(r) = row else { return Ok(None) };
        let pkg: String = r.get(0);
        let version: i64 = r.get(1);
        let score: f32 = r.get(2);
        let max_severity: String = r.get(3);
        let scanned_at: DateTime<Utc> = r.get(4);
        let findings = self.list_findings(&pkg, version as u64).await?;

        Ok(Some(SecurityReport {
            package_id: pkg,
            version: version as u64,
            score,
            max_severity: severity_from_str(&max_severity),
            findings,
            scanned_at,
        }))
    }

    pub async fn list_findings(
        &self,
        package_id: &str,
        version: u64,
    ) -> Result<Vec<SecurityFinding>> {
        let rows = sqlx::query(
            r#"SELECT rule_id, rule_name, severity, confidence, module, function, location, message, suggestion
               FROM security_findings WHERE package_id = $1 AND version = $2
               ORDER BY severity DESC"#,
        )
        .bind(package_id)
        .bind(version as i64)
        .fetch_all(self.db.pool())
        .await
        .map_err(|e| Error::Database(e.to_string()))?;
        Ok(rows
            .into_iter()
            .map(|r| SecurityFinding {
                rule_id: r.get(0),
                rule_name: r.get(1),
                severity: severity_from_str(&r.get::<String, _>(2)),
                confidence: r.get(3),
                module: r.get(4),
                function: r.get(5),
                location: r.get(6),
                message: r.get(7),
                suggestion: r.get(8),
            })
            .collect())
    }

    /// Latest individual findings across all packages, joined with the
    /// owning report so we can render a global feed.
    pub async fn recent_findings(&self, limit: i64) -> Result<Vec<RecentFinding>> {
        let rows = sqlx::query(
            r#"SELECT f.package_id, f.version, f.rule_id, f.rule_name, f.severity,
                      f.confidence, f.module, f.function, f.location, f.message,
                      f.suggestion, r.scanned_at
               FROM security_findings f
               JOIN security_reports r
                 ON r.package_id = f.package_id AND r.version = f.version
               ORDER BY r.scanned_at DESC, f.id DESC
               LIMIT $1"#,
        )
        .bind(limit)
        .fetch_all(self.db.pool())
        .await
        .map_err(|e| Error::Database(e.to_string()))?;
        Ok(rows
            .into_iter()
            .map(|r| RecentFinding {
                package_id: r.get(0),
                version: r.get::<i64, _>(1) as u64,
                finding: SecurityFinding {
                    rule_id: r.get(2),
                    rule_name: r.get(3),
                    severity: severity_from_str(&r.get::<String, _>(4)),
                    confidence: r.get(5),
                    module: r.get(6),
                    function: r.get(7),
                    location: r.get(8),
                    message: r.get(9),
                    suggestion: r.get(10),
                },
                scanned_at: r.get(11),
            })
            .collect())
    }

    /// (severity, count) pairs over all findings in the last `days` days.
    pub async fn severity_counts(&self, days: i64) -> Result<Vec<(Severity, i64)>> {
        let rows = sqlx::query(
            r#"SELECT f.severity, COUNT(*)::bigint
               FROM security_findings f
               JOIN security_reports r
                 ON r.package_id = f.package_id AND r.version = f.version
               WHERE r.scanned_at >= NOW() - ($1 || ' days')::interval
               GROUP BY f.severity"#,
        )
        .bind(days.to_string())
        .fetch_all(self.db.pool())
        .await
        .map_err(|e| Error::Database(e.to_string()))?;
        Ok(rows
            .into_iter()
            .map(|r| {
                (
                    severity_from_str(&r.get::<String, _>(0)),
                    r.get::<i64, _>(1),
                )
            })
            .collect())
    }

    /// (rule_id, rule_name, severity, count) triggered most often in the window.
    pub async fn rule_rankings(&self, days: i64, limit: i64) -> Result<Vec<RuleRanking>> {
        let rows = sqlx::query(
            r#"SELECT f.rule_id, f.rule_name, f.severity, COUNT(*)::bigint AS hits
               FROM security_findings f
               JOIN security_reports r
                 ON r.package_id = f.package_id AND r.version = f.version
               WHERE r.scanned_at >= NOW() - ($1 || ' days')::interval
               GROUP BY f.rule_id, f.rule_name, f.severity
               ORDER BY hits DESC
               LIMIT $2"#,
        )
        .bind(days.to_string())
        .bind(limit)
        .fetch_all(self.db.pool())
        .await
        .map_err(|e| Error::Database(e.to_string()))?;
        Ok(rows
            .into_iter()
            .map(|r| RuleRanking {
                rule_id: r.get(0),
                rule_name: r.get(1),
                severity: severity_from_str(&r.get::<String, _>(2)),
                hits: r.get(3),
            })
            .collect())
    }
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct RecentFinding {
    pub package_id: String,
    pub version: u64,
    pub finding: SecurityFinding,
    pub scanned_at: DateTime<Utc>,
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct RuleRanking {
    pub rule_id: String,
    pub rule_name: String,
    pub severity: Severity,
    pub hits: i64,
}

fn severity_to_str(s: Severity) -> &'static str {
    match s {
        Severity::Info => "info",
        Severity::Low => "low",
        Severity::Medium => "medium",
        Severity::High => "high",
        Severity::Critical => "critical",
    }
}

fn severity_from_str(s: &str) -> Severity {
    match s {
        "info" => Severity::Info,
        "low" => Severity::Low,
        "medium" => Severity::Medium,
        "high" => Severity::High,
        "critical" => Severity::Critical,
        _ => Severity::Info,
    }
}
