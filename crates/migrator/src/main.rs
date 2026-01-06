use diesel::PgConnection;
use diesel::prelude::*;
use diesel_migrations::{EmbeddedMigrations, MigrationHarness, embed_migrations};
use log::info;
use url::Url;

type Db = diesel::pg::Pg;

const MIGRATIONS: EmbeddedMigrations = embed_migrations!("migrations");

fn main() {
    dotenvy::dotenv().ok();
    env_logger::init();

    let database_url = std::env::var("DATABASE_URL")
        .expect("DATABASE_URL must be set")
        .parse::<Url>()
        .expect("Invalid database URL");

    let mut conn =
        PgConnection::establish(database_url.as_str()).expect("Failed to connect to database");

    info!("Running migrations ...");

    run_db_migrations(&mut conn);

    info!("Migrations complete.");
}

fn run_db_migrations(conn: &mut impl MigrationHarness<Db>) {
    conn.run_pending_migrations(MIGRATIONS)
        .expect("Could not run migrations");
}
