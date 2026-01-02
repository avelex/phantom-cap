use actix_files as fs;
use actix_web::web::Html;
use actix_web::{App, HttpServer, Responder, error, get, web};
use askama::Template;
use chrono::{DateTime, Utc};
use diesel::OptionalExtension;
use diesel::{ExpressionMethods, QueryDsl, QueryResult};
use diesel_async::{
    AsyncPgConnection, RunQueryDsl,
    pooled_connection::{AsyncDieselConnectionManager, bb8::Pool},
};
use serde::Deserialize;

use anyhow::Result;
use sui_types::base_types::{ObjectID, SuiAddress};

use crate::schema::upgrade_cap_transfers::dsl as upgrade_cap_transfers_dsl;
use crate::schema::upgrade_cap_versions::dsl as upgrade_cap_versions_dsl;
use crate::schema::upgrade_caps::dsl as upgrade_caps_dsl;

mod models;
mod schema;

const SUI_TX_EXPLORER_URL: &str = "https://suivision.xyz/txblock/";
const SUI_CHECKPOINT_EXPLORER_URL: &str = "https://suivision.xyz/checkpoint/";
const SUI_PACKAGE_EXPLORER_URL: &str = "https://suivision.xyz/package/";
const SUI_ADDRESS_EXPLORER_URL: &str = "https://suivision.xyz/account/";
const SUI_OBJECT_EXPLORER_URL: &str = "https://suivision.xyz/object/";

#[derive(Template)]
#[template(path = "index.html")]
struct HomePage {
    upgrade_caps_count: i64,
    packages_count: i64,
    transfers_count: i64,
}

#[derive(Template, Debug)]
#[template(path = "search.html")]
enum SearchResult {
    Cap(String),
    Package(String),
}

#[derive(Template)]
#[template(path = "upgrade_cap.html")]
struct Cap {
    id: String,
    short_id: String,
    id_url: String,
    package: String,
    package_full: String,
    package_url: String,
    version: String,
    policy: String,
    owner: String,
    owner_full: String,
    owner_url: String,
    created_by: String,
    created_by_full: String,
    created_by_url: String,
    tx_digest_url: String,
    time_ago: String,
}

#[derive(Template)]
#[template(path = "cap_versions.html")]
struct CapVersionsTemplate {
    versions: Vec<CapVersion>,
}

#[derive(Template)]
#[template(path = "cap_transfers.html")]
struct CapTransfersTemplate {
    transfers: Vec<CapTransfer>,
}

#[derive(Template)]
#[template(path = "package.html")]
struct Package {
    id: String,
    short_id: String,
    id_url: String,
    upgrade_cap_id: String,
    upgrade_cap_id_full: String,
    upgrade_cap_id_url: String,
    version: i64,
    published_by: String,
    published_by_full: String,
    published_by_url: String,
    tx_digest_url: String,
    time_ago: String,
}

pub struct CapVersion {
    pub version: i64,
    pub package_id: String,
    pub package_id_full: String,
    pub package_url: String,
    pub tx_digest: String,
    pub tx_digest_full: String,
    pub tx_url: String,
    pub seq_checkpoint: i64,
    pub seq_checkpoint_url: String,
    pub time_ago: String,
}

pub struct CapTransfer {
    pub tx_digest: String,
    pub tx_digest_full: String,
    pub tx_url: String,
    pub seq_checkpoint: i64,
    pub seq_checkpoint_url: String,
    pub time_ago: String,
    pub from: String,
    pub from_full: String,
    pub from_url: String,
    pub to: String,
    pub to_full: String,
    pub to_url: String,
}

#[derive(Deserialize)]
struct SearchQuery {
    id: String,
}

#[get("/")]
async fn home(pool: web::Data<DbPool>) -> actix_web::Result<Html> {
    let mut conn = pool.get().await.map_err(error::ErrorInternalServerError)?;

    let upgrade_caps_count = fetch_upgrade_caps_count(&mut conn).await.unwrap_or(0);
    let packages_count = fetch_packages_count(&mut conn).await.unwrap_or(0);
    let transfers_count = fetch_transfers_count(&mut conn).await.unwrap_or(0);

    Ok(Html::new(
        HomePage {
            upgrade_caps_count,
            packages_count,
            transfers_count,
        }
        .render()
        .unwrap(),
    ))
}

#[get("/search")]
async fn search_cap(
    pool: web::Data<DbPool>,
    query: web::Query<SearchQuery>,
) -> actix_web::Result<Html> {
    let object_id = ObjectID::from_hex_literal(&query.id.clone())
        .map_err(error::ErrorBadRequest)?
        .to_hex_literal();

    let mut conn = pool.get().await.map_err(error::ErrorInternalServerError)?;

    let cap_result = find_upgrade_cap_by_id(&mut conn, &object_id).await;
    let package_result = find_package_by_id(&mut conn, &object_id).await;

    if let Ok(cap) = cap_result {
        return Ok(Html::new(
            SearchResult::Cap(cap.object_id).render().unwrap(),
        ));
    } else if let Ok(package) = package_result {
        return Ok(Html::new(
            SearchResult::Package(package.package_id).render().unwrap(),
        ));
    } else {
        return Ok(Html::new("Not Found"));
    }
}

async fn find_upgrade_cap_by_id(
    conn: &mut AsyncPgConnection,
    id: &String,
) -> QueryResult<models::UpgradeCap> {
    upgrade_caps_dsl::upgrade_caps
        .filter(upgrade_caps_dsl::object_id.eq(&id))
        .first::<models::UpgradeCap>(conn)
        .await
}

async fn find_package_by_id(
    conn: &mut AsyncPgConnection,
    id: &String,
) -> QueryResult<models::UpgradeCapVersion> {
    upgrade_cap_versions_dsl::upgrade_cap_versions
        .filter(upgrade_cap_versions_dsl::package_id.eq(&id))
        .first::<models::UpgradeCapVersion>(conn)
        .await
}

#[get("/cap/{id}")]
async fn show_cap_info(pool: web::Data<DbPool>, id: web::Path<String>) -> actix_web::Result<Html> {
    let object_id = ObjectID::from_hex_literal(&id).map_err(error::ErrorBadRequest)?;
    let mut conn = pool.get().await.map_err(error::ErrorInternalServerError)?;
    let cap = fetch_cap_details(&mut conn, &object_id.to_hex_literal()).await?;
    Ok(Html::new(
        cap.render().map_err(error::ErrorInternalServerError)?,
    ))
}

#[get("/cap/{id}/transfers")]
async fn show_cap_transfers(
    pool: web::Data<DbPool>,
    id: web::Path<String>,
) -> actix_web::Result<Html> {
    let object_id = ObjectID::from_hex_literal(&id).map_err(error::ErrorBadRequest)?;
    let mut conn = pool.get().await.map_err(error::ErrorInternalServerError)?;
    let transfers = fetch_cap_transfers_history(&mut conn, &object_id.to_hex_literal())
        .await
        .unwrap_or(vec![]);

    let now = chrono::Utc::now();
    let transfer_views = transfers
        .iter()
        .map(|t| {
            let time_ago = format_time_ago(&t.timestamp, &now);
            CapTransfer {
                tx_digest: short_sui_object_id(&t.tx_digest),
                tx_digest_full: t.tx_digest.clone(),
                tx_url: sui_tx_url(&t.tx_digest),
                seq_checkpoint: t.seq_checkpoint,
                seq_checkpoint_url: sui_checkpoint_url(&t.seq_checkpoint),
                time_ago,
                from: short_sui_object_id(&t.old_owner_address),
                from_full: t.old_owner_address.clone(),
                from_url: sui_address_url(&t.old_owner_address),
                to: short_sui_object_id(&t.new_owner_address),
                to_full: t.new_owner_address.clone(),
                to_url: sui_address_url(&t.new_owner_address),
            }
        })
        .collect();

    Ok(Html::new(
        CapTransfersTemplate {
            transfers: transfer_views,
        }
        .render()
        .map_err(error::ErrorInternalServerError)?,
    ))
}

#[get("/cap/{id}/versions")]
async fn show_cap_versions(
    pool: web::Data<DbPool>,
    id: web::Path<String>,
) -> actix_web::Result<Html> {
    let object_id = ObjectID::from_hex_literal(&id).map_err(error::ErrorBadRequest)?;
    let mut conn = pool.get().await.map_err(error::ErrorInternalServerError)?;
    let versions = fetch_cap_versions_history(&mut conn, &object_id.to_hex_literal())
        .await
        .unwrap_or(vec![]);

    let now = chrono::Utc::now();
    let version_views = versions
        .iter()
        .map(|v| {
            let time_ago = format_time_ago(&v.timestamp, &now);
            CapVersion {
                version: v.version,
                package_id: short_sui_object_id(&v.package_id),
                package_id_full: v.package_id.clone(),
                package_url: sui_package_url(&v.package_id),
                tx_digest: short_sui_object_id(&v.tx_digest),
                tx_digest_full: v.tx_digest.clone(),
                tx_url: sui_tx_url(&v.tx_digest),
                seq_checkpoint: v.seq_checkpoint,
                seq_checkpoint_url: sui_checkpoint_url(&v.seq_checkpoint),
                time_ago,
            }
        })
        .collect();

    Ok(Html::new(
        CapVersionsTemplate {
            versions: version_views,
        }
        .render()
        .map_err(error::ErrorInternalServerError)?,
    ))
}

#[get("/package/{id}")]
async fn show_package_info(
    pool: web::Data<DbPool>,
    id: web::Path<String>,
) -> actix_web::Result<Html> {
    let object_id = ObjectID::from_hex_literal(&id).map_err(error::ErrorBadRequest)?;
    let mut conn = pool.get().await.map_err(error::ErrorInternalServerError)?;
    let package = fetch_package_details(&mut conn, &object_id.to_hex_literal()).await?;
    Ok(Html::new(
        package.render().map_err(error::ErrorInternalServerError)?,
    ))
}

async fn fetch_package_details(
    conn: &mut AsyncPgConnection,
    id: &str,
) -> Result<Package, actix_web::Error> {
    let package = upgrade_cap_versions_dsl::upgrade_cap_versions
        .filter(upgrade_cap_versions_dsl::package_id.eq(id))
        .first::<models::UpgradeCapVersion>(conn)
        .await
        .map_err(error::ErrorInternalServerError)?;

    Ok(Package {
        id: package.package_id.clone(),
        short_id: short_sui_object_id(&package.package_id),
        id_url: sui_package_url(&package.package_id),
        upgrade_cap_id: short_sui_object_id(&package.object_id),
        upgrade_cap_id_full: package.object_id.clone(),
        upgrade_cap_id_url: phantom_cap_url(&package.object_id),
        version: package.version,
        published_by: short_sui_object_id(&package.publisher),
        published_by_full: package.publisher.clone(),
        published_by_url: sui_address_url(&package.publisher),
        tx_digest_url: sui_tx_url(&package.tx_digest),
        time_ago: format_time_ago(&package.timestamp, &chrono::Utc::now()),
    })
}

async fn fetch_cap_details(
    conn: &mut AsyncPgConnection,
    cap_id: &str,
) -> Result<Cap, actix_web::Error> {
    let cap = upgrade_caps_dsl::upgrade_caps
        .filter(upgrade_caps_dsl::object_id.eq(cap_id))
        .first::<models::UpgradeCap>(conn)
        .await
        .map_err(error::ErrorInternalServerError)?;

    let latest_version = upgrade_cap_versions_dsl::upgrade_cap_versions
        .filter(upgrade_cap_versions_dsl::object_id.eq(cap_id))
        .order(upgrade_cap_versions_dsl::version.desc())
        .first::<models::UpgradeCapVersion>(conn)
        .await
        .optional()
        .map_err(error::ErrorInternalServerError)?;

    let (package_id, version_str) = match latest_version {
        Some(v) => (v.package_id, v.version.to_string()),
        None => ("Unknown".to_string(), "0".to_string()),
    };

    let latest_transfer = upgrade_cap_transfers_dsl::upgrade_cap_transfers
        .filter(upgrade_cap_transfers_dsl::object_id.eq(cap_id))
        .order(upgrade_cap_transfers_dsl::seq_checkpoint.desc())
        .first::<models::UpgradeCapTransfer>(conn)
        .await
        .optional()
        .map_err(error::ErrorInternalServerError)?;

    let owner_address = match latest_transfer {
        Some(t) => t.new_owner_address,
        None => "Unknown".to_string(),
    };

    let first_transfer = upgrade_cap_transfers_dsl::upgrade_cap_transfers
        .filter(upgrade_cap_transfers_dsl::object_id.eq(cap_id))
        .order(upgrade_cap_transfers_dsl::seq_checkpoint.asc())
        .first::<models::UpgradeCapTransfer>(conn)
        .await
        .optional()
        .map_err(error::ErrorInternalServerError)?;

    let created_by = match &first_transfer {
        Some(t) => t.new_owner_address.clone(),
        None => "Unknown".to_string(),
    };

    let created_by_url = match &first_transfer {
        Some(t) => sui_address_url(&t.new_owner_address),
        None => "Unknown".to_string(),
    };

    let policy_str = cap.policy.to_string();
    let now = chrono::Utc::now();
    let time_ago = format_time_ago(&cap.created_at, &now);

    Ok(Cap {
        id: cap.object_id.clone(),
        id_url: sui_object_url(&cap.object_id),
        short_id: short_sui_object_id(&cap.object_id),
        package: short_sui_object_id(&package_id),
        package_full: package_id.clone(),
        package_url: phantom_package_url(&package_id),
        version: version_str,
        policy: policy_str,
        owner: short_sui_object_id(&owner_address),
        owner_full: owner_address.clone(),
        owner_url: sui_address_url(&owner_address),
        created_by: short_sui_object_id(&created_by),
        created_by_full: created_by.clone(),
        created_by_url,
        tx_digest_url: sui_tx_url(&cap.created_tx_digest),
        time_ago,
    })
}

async fn fetch_cap_versions_history(
    conn: &mut AsyncPgConnection,
    cap_id: &str,
) -> QueryResult<Vec<models::UpgradeCapVersion>> {
    upgrade_cap_versions_dsl::upgrade_cap_versions
        .filter(upgrade_cap_versions_dsl::object_id.eq(cap_id))
        .order(upgrade_cap_versions_dsl::seq_checkpoint.desc())
        .load::<models::UpgradeCapVersion>(conn)
        .await
}

async fn fetch_cap_transfers_history(
    conn: &mut AsyncPgConnection,
    cap_id: &str,
) -> QueryResult<Vec<models::UpgradeCapTransfer>> {
    upgrade_cap_transfers_dsl::upgrade_cap_transfers
        .filter(upgrade_cap_transfers_dsl::object_id.eq(cap_id))
        .order(upgrade_cap_transfers_dsl::seq_checkpoint.desc())
        .load::<models::UpgradeCapTransfer>(conn)
        .await
}

async fn fetch_upgrade_caps_count(conn: &mut AsyncPgConnection) -> QueryResult<i64> {
    upgrade_caps_dsl::upgrade_caps
        .count()
        .get_result(conn)
        .await
}

async fn fetch_packages_count(conn: &mut AsyncPgConnection) -> QueryResult<i64> {
    upgrade_cap_versions_dsl::upgrade_cap_versions
        .count()
        .get_result(conn)
        .await
}

async fn fetch_transfers_count(conn: &mut AsyncPgConnection) -> QueryResult<i64> {
    upgrade_cap_transfers_dsl::upgrade_cap_transfers
        .count()
        .get_result(conn)
        .await
}

fn short_sui_object_id(id: &str) -> String {
    if id.len() > 14 {
        format!("{}...{}", &id[..8], &id[id.len() - 6..])
    } else {
        id.to_string()
    }
}

fn sui_tx_url(tx_digest: &str) -> String {
    format!("{}{}", SUI_TX_EXPLORER_URL, tx_digest)
}

fn sui_checkpoint_url(checkpoint: &i64) -> String {
    format!("{}{}", SUI_CHECKPOINT_EXPLORER_URL, checkpoint)
}

fn sui_package_url(package_id: &str) -> String {
    format!("{}{}", SUI_PACKAGE_EXPLORER_URL, package_id)
}

fn sui_address_url(address: &str) -> String {
    format!("{}{}", SUI_ADDRESS_EXPLORER_URL, address)
}

fn sui_object_url(object_id: &str) -> String {
    format!("{}{}", SUI_OBJECT_EXPLORER_URL, object_id)
}

fn phantom_cap_url(cap_id: &str) -> String {
    format!("/cap/{}", cap_id)
}

fn phantom_package_url(package_id: &str) -> String {
    format!("/package/{}", package_id)
}

fn format_time_ago(timestamp: &DateTime<Utc>, current: &DateTime<Utc>) -> String {
    let diff = current.signed_duration_since(timestamp);
    let time_ago = if diff.num_days() > 0 {
        format!("{}d ago", diff.num_days())
    } else if diff.num_hours() > 0 {
        format!("{}h ago", diff.num_hours())
    } else {
        format!("{}m ago", diff.num_minutes())
    };
    time_ago
}

#[derive(Template)]
#[template(path = "not_found.html")]
struct NotFoundTemplate;

async fn not_found() -> actix_web::Result<Html> {
    Ok(Html::new(
        NotFoundTemplate
            .render()
            .map_err(error::ErrorInternalServerError)?,
    ))
}

type DbPool = Pool<AsyncPgConnection>;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let manager = AsyncDieselConnectionManager::<AsyncPgConnection>::new(
        "postgres://postgres:0681d7bf4e0c@localhost:5432/phantom",
    );

    let pool = Pool::builder().build(manager).await.unwrap();

    load_mock_data(&pool).await.unwrap();

    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(pool.clone()))
            .service(home)
            .service(search_cap)
            .service(show_cap_info)
            .service(show_cap_transfers)
            .service(show_cap_versions)
            .service(show_package_info)
            .service(fs::Files::new("/static", "static").show_files_listing())
            .default_service(web::route().to(not_found))
    })
    .bind(("127.0.0.1", 8080))?
    .run()
    .await
}

async fn load_mock_data(pool: &DbPool) -> Result<()> {
    let mut conn = pool
        .get()
        .await
        .expect("couldn't get db connection from pool");

    let date = DateTime::parse_from_rfc3339("2025-11-18T00:54:41+00:00").unwrap();

    let cap = models::UpgradeCap {
        object_id: "0x6906173d537f5a1ac4556bd2653129cff278b9e1567fefe2a97fe754d3162ffb".to_string(),
        policy: models::UpgradeCompatibilityPolicyEnum::Compatible,
        created_at: date.to_utc(),
        created_seq_checkpoint: 213327389,
        created_tx_digest: "8fzRTEaUQNHQmpYXdRcbX1qsn9gvNk6pABPu86koLmfy".to_string(),
    };

    diesel::insert_into(upgrade_caps_dsl::upgrade_caps)
        .values(&cap)
        .on_conflict_do_nothing()
        .execute(&mut conn)
        .await?;

    let version = models::UpgradeCapVersion {
        object_id: cap.object_id.clone(),
        package_id: "0x8b4a56d1811aeaeecfda30975d286200e5f27d11622246b3acf6115399b51592"
            .to_string(),
        version: 1,
        seq_checkpoint: 213327389,
        publisher: "0x066ceb4e01d5dbfda6f6737dd484a9085851624604cae86ac4c4712af7627d24".to_string(),
        tx_digest: "8fzRTEaUQNHQmpYXdRcbX1qsn9gvNk6pABPu86koLmfy".to_string(),
        timestamp: date.to_utc(),
    };

    diesel::insert_into(upgrade_cap_versions_dsl::upgrade_cap_versions)
        .values(&version)
        .on_conflict_do_nothing()
        .execute(&mut conn)
        .await?;

    let transfer = models::UpgradeCapTransfer {
        object_id: cap.object_id.clone(),
        old_owner_address: SuiAddress::ZERO.to_string(),
        new_owner_address: "0x066ceb4e01d5dbfda6f6737dd484a9085851624604cae86ac4c4712af7627d24"
            .to_string(),
        seq_checkpoint: 213327389,
        tx_digest: "8fzRTEaUQNHQmpYXdRcbX1qsn9gvNk6pABPu86koLmfy".to_string(),
        timestamp: date.to_utc(),
    };

    diesel::insert_into(upgrade_cap_transfers_dsl::upgrade_cap_transfers)
        .values(&transfer)
        .on_conflict_do_nothing()
        .execute(&mut conn)
        .await?;

    Ok(())
}
