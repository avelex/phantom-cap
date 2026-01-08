use actix_web::web::Html;
use actix_web::{error, get, web};
use askama::Template;
use serde::Deserialize;

use anyhow;
use sui_types::base_types::{ObjectID, SuiAddress};

use diesel_async::{AsyncPgConnection, pooled_connection::bb8::Pool};

use crate::format;
use crate::query;
use crate::templates;

type DbPool = Pool<AsyncPgConnection>;

#[get("/")]
pub async fn home(pool: web::Data<DbPool>) -> actix_web::Result<Html> {
    let mut conn = pool.get().await.map_err(error::ErrorInternalServerError)?;

    let upgrade_caps_count = query::get_upgrade_caps_count(&mut conn).await.unwrap_or(0);
    let packages_count = query::get_packages_count(&mut conn).await.unwrap_or(0);
    let transfers_count = query::get_transfers_count(&mut conn).await.unwrap_or(0);

    Ok(Html::new(
        templates::HomePage {
            upgrade_caps_count,
            packages_count,
            transfers_count,
        }
        .render()
        .unwrap(),
    ))
}

#[derive(Deserialize)]
struct SearchQuery {
    id: String,
}

#[get("/search")]
pub async fn search_cap(
    pool: web::Data<DbPool>,
    query: web::Query<SearchQuery>,
) -> actix_web::Result<Html> {
    let object_id = ObjectID::from_hex_literal(&query.id.clone())
        .map_err(error::ErrorBadRequest)?
        .to_hex_literal();

    let mut conn = pool.get().await.map_err(error::ErrorInternalServerError)?;

    if let Ok(cap_result) = query::get_cap_by_id(&mut conn, &object_id).await {
        return Ok(Html::new(
            templates::SearchResult::Cap(cap_result.object_id)
                .render()
                .unwrap(),
        ));
    }

    if let Ok(package_result) = query::get_package_by_id(&mut conn, &object_id).await {
        return Ok(Html::new(
            templates::SearchResult::Package(package_result.package_id)
                .render()
                .unwrap(),
        ));
    }

    Ok(Html::new("Not Found"))
}

#[get("/cap/{id}")]
pub async fn show_cap_info(
    pool: web::Data<DbPool>,
    id: web::Path<String>,
) -> actix_web::Result<Html> {
    let object_id = ObjectID::from_hex_literal(&id).map_err(error::ErrorBadRequest)?;
    let mut conn = pool.get().await.map_err(error::ErrorInternalServerError)?;
    let cap = fetch_cap_details(&mut conn, &object_id.to_hex_literal()).await?;
    Ok(Html::new(
        cap.render().map_err(error::ErrorInternalServerError)?,
    ))
}

#[get("/cap/{id}/transfers")]
pub async fn show_cap_transfers(
    pool: web::Data<DbPool>,
    id: web::Path<String>,
) -> actix_web::Result<Html> {
    let object_id = ObjectID::from_hex_literal(&id).map_err(error::ErrorBadRequest)?;
    let mut conn = pool.get().await.map_err(error::ErrorInternalServerError)?;
    let transfers = query::get_cap_transfers_history(&mut conn, &object_id.to_hex_literal())
        .await
        .unwrap_or(vec![]);

    let now = chrono::Utc::now();
    let transfer_views = transfers
        .iter()
        .map(|t| {
            let time_ago = format::format_time_ago(&t.timestamp, &now);
            templates::CapTransfer {
                tx_digest: format::short_sui_object_id(&t.tx_digest),
                tx_digest_full: t.tx_digest.clone(),
                tx_url: format::sui_tx_url(&t.tx_digest),
                seq_checkpoint: t.seq_checkpoint,
                seq_checkpoint_url: format::sui_checkpoint_url(&t.seq_checkpoint),
                time_ago,
                from: format::short_sui_object_id(&t.old_owner_address),
                from_full: t.old_owner_address.clone(),
                from_url: format::sui_address_url(&t.old_owner_address),
                to: format::short_sui_object_id(&t.new_owner_address),
                to_full: t.new_owner_address.clone(),
                to_url: format::sui_address_url(&t.new_owner_address),
            }
        })
        .collect();

    Ok(Html::new(
        templates::CapTransfers {
            transfers: transfer_views,
        }
        .render()
        .map_err(error::ErrorInternalServerError)?,
    ))
}

#[get("/cap/{id}/versions")]
pub async fn show_cap_versions(
    pool: web::Data<DbPool>,
    id: web::Path<String>,
) -> actix_web::Result<Html> {
    let object_id = ObjectID::from_hex_literal(&id).map_err(error::ErrorBadRequest)?;
    let mut conn = pool.get().await.map_err(error::ErrorInternalServerError)?;
    let versions = query::get_cap_versions_history(&mut conn, &object_id.to_hex_literal())
        .await
        .unwrap_or(vec![]);

    let now = chrono::Utc::now();
    let version_views = versions
        .iter()
        .map(|v| templates::CapVersion {
            version: v.version,
            package_id: format::short_sui_object_id(&v.package_id),
            package_id_full: v.package_id.clone(),
            package_url: format::sui_package_url(&v.package_id),
            tx_digest: format::short_sui_object_id(&v.tx_digest),
            tx_digest_full: v.tx_digest.clone(),
            tx_url: format::sui_tx_url(&v.tx_digest),
            seq_checkpoint: v.seq_checkpoint,
            seq_checkpoint_url: format::sui_checkpoint_url(&v.seq_checkpoint),
            time_ago: format::format_time_ago(&v.timestamp, &now),
        })
        .collect();

    Ok(Html::new(
        templates::CapVersions {
            versions: version_views,
        }
        .render()
        .map_err(error::ErrorInternalServerError)?,
    ))
}

#[get("/package/{id}")]
pub async fn show_package_info(
    pool: web::Data<DbPool>,
    id: web::Path<String>,
) -> actix_web::Result<Html> {
    let object_id = ObjectID::from_hex_literal(&id).map_err(error::ErrorBadRequest)?;
    let mut conn = pool.get().await.map_err(error::ErrorInternalServerError)?;

    let package = query::get_package_by_id(&mut conn, &object_id.to_hex_literal())
        .await
        .map(|p| templates::Package {
            id: p.package_id.clone(),
            short_id: format::short_sui_object_id(&p.package_id),
            // id_url: sui_package_url(&package.package_id),
            upgrade_cap_id: format::short_sui_object_id(&p.object_id),
            upgrade_cap_id_full: p.object_id.clone(),
            upgrade_cap_id_url: format::phantom_cap_url(&p.object_id),
            version: p.version,
            published_by: format::short_sui_object_id(&p.publisher),
            published_by_full: p.publisher.clone(),
            published_by_url: format::sui_address_url(&p.publisher),
            tx_digest_url: format::sui_tx_url(&p.tx_digest),
            time_ago: format::format_time_ago(&p.timestamp, &chrono::Utc::now()),
        })
        .map_err(error::ErrorInternalServerError)?;

    Ok(Html::new(
        package.render().map_err(error::ErrorInternalServerError)?,
    ))
}

pub async fn not_found() -> actix_web::Result<Html> {
    Ok(Html::new(
        templates::NotFound
            .render()
            .map_err(error::ErrorInternalServerError)?,
    ))
}

async fn fetch_cap_details(
    conn: &mut AsyncPgConnection,
    cap_id: &str,
) -> anyhow::Result<templates::Cap, actix_web::Error> {
    let cap = query::get_cap_by_id(conn, cap_id)
        .await
        .map_err(error::ErrorInternalServerError)?;

    let latest_version = query::get_cap_latest_version(conn, cap_id)
        .await
        .map_or(("Unknown".to_string(), 0), |c| (c.package_id, c.version));

    let owner_address = query::get_cap_latest_transfer(conn, cap_id)
        .await
        .map_or(SuiAddress::ZERO.to_string(), |t| t.new_owner_address);

    let created_by = query::get_cap_first_transfer(conn, cap_id)
        .await
        .map_or(SuiAddress::ZERO.to_string(), |t| t.new_owner_address);

    let created_by_url = format::sui_address_url(&created_by);

    let policy_str = cap.policy.to_string();
    let now = chrono::Utc::now();
    let time_ago = format::format_time_ago(&cap.created_at, &now);
    let package_id = latest_version.0;
    let version_str = latest_version.1.to_string();

    Ok(templates::Cap {
        id: cap.object_id.clone(),
        short_id: format::short_sui_object_id(&cap.object_id),
        package: format::short_sui_object_id(&package_id),
        package_full: package_id.clone(),
        package_url: format::phantom_package_url(&package_id),
        version: version_str,
        policy: policy_str,
        owner: format::short_sui_object_id(&owner_address),
        owner_full: owner_address.clone(),
        owner_url: format::sui_address_url(&owner_address),
        created_by: format::short_sui_object_id(&created_by),
        created_by_full: created_by.clone(),
        created_by_url,
        tx_digest_url: format::sui_tx_url(&cap.created_tx_digest),
        time_ago,
    })
}
