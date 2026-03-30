use axum::{
    extract::{Path, Query, State},
    routing::get,
    Json, Router,
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use uuid::Uuid;
use crate::{
    errors::{AppError, AppResult},
    models::batch::BatchResponse,
    blockchain::solana_client::SolanaClient,
    AppState,
};

#[derive(Deserialize)]
pub struct BatchFilter {
    pub status: Option<String>,
}

#[derive(Serialize)]
pub struct OnChainBatch {
    pub batch_id: u64,
    pub total_units: u64,
    pub sold_units: u64,
    pub available_units: u64,
    pub price_per_unit_lamports: u64,
    pub price_per_unit_sol: f64,
    pub is_active: bool,
}

pub fn router() -> Router<Arc<AppState>> {
    Router::new()
        .route("/batches", get(get_all_batches))
        .route("/batches/:id", get(get_batch_by_id))
        .route("/batches/onchain/:batch_id", get(get_batch_onchain))
}

async fn get_all_batches(
    State(state): State<Arc<AppState>>,
    Query(filter): Query<BatchFilter>,
) -> AppResult<Json<Vec<BatchResponse>>> {
    let status = filter.status.unwrap_or("open".to_string());

    let batches = sqlx::query_as!(
        crate::models::batch::Batch,
        "SELECT * FROM batches WHERE status = $1 ORDER BY created_at DESC",
        status
    )
    .fetch_all(&state.db)
    .await?;

    Ok(Json(batches.into_iter().map(BatchResponse::from).collect()))
}

async fn get_batch_by_id(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
) -> AppResult<Json<BatchResponse>> {
    let batch = sqlx::query_as!(
        crate::models::batch::Batch,
        "SELECT * FROM batches WHERE id = $1",
        id
    )
    .fetch_optional(&state.db)
    .await?
    .ok_or_else(|| AppError::NotFound(format!("Batch {} tidak ditemukan", id)))?;

    Ok(Json(BatchResponse::from(batch)))
}

async fn get_batch_onchain(
    State(state): State<Arc<AppState>>,
    Path(batch_id): Path<u64>,
) -> AppResult<Json<OnChainBatch>> {
    // Derive PDA dari batch_id
    let pda = derive_batch_pda(batch_id, &state.config.program_id);

    let client = SolanaClient::new(
        &state.config.solana_rpc_url,
        &state.config.program_id,
    );

    let account = client
        .get_account_info(&pda)
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?
        .ok_or_else(|| AppError::NotFound(format!("Batch {} tidak ada on-chain", batch_id)))?;

    // Parse data account (skip 8 bytes discriminator)
    let data_b64 = account["data"][0]
        .as_str()
        .ok_or_else(|| AppError::Internal("Data tidak valid".to_string()))?;

    let data = base64::decode(data_b64)
        .map_err(|e| AppError::Internal(e.to_string()))?;

    if data.len() < 8 + 8 + 32 + 8 + 8 + 8 + 1 + 1 {
        return Err(AppError::Internal("Data account terlalu pendek".to_string()));
    }

    // Parse manual dari bytes (little-endian)
    let offset = 8; // skip discriminator
    let batch_id_val = u64::from_le_bytes(data[offset..offset+8].try_into().unwrap());
    let total_units = u64::from_le_bytes(data[offset+40..offset+48].try_into().unwrap());
    let sold_units = u64::from_le_bytes(data[offset+48..offset+56].try_into().unwrap());
    let price_per_unit = u64::from_le_bytes(data[offset+56..offset+64].try_into().unwrap());
    let is_active = data[offset+64] != 0;

    let available = total_units - sold_units;
    let price_sol = price_per_unit as f64 / 1_000_000_000.0;

    Ok(Json(OnChainBatch {
        batch_id: batch_id_val,
        total_units,
        sold_units,
        available_units: available,
        price_per_unit_lamports: price_per_unit,
        price_per_unit_sol: price_sol,
        is_active,
    }))
}

fn derive_batch_pda(batch_id: u64, program_id: &str) -> String {
    // Kita simpan PDA di config, return berdasarkan batch_id
    match batch_id {
        1 => std::env::var("BATCH_1_PDA").unwrap_or_default(),
        2 => std::env::var("BATCH_2_PDA").unwrap_or_default(),
        3 => std::env::var("BATCH_3_PDA").unwrap_or_default(),
        _ => String::new(),
    }
}