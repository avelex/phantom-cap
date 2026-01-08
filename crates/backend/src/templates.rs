use askama::Template;

#[derive(Template)]
#[template(path = "not_found.html")]
pub struct NotFound;

#[derive(Template)]
#[template(path = "index.html")]
pub struct HomePage {
    pub upgrade_caps_count: i64,
    pub packages_count: i64,
    pub transfers_count: i64,
}

#[derive(Template, Debug)]
#[template(path = "search.html")]
pub enum SearchResult {
    Cap(String),
    Package(String),
}

#[derive(Template)]
#[template(path = "upgrade_cap.html")]
pub struct Cap {
    pub id: String,
    pub short_id: String,
    pub package: String,
    pub package_full: String,
    pub package_url: String,
    pub version: String,
    pub policy: String,
    pub owner: String,
    pub owner_full: String,
    pub owner_url: String,
    pub created_by: String,
    pub created_by_full: String,
    pub created_by_url: String,
    pub tx_digest_url: String,
    pub time_ago: String,
}

#[derive(Template)]
#[template(path = "cap_versions.html")]
pub struct CapVersions {
    pub versions: Vec<CapVersion>,
}

#[derive(Template)]
#[template(path = "cap_transfers.html")]
pub struct CapTransfers {
    pub transfers: Vec<CapTransfer>,
}

#[derive(Template)]
#[template(path = "package.html")]
pub struct Package {
    pub id: String,
    pub short_id: String,
    // id_url: String,
    pub upgrade_cap_id: String,
    pub upgrade_cap_id_full: String,
    pub upgrade_cap_id_url: String,
    pub version: i64,
    pub published_by: String,
    pub published_by_full: String,
    pub published_by_url: String,
    pub tx_digest_url: String,
    pub time_ago: String,
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
