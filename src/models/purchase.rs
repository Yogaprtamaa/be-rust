use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize, FromRow, Clone)]
pub struct Purchase {
    pub id: Uuid,
    pub user_id: Uuid,
    pub batch_id: Uuid,
    pub units: i32,
    pub total_paid_wei: String,
    pub total_paid_eth: Decimal,
    pub tx_hash: Option<String>,
    pub block_number: Option<i64>,
    pub status: String,
    pub created_at: DateTime<Utc>,
}

// Request body dari frontend saat beli
#[derive(Debug, Deserialize)]
pub struct CreatePurchaseRequest {
    pub batch_id: Uuid,
    pub units: i32,
    pub tx_hash: String,        // frontend kirim tx_hash setelah sign di MetaMask
    pub total_paid_wei: String,
}
