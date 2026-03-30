use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize, FromRow, Clone)]
pub struct Batch {
    pub id: Uuid,
    pub contract_batch_id: Option<i32>,
    pub name: String,
    pub location: String,
    pub commodity: String,
    pub area_hectares: Decimal,
    pub total_units: i32,
    pub sold_units: i32,
    pub price_per_unit_wei: String,
    pub price_per_unit_eth: Decimal,
    pub status: String,
    pub description: Option<String>,
    pub image_url: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

// Response yang dikirim ke frontend (tambah field kalkulasi)
#[derive(Debug, Serialize)]
pub struct BatchResponse {
    #[serde(flatten)]
    pub batch: Batch,
    pub available_units: i32,
    pub sold_percentage: f64,
}

impl BatchResponse {
    pub fn from(batch: Batch) -> Self {
        let available = batch.total_units - batch.sold_units;
        let pct = if batch.total_units > 0 {
            (batch.sold_units as f64 / batch.total_units as f64) * 100.0
        } else {
            0.0
        };
        BatchResponse {
            batch,
            available_units: available,
            sold_percentage: pct,
        }
    }
}
