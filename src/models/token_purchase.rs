use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize, FromRow, Clone)]
pub struct TokenPurchase {
    pub purchase_id: Uuid,
    pub wallet_address: String,
    pub usdt_amount: Decimal,
    pub tani_amount: Decimal,
    pub rate_used: Decimal,
    pub tx_hash: Option<String>,
    pub status: String,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Deserialize)]
pub struct CreateTokenPurchaseRequest {
    pub usdt_amount: Decimal,
    pub tx_hash: String,
}

#[derive(Debug, Serialize)]
pub struct TokenPurchasePreview {
    pub usdt_amount: Decimal,
    pub tani_amount: Decimal,
    pub rate: f64,
    pub sale_inventory_balance: u64,
}
