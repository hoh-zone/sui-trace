use sqlx::Row;
use trace_common::{
    Error,
    error::Result,
    model::{AddressLabel, LabelCategory, LabelSource},
};

use crate::Db;

pub struct LabelRepo<'a> {
    db: &'a Db,
}

impl<'a> LabelRepo<'a> {
    pub fn new(db: &'a Db) -> Self {
        Self { db }
    }

    pub async fn upsert(&self, l: &AddressLabel) -> Result<()> {
        sqlx::query(
            r#"
            INSERT INTO address_labels
                (address, label, category, source, confidence, evidence_url, verified)
            VALUES ($1, $2, $3, $4, $5, $6, $7)
            ON CONFLICT (address, label, source) DO UPDATE SET
                category = EXCLUDED.category,
                confidence = EXCLUDED.confidence,
                evidence_url = EXCLUDED.evidence_url,
                verified = EXCLUDED.verified
            "#,
        )
        .bind(&l.address)
        .bind(&l.label)
        .bind(category_to_str(l.category))
        .bind(source_to_str(l.source))
        .bind(l.confidence)
        .bind(&l.evidence_url)
        .bind(l.verified)
        .execute(self.db.pool())
        .await
        .map_err(|e| Error::Database(e.to_string()))?;
        Ok(())
    }

    pub async fn for_address(&self, address: &str) -> Result<Vec<AddressLabel>> {
        let rows = sqlx::query(
            r#"SELECT address, label, category, source, confidence, evidence_url, verified
               FROM address_labels WHERE address = $1"#,
        )
        .bind(address)
        .fetch_all(self.db.pool())
        .await
        .map_err(|e| Error::Database(e.to_string()))?;
        Ok(rows.into_iter().map(row_to_label).collect())
    }

    pub async fn search(&self, term: &str, limit: i64) -> Result<Vec<AddressLabel>> {
        let rows = sqlx::query(
            r#"SELECT address, label, category, source, confidence, evidence_url, verified
               FROM address_labels
               WHERE label ILIKE '%' || $1 || '%' OR address ILIKE '%' || $1 || '%'
               ORDER BY verified DESC, confidence DESC
               LIMIT $2"#,
        )
        .bind(term)
        .bind(limit)
        .fetch_all(self.db.pool())
        .await
        .map_err(|e| Error::Database(e.to_string()))?;
        Ok(rows.into_iter().map(row_to_label).collect())
    }
}

fn row_to_label(r: sqlx::postgres::PgRow) -> AddressLabel {
    AddressLabel {
        address: r.get(0),
        label: r.get(1),
        category: category_from_str(&r.get::<String, _>(2)),
        source: source_from_str(&r.get::<String, _>(3)),
        confidence: r.get(4),
        evidence_url: r.get(5),
        verified: r.get(6),
    }
}

fn category_to_str(c: LabelCategory) -> &'static str {
    use LabelCategory::*;
    match c {
        Exchange => "exchange",
        CexHotwallet => "cex_hotwallet",
        CexColdwallet => "cex_coldwallet",
        MarketMaker => "market_maker",
        VcFund => "vc_fund",
        ProtocolTreasury => "protocol_treasury",
        Bridge => "bridge",
        Validator => "validator",
        Hacker => "hacker",
        Scam => "scam",
        Phishing => "phishing",
        Mixer => "mixer",
        Sanctioned => "sanctioned",
        RugPull => "rug_pull",
        TeamMultisig => "team_multisig",
        Vesting => "vesting",
        AirdropDistributor => "airdrop_distributor",
        Other => "other",
    }
}

fn category_from_str(s: &str) -> LabelCategory {
    use LabelCategory::*;
    match s {
        "exchange" => Exchange,
        "cex_hotwallet" => CexHotwallet,
        "cex_coldwallet" => CexColdwallet,
        "market_maker" => MarketMaker,
        "vc_fund" => VcFund,
        "protocol_treasury" => ProtocolTreasury,
        "bridge" => Bridge,
        "validator" => Validator,
        "hacker" => Hacker,
        "scam" => Scam,
        "phishing" => Phishing,
        "mixer" => Mixer,
        "sanctioned" => Sanctioned,
        "rug_pull" => RugPull,
        "team_multisig" => TeamMultisig,
        "vesting" => Vesting,
        "airdrop_distributor" => AirdropDistributor,
        _ => Other,
    }
}

fn source_to_str(s: LabelSource) -> &'static str {
    use LabelSource::*;
    match s {
        Official => "official",
        Community => "community",
        Oracle => "oracle",
        Heuristic => "heuristic",
        Imported => "imported",
    }
}

fn source_from_str(s: &str) -> LabelSource {
    use LabelSource::*;
    match s {
        "official" => Official,
        "community" => Community,
        "oracle" => Oracle,
        "heuristic" => Heuristic,
        _ => Imported,
    }
}
