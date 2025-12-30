mod handlers;
mod models;
mod schema;

use handlers::{
    created::UpgradeCapHandler as CreatedHandler, transfer::UpgradeCapHandler as TransferHandler,
    upgrade::UpgradeCapHandler as UpgradeHandler,
};

use anyhow::Result;
use clap::Parser;
use diesel_migrations::{EmbeddedMigrations, embed_migrations};
use sui_indexer_alt_framework::{
    cluster::{Args, IndexerCluster},
    ingestion::{
        ClientArgs, ingestion_client::IngestionClientArgs, streaming_client::StreamingClientArgs,
    },
    pipeline::sequential::SequentialConfig,
};
use tokio;
use url::Url;

const MIGRATIONS: EmbeddedMigrations = embed_migrations!("migrations");

#[tokio::main]
async fn main() -> Result<()> {
    dotenvy::dotenv().ok();

    let database_url = std::env::var("DATABASE_URL")
        .expect("DATABASE_URL must be set")
        .parse::<Url>()
        .expect("Invalid database URL");

    let args = Args::try_parse().expect("Failed to parse arguments");

    let remote_store_url = std::env::var("REMOTE_STORE_URL")
        .expect("REMOTE_STORE_URL must be set")
        .parse::<Url>()
        .expect("Invalid remote store URL");

    let mut ingestion_client_args = IngestionClientArgs::default();
    ingestion_client_args.remote_store_url = Some(remote_store_url.clone());

    let client_args = ClientArgs {
        ingestion: ingestion_client_args,
        streaming: StreamingClientArgs::default(),
    };

    let mut cluster = IndexerCluster::builder()
        .with_args(args)
        .with_database_url(database_url)
        .with_migrations(&MIGRATIONS)
        .with_client_args(client_args)
        .build()
        .await
        .expect("Failed to build indexer cluster");

    cluster
        .sequential_pipeline(CreatedHandler, SequentialConfig::default())
        .await?;

    cluster
        .sequential_pipeline(TransferHandler, SequentialConfig::default())
        .await?;

    cluster
        .sequential_pipeline(UpgradeHandler, SequentialConfig::default())
        .await?;

    println!("Running Sequential Indexer");

    let handle = cluster.run().await?;
    handle.await?;

    Ok(())
}
