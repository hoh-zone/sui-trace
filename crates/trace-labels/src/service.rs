use trace_common::{
    error::Result,
    model::{AddressLabel, LabelCategory, LabelSource},
};
use trace_storage::Db;
use trace_storage::repo::labels::LabelRepo;

#[derive(Clone)]
pub struct LabelService {
    db: Db,
}

impl LabelService {
    pub fn new(db: Db) -> Self {
        Self { db }
    }

    pub async fn lookup(&self, address: &str) -> Result<Vec<AddressLabel>> {
        LabelRepo::new(&self.db).for_address(address).await
    }

    pub async fn submit(&self, label: AddressLabel) -> Result<()> {
        LabelRepo::new(&self.db).upsert(&label).await
    }

    pub async fn search(&self, term: &str, limit: i64) -> Result<Vec<AddressLabel>> {
        LabelRepo::new(&self.db).search(term, limit).await
    }

    pub async fn is_risky(&self, address: &str) -> Result<bool> {
        let labels = self.lookup(address).await?;
        Ok(labels.iter().any(|l| l.category.is_risky()))
    }

    pub fn parse_category(s: &str) -> Option<LabelCategory> {
        use LabelCategory::*;
        Some(match s {
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
            "other" => Other,
            _ => return None,
        })
    }

    pub fn parse_source(s: &str) -> Option<LabelSource> {
        use LabelSource::*;
        Some(match s {
            "official" => Official,
            "community" => Community,
            "oracle" => Oracle,
            "heuristic" => Heuristic,
            "imported" => Imported,
            _ => return None,
        })
    }
}
