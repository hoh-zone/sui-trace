use std::net::SocketAddr;

use anyhow::Result;
use async_graphql::http::GraphiQLSource;
use async_graphql_axum::{GraphQLRequest, GraphQLResponse};
use axum::{Extension, Router, response::Html, routing::get};
use trace_api::AppState;
use trace_api::graphql::{AppSchema, build_schema};
use trace_api::routes::router;
use trace_common::config::AppConfig;

#[tokio::main(flavor = "multi_thread")]
async fn main() -> Result<()> {
    trace_common::telemetry::init("trace-api");
    let cfg_path = std::env::var("TRACE_CONFIG").unwrap_or_else(|_| "config/default.toml".into());
    let cfg = AppConfig::load(&cfg_path)?;
    let bind: SocketAddr = cfg.api.bind.parse()?;

    let state = AppState::new(cfg).await?;
    state.db.health().await?;

    let schema = build_schema(state.clone());
    // GraphQL routes don't need state since the schema embeds it.
    let gql: Router = Router::new()
        .route("/graphql", get(graphiql).post(graphql_handler))
        .layer(Extension(schema));

    let app = router(state).merge(gql);

    tracing::info!(%bind, "trace-api listening");
    let listener = tokio::net::TcpListener::bind(bind).await?;
    axum::serve(listener, app).await?;
    Ok(())
}

async fn graphql_handler(
    Extension(schema): Extension<AppSchema>,
    req: GraphQLRequest,
) -> GraphQLResponse {
    schema.execute(req.into_inner()).await.into()
}

async fn graphiql() -> Html<String> {
    Html(GraphiQLSource::build().endpoint("/graphql").finish())
}
