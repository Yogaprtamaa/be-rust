use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize, FromRow, Clone)]
pub struct Allocation {
    pub allocation_id: Uuid,
    pub wallet_address: String,
    pub plot_id: String,
    pub allocation_quantity: i32,
    pub tani_spent: Decimal,
    pub treasury_amount: Decimal,
    pub burn_amount: Decimal,
    pub nft_id: Option<String>,
    pub tx_hash: Option<String>,
    pub batch_id: Option<Uuid>,
    pub refund_tx_hash: Option<String>,
    pub status: String,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Deserialize)]
pub struct CreateAllocationRequest {
    pub plot_id: String,
    pub allocation_quantity: i32,
    pub tx_hash: String,
}

#[derive(Debug, Serialize)]
pub struct AllocationPreview {
    pub plot_id: String,
    pub allocation_quantity: i32,
    pub tani_required: Decimal,
    pub treasury_amount: Decimal,
    pub burn_amount: Decimal,
    pub treasury_percentage: f64,
    pub burn_percentage: f64,
}
