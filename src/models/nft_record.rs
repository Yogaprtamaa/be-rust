use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize, FromRow, Clone)]
pub struct NftRecord {
    pub nft_id: String,
    pub wallet_address: String,
    pub plot_id: String,
    pub allocation_id: Option<Uuid>,
    pub metadata_uri: Option<String>,
    pub legal_reference_id: Option<String>,
    pub mint_tx_hash: Option<String>,
    pub status: String,
    pub minted_at: DateTime<Utc>,
}
