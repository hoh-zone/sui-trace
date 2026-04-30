//! GraphQL surface — mounted alongside REST under `/graphql` and
//! `/graphql/playground`.

use async_graphql::{EmptyMutation, EmptySubscription, Object, Schema};
use chrono::{DateTime, Duration, Utc};
use trace_common::model::AddressLabel;
use trace_storage::repo::{
    analytics::AnalyticsRepo, packages::PackageRepo, security::SecurityRepo,
    transactions::TransactionRepo, tvl::TvlRepo,
};

use crate::state::AppState;

pub type AppSchema = Schema<QueryRoot, EmptyMutation, EmptySubscription>;

pub fn build_schema(state: AppState) -> AppSchema {
    Schema::build(QueryRoot, EmptyMutation, EmptySubscription)
        .data(state)
        .finish()
}

pub struct QueryRoot;

#[Object]
impl QueryRoot {
    async fn version(&self) -> &'static str {
        env!("CARGO_PKG_VERSION")
    }

    async fn transaction(
        &self,
        ctx: &async_graphql::Context<'_>,
        digest: String,
    ) -> async_graphql::Result<Option<TxView>> {
        let state = ctx.data::<AppState>()?;
        let tx = TransactionRepo::new(&state.db).get(&digest).await?;
        Ok(tx.map(|t| TxView {
            digest: t.digest,
            sender: t.sender,
            status: format!("{:?}", t.status),
            gas_used: t.gas_used as i64,
            timestamp: t.timestamp,
        }))
    }

    async fn package(
        &self,
        ctx: &async_graphql::Context<'_>,
        id: String,
    ) -> async_graphql::Result<Option<PackageView>> {
        let state = ctx.data::<AppState>()?;
        let pkg = PackageRepo::new(&state.db).get(&id).await?;
        Ok(pkg.map(|p| PackageView {
            id: p.id,
            version: p.version as i64,
            publisher: p.publisher,
            modules_count: p.modules_count as i32,
            published_at: p.published_at,
        }))
    }

    async fn security_report(
        &self,
        ctx: &async_graphql::Context<'_>,
        package_id: String,
    ) -> async_graphql::Result<Option<SecurityReportView>> {
        let state = ctx.data::<AppState>()?;
        let report = SecurityRepo::new(&state.db).get_report(&package_id).await?;
        Ok(report.map(|r| SecurityReportView {
            package_id: r.package_id,
            score: r.score as f64,
            findings_count: r.findings.len() as i32,
            max_severity: format!("{:?}", r.max_severity).to_lowercase(),
            scanned_at: r.scanned_at,
        }))
    }

    async fn labels(
        &self,
        ctx: &async_graphql::Context<'_>,
        address: String,
    ) -> async_graphql::Result<Vec<LabelView>> {
        let state = ctx.data::<AppState>()?;
        let labels = state.labels.lookup(&address).await?;
        Ok(labels.into_iter().map(LabelView::from).collect())
    }

    async fn daily_deploys(
        &self,
        ctx: &async_graphql::Context<'_>,
        days: Option<i32>,
    ) -> async_graphql::Result<Vec<DailyDeployView>> {
        let state = ctx.data::<AppState>()?;
        let to = Utc::now();
        let from = to - Duration::days(days.unwrap_or(30) as i64);
        let stats = AnalyticsRepo::new(&state.db)
            .daily_deploys(from, to)
            .await?;
        Ok(stats
            .into_iter()
            .map(|s| DailyDeployView {
                day: s.day.to_string(),
                package_count: s.package_count,
                unique_publishers: s.unique_publishers,
            })
            .collect())
    }

    async fn active_projects(
        &self,
        ctx: &async_graphql::Context<'_>,
        hours: Option<i32>,
        limit: Option<i32>,
    ) -> async_graphql::Result<Vec<ProjectRankingView>> {
        let state = ctx.data::<AppState>()?;
        let since = Utc::now() - Duration::hours(hours.unwrap_or(24) as i64);
        let rows = AnalyticsRepo::new(&state.db)
            .active_packages(since, limit.unwrap_or(50) as i64)
            .await?;
        Ok(rows
            .into_iter()
            .map(|r| ProjectRankingView {
                package_id: r.package_id,
                calls: r.calls,
                unique_callers: r.unique_callers,
                gas_total: r.gas_total,
            })
            .collect())
    }

    async fn tvl_history(
        &self,
        ctx: &async_graphql::Context<'_>,
        protocol_id: String,
        hours: Option<i32>,
    ) -> async_graphql::Result<Vec<TvlPointView>> {
        let state = ctx.data::<AppState>()?;
        let to = Utc::now();
        let from = to - Duration::hours(hours.unwrap_or(24) as i64);
        let points = TvlRepo::new(&state.db)
            .history(&protocol_id, from, to)
            .await?;
        Ok(points
            .into_iter()
            .map(|p| TvlPointView {
                timestamp: p.timestamp,
                tvl_usd: p.tvl_usd,
            })
            .collect())
    }
}

#[derive(async_graphql::SimpleObject)]
pub struct TxView {
    pub digest: String,
    pub sender: String,
    pub status: String,
    pub gas_used: i64,
    pub timestamp: DateTime<Utc>,
}

#[derive(async_graphql::SimpleObject)]
pub struct PackageView {
    pub id: String,
    pub version: i64,
    pub publisher: String,
    pub modules_count: i32,
    pub published_at: DateTime<Utc>,
}

#[derive(async_graphql::SimpleObject)]
pub struct SecurityReportView {
    pub package_id: String,
    pub score: f64,
    pub max_severity: String,
    pub findings_count: i32,
    pub scanned_at: DateTime<Utc>,
}

#[derive(async_graphql::SimpleObject)]
pub struct LabelView {
    pub address: String,
    pub label: String,
    pub category: String,
    pub source: String,
    pub confidence: f64,
    pub verified: bool,
}

impl From<AddressLabel> for LabelView {
    fn from(l: AddressLabel) -> Self {
        Self {
            address: l.address,
            label: l.label,
            category: format!("{:?}", l.category).to_lowercase(),
            source: format!("{:?}", l.source).to_lowercase(),
            confidence: l.confidence as f64,
            verified: l.verified,
        }
    }
}

#[derive(async_graphql::SimpleObject)]
pub struct DailyDeployView {
    pub day: String,
    pub package_count: i64,
    pub unique_publishers: i64,
}

#[derive(async_graphql::SimpleObject)]
pub struct ProjectRankingView {
    pub package_id: String,
    pub calls: i64,
    pub unique_callers: i64,
    pub gas_total: i64,
}

#[derive(async_graphql::SimpleObject)]
pub struct TvlPointView {
    pub timestamp: DateTime<Utc>,
    pub tvl_usd: f64,
}
