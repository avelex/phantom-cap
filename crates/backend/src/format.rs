use chrono::{DateTime, Utc};

const SUI_TX_EXPLORER_URL: &str = "https://suivision.xyz/txblock/";
const SUI_CHECKPOINT_EXPLORER_URL: &str = "https://suivision.xyz/checkpoint/";
const SUI_PACKAGE_EXPLORER_URL: &str = "https://suivision.xyz/package/";
const SUI_ADDRESS_EXPLORER_URL: &str = "https://suivision.xyz/account/";
const SUI_OBJECT_EXPLORER_URL: &str = "https://suivision.xyz/object/";

pub fn short_sui_object_id(id: &str) -> String {
    if id.len() > 14 {
        format!("{}...{}", &id[..8], &id[id.len() - 6..])
    } else {
        id.to_string()
    }
}

pub fn sui_tx_url(tx_digest: &str) -> String {
    format!("{}{}", SUI_TX_EXPLORER_URL, tx_digest)
}

pub fn sui_checkpoint_url(checkpoint: &i64) -> String {
    format!("{}{}", SUI_CHECKPOINT_EXPLORER_URL, checkpoint)
}

pub fn sui_package_url(package_id: &str) -> String {
    format!("{}{}", SUI_PACKAGE_EXPLORER_URL, package_id)
}

pub fn sui_address_url(address: &str) -> String {
    format!("{}{}", SUI_ADDRESS_EXPLORER_URL, address)
}

pub fn sui_object_url(object_id: &str) -> String {
    format!("{}{}", SUI_OBJECT_EXPLORER_URL, object_id)
}

pub fn phantom_cap_url(cap_id: &str) -> String {
    format!("/object/{}", cap_id)
}

pub fn phantom_package_url(package_id: &str) -> String {
    format!("/package/{}", package_id)
}

pub fn format_time_ago(timestamp: &DateTime<Utc>, current: &DateTime<Utc>) -> String {
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
