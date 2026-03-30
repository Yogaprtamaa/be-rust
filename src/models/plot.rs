use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use sqlx::FromRow;

#[derive(Debug, Serialize, Deserialize, FromRow, Clone)]
pub struct Plot {
    pub plot_id: String,
    pub block_id: String,
    pub location_name: String,
    pub asset_type: String,
    pub total_area: Decimal,
    pub total_allocation_capacity: i32,
    pub allocated_capacity: i32,
    pub remaining_capacity: i32,
    pub price_in_tani: Decimal,
    pub price_in_usdt_reference: Option<Decimal>,
    pub status: String,
    pub legal_reference_id: Option<String>,
    pub metadata_uri: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct PlotResponse {
    pub plot_id: String,
    pub block_id: String,
    pub location_name: String,
    pub asset_type: String,
    pub total_area: Decimal,
    pub total_allocation_capacity: i32,
    pub allocated_capacity: i32,
    pub remaining_capacity: i32,
    pub price_in_tani: Decimal,
    pub price_in_usdt_reference: Option<Decimal>,
    pub status: String,
    pub legal_reference_id: Option<String>,
    pub metadata_uri: Option<String>,
}

impl From<Plot> for PlotResponse {
    fn from(value: Plot) -> Self {
        Self {
            plot_id: value.plot_id,
            block_id: value.block_id,
            location_name: value.location_name,
            asset_type: value.asset_type,
            total_area: value.total_area,
            total_allocation_capacity: value.total_allocation_capacity,
            allocated_capacity: value.allocated_capacity,
            remaining_capacity: value.remaining_capacity,
            price_in_tani: value.price_in_tani,
            price_in_usdt_reference: value.price_in_usdt_reference,
            status: value.status,
            legal_reference_id: value.legal_reference_id,
            metadata_uri: value.metadata_uri,
        }
    }
}
