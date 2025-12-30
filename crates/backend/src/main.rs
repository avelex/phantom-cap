use actix_web::web::Html;
use actix_web::{App, HttpServer, Responder, error, get, web};
use askama::Template;
use diesel::OptionalExtension;
use diesel::{ExpressionMethods, QueryDsl, QueryResult};
use diesel_async::{
    AsyncPgConnection, RunQueryDsl,
    pooled_connection::{
        AsyncDieselConnectionManager,
        bb8::{Pool, PooledConnection},
    },
};
use serde::Deserialize;

use anyhow::Result;
use sui_types::base_types::SuiAddress;

use crate::schema::upgrade_cap_transfers::dsl as upgrade_cap_transfers_dsl;
use crate::schema::upgrade_cap_versions::dsl as upgrade_cap_versions_dsl;
use crate::schema::upgrade_caps::dsl as upgrade_caps_dsl;

mod models;
mod schema;

#[derive(Template)]
#[template(path = "index.html")]
struct HomePage;

#[derive(Template)]
#[template(path = "search.html")]
struct SearchResult {
    id: String,
}

#[derive(Template)]
#[template(path = "upgrade_cap.html")]
struct UpgradeCap {
    id: String,
    package: String,
    version: String,
    policy: String,
    owner: String,
    created_by: String,
    time_ago: String,
}

#[derive(Deserialize)]
struct SearchQuery {
    id: String,
}

#[get("/")]
async fn home() -> impl Responder {
    Html::new(HomePage.render().unwrap())
}

#[get("/cap/search")]
async fn search_cap(
    pool: web::Data<DbPool>,
    query: web::Query<SearchQuery>,
) -> actix_web::Result<Html> {
    let cap_id = query.id.clone();

    let mut conn = pool
        .get()
        .await
        .expect("couldn't get db connection from pool");

    let cap = find_upgrade_cap_by_id(&mut conn, &cap_id).await;

    match cap {
        Ok(cap) => Ok(Html::new(
            SearchResult { id: cap.object_id }.render().unwrap(),
        )),
        Err(_) => Ok(Html::new("Not Found")),
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

#[get("/cap/{id}")]
async fn show_cap_info(pool: web::Data<DbPool>, id: web::Path<String>) -> actix_web::Result<Html> {
    let mut conn = pool.get().await.map_err(error::ErrorInternalServerError)?;
    let cap = fetch_cap_details(&mut conn, &id).await?;
    Ok(Html::new(
        cap.render().map_err(error::ErrorInternalServerError)?,
    ))
}

#[get("/cap/{id}/transfers")]
async fn show_cap_transfers(
    pool: web::Data<DbPool>,
    id: web::Path<String>,
) -> actix_web::Result<Html> {
    let mut conn = pool.get().await.map_err(error::ErrorInternalServerError)?;
    let cap = fetch_cap_details(&mut conn, &id).await?;
    Ok(Html::new(
        cap.render().map_err(error::ErrorInternalServerError)?,
    ))
}

#[get("/cap/{id}/versions")]
async fn show_cap_versions(
    pool: web::Data<DbPool>,
    id: web::Path<String>,
) -> actix_web::Result<Html> {
    let mut conn = pool.get().await.map_err(error::ErrorInternalServerError)?;
    let cap = fetch_cap_details(&mut conn, &id).await?;
    Ok(Html::new(
        cap.render().map_err(error::ErrorInternalServerError)?,
    ))
}

async fn fetch_cap_details(
    conn: &mut AsyncPgConnection,
    cap_id: &str,
) -> Result<UpgradeCap, actix_web::Error> {
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

    let (package, version_str) = match latest_version {
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

    let owner = match latest_transfer {
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

    let created_by = match first_transfer {
        Some(t) => t.new_owner_address,
        None => "Unknown".to_string(),
    };

    let policy_str = match cap.policy {
        models::UpgradeCompatibilityPolicyEnum::Compatible => "Compatible",
        models::UpgradeCompatibilityPolicyEnum::Additive => "Additive",
        models::UpgradeCompatibilityPolicyEnum::DepOnly => "DepOnly",
        models::UpgradeCompatibilityPolicyEnum::Immutable => "Immutable",
    }
    .to_string();

    let now = chrono::Utc::now();
    let diff = now.signed_duration_since(cap.created_at);

    let time_ago = if diff.num_days() > 0 {
        format!("{}d ago", diff.num_days())
    } else if diff.num_hours() > 0 {
        format!("{}h ago", diff.num_hours())
    } else {
        format!("{}m ago", diff.num_minutes())
    };

    Ok(UpgradeCap {
        id: cap.object_id,
        package,
        version: version_str,
        policy: policy_str,
        owner,
        created_by,
        time_ago,
    })
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

    let cap = models::UpgradeCap {
        object_id: "0xA".to_string(),
        policy: models::UpgradeCompatibilityPolicyEnum::Compatible,
        created_at: chrono::Utc::now(),
        created_seq_checkpoint: 1,
        created_tx_digest: "tx_digest_1".to_string(),
    };

    diesel::insert_into(upgrade_caps_dsl::upgrade_caps)
        .values(&cap)
        .on_conflict_do_nothing()
        .execute(&mut conn)
        .await?;

    let version = models::UpgradeCapVersion {
        object_id: cap.object_id.clone(),
        package_id: "0xP".to_string(),
        version: 1,
        seq_checkpoint: 2,
        tx_digest: "tx_digest_2".to_string(),
        timestamp: chrono::Utc::now(),
    };

    diesel::insert_into(upgrade_cap_versions_dsl::upgrade_cap_versions)
        .values(&version)
        .on_conflict_do_nothing()
        .execute(&mut conn)
        .await?;

    let transfer = models::UpgradeCapTransfer {
        object_id: cap.object_id.clone(),
        old_owner_address: SuiAddress::ZERO.to_string(),
        new_owner_address: "0xNEW".to_string(),
        seq_checkpoint: 3,
        tx_digest: "tx_digest_3".to_string(),
        timestamp: chrono::Utc::now(),
    };

    diesel::insert_into(upgrade_cap_transfers_dsl::upgrade_cap_transfers)
        .values(&transfer)
        .on_conflict_do_nothing()
        .execute(&mut conn)
        .await?;

    Ok(())
}
