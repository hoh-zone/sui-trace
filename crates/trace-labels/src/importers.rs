//! Bulk importers for the address label database.
//!
//! Each importer is a JSON file shaped like:
//! ```json
//! [
//!   {"address": "0x..", "label": "Binance Hot 1", "category": "cex_hotwallet"},
//!   {"address": "0x..", "label": "Lazarus Group", "category": "hacker", "evidence_url": "https://..."}
//! ]
//! ```

use serde::Deserialize;
use std::path::Path;
use trace_common::{
    Error,
    error::Result,
    model::{AddressLabel, LabelSource},
};
use trace_storage::Db;
use trace_storage::repo::labels::LabelRepo;

use crate::service::LabelService;

#[derive(Debug, Deserialize)]
struct ImportRow {
    address: String,
    label: String,
    category: String,
    #[serde(default)]
    confidence: Option<f32>,
    #[serde(default)]
    evidence_url: Option<String>,
}

pub async fn import_from_file<P: AsRef<Path>>(db: &Db, source: &str, path: P) -> Result<usize> {
    let bytes = std::fs::read(path.as_ref()).map_err(|e| Error::Internal(e.to_string()))?;
    let rows: Vec<ImportRow> = serde_json::from_slice(&bytes)?;
    let src = LabelService::parse_source(source)
        .ok_or_else(|| Error::Validation(format!("bad source: {source}")))?;
    import_rows(db, src, rows).await
}

async fn import_rows(db: &Db, source: LabelSource, rows: Vec<ImportRow>) -> Result<usize> {
    let repo = LabelRepo::new(db);
    let mut count = 0;
    for r in rows {
        let category = LabelService::parse_category(&r.category)
            .ok_or_else(|| Error::Validation(format!("bad category: {}", r.category)))?;
        let label = AddressLabel {
            address: r.address,
            label: r.label,
            category,
            source,
            confidence: r.confidence.unwrap_or(0.7),
            evidence_url: r.evidence_url,
            verified: matches!(source, LabelSource::Official | LabelSource::Imported),
        };
        repo.upsert(&label).await?;
        count += 1;
    }
    Ok(count)
}
