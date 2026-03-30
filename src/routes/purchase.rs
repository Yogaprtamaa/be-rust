use axum::{
    extract::{Extension, State},
    routing::post,
    Json, Router,
};
use std::sync::Arc;
use crate::{
    errors::{AppError, AppResult},
    middleware::auth::AuthUser,
    models::purchase::{CreatePurchaseRequest, Purchase},
    AppState,
};

pub fn router() -> Router<Arc<AppState>> {
    Router::new().route("/purchases", post(create_purchase))
}

async fn create_purchase(
    State(state): State<Arc<AppState>>,
    Extension(auth_user): Extension<AuthUser>,
    Json(body): Json<CreatePurchaseRequest>,
) -> AppResult<Json<Purchase>> {
    if !body.tx_hash.starts_with("0x") || body.tx_hash.len() != 66 {
        return Err(AppError::BadRequest("tx_hash tidak valid".to_string()));
    }

    let batch = sqlx::query!(
        "SELECT id, total_units, sold_units, price_per_unit_eth, status FROM batches WHERE id = $1",
        body.batch_id
    )
    .fetch_optional(&state.db)
    .await?
    .ok_or_else(|| AppError::NotFound("Batch tidak ditemukan".to_string()))?;

    if batch.status != "open" && batch.status != "partial" {
        return Err(AppError::BadRequest("Batch tidak aktif".to_string()));
    }

    if body.units <= 0 {
        return Err(AppError::BadRequest("Units harus lebih dari 0".to_string()));
    }

    let available = batch.total_units - batch.sold_units;
    if body.units > available {
        return Err(AppError::BadRequest(format!("Stok tidak cukup. Tersedia: {} unit", available)));
    }

    let existing = sqlx::query!(
        "SELECT id FROM purchases WHERE tx_hash = $1",
        body.tx_hash
    )
    .fetch_optional(&state.db)
    .await?;

    if existing.is_some() {
        return Err(AppError::BadRequest("tx_hash sudah digunakan".to_string()));
    }

    let total_eth = batch.price_per_unit_eth * rust_decimal::Decimal::from(body.units);

    let purchase = sqlx::query_as!(
        Purchase,
        r#"
        INSERT INTO purchases (user_id, batch_id, units, total_paid_wei, total_paid_eth, tx_hash, status)
        VALUES ($1, $2, $3, $4, $5, $6, 'pending')
        RETURNING *
        "#,
        auth_user.user_id,
        body.batch_id,
        body.units,
        body.total_paid_wei,
        total_eth,
        body.tx_hash
    )
    .fetch_one(&state.db)
    .await?;

    let new_sold = batch.sold_units + body.units;
    let new_status = if new_sold >= batch.total_units { "closed" }
                     else { "partial" };

    sqlx::query!(
        "UPDATE batches SET sold_units = $1, status = $2, updated_at = NOW() WHERE id = $3",
        new_sold, new_status, body.batch_id
    )
    .execute(&state.db)
    .await?;

    Ok(Json(purchase))
}
