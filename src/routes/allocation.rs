use axum::{
    extract::{Extension, Path, State},
    routing::{get, post},
    Json, Router,
};
use rust_decimal::Decimal;
use std::sync::Arc;

use crate::{
    errors::{AppError, AppResult},
    middleware::auth::AuthUser,
    models::allocation::{Allocation, AllocationPreview, CreateAllocationRequest},
    AppState,
};

pub fn router() -> Router<Arc<AppState>> {
    Router::new()
        .route("/allocations/preview/:plot_id", get(preview_allocation))
        .route("/allocations", post(create_allocation))
}

// Preview routing 70/30 sebelum user confirm
async fn preview_allocation(
    State(state): State<Arc<AppState>>,
    Extension(_auth_user): Extension<AuthUser>,
    Path(plot_id): Path<String>,
) -> AppResult<Json<AllocationPreview>> {
    let plot = sqlx::query!(
        "SELECT plot_id, price_in_tani, remaining_capacity, status FROM plots WHERE plot_id = $1",
        plot_id.to_uppercase()
    )
    .fetch_optional(&state.db)
    .await?
    .ok_or_else(|| AppError::NotFound(format!("Plot {} tidak ditemukan", plot_id)))?;

    if plot.status == "filled" || plot.status == "paused" || plot.status == "locked" {
        return Err(AppError::BadRequest(format!(
            "Plot {} tidak tersedia untuk alokasi (status: {})",
            plot_id, plot.status
        )));
    }

    let tani_required = plot.price_in_tani;
    let treasury = tani_required * Decimal::new(70, 2); // 70%
    let burn = tani_required * Decimal::new(30, 2); // 30%

    Ok(Json(AllocationPreview {
        plot_id: plot.plot_id,
        allocation_quantity: 1,
        tani_required,
        treasury_amount: treasury,
        burn_amount: burn,
        treasury_percentage: 70.0,
        burn_percentage: 30.0,
    }))
}

// Catat allocation setelah transaksi on-chain berhasil
async fn create_allocation(
    State(state): State<Arc<AppState>>,
    Extension(auth_user): Extension<AuthUser>,
    Json(body): Json<CreateAllocationRequest>,
) -> AppResult<Json<Allocation>> {
    // Validasi plot
    let plot = sqlx::query!(
        "SELECT plot_id, price_in_tani, remaining_capacity, status FROM plots WHERE plot_id = $1",
        body.plot_id.to_uppercase()
    )
    .fetch_optional(&state.db)
    .await?
    .ok_or_else(|| AppError::NotFound("Plot tidak ditemukan".to_string()))?;

    // Validasi status
    if plot.status == "filled" || plot.status == "paused" || plot.status == "locked" {
        return Err(AppError::BadRequest(format!(
            "Plot tidak tersedia (status: {})",
            plot.status
        )));
    }

    // Validasi kapasitas
    if body.allocation_quantity > plot.remaining_capacity {
        return Err(AppError::BadRequest(format!(
            "Kapasitas tidak cukup. Tersisa: {} unit",
            plot.remaining_capacity
        )));
    }

    // Validasi tx_hash
    let existing = sqlx::query!(
        "SELECT allocation_id FROM allocations WHERE tx_hash = $1",
        body.tx_hash
    )
    .fetch_optional(&state.db)
    .await?;

    if existing.is_some() {
        return Err(AppError::BadRequest("Transaksi sudah diproses".to_string()));
    }

    // Hitung routing 70/30
    let tani_per_unit = plot.price_in_tani;
    let qty = Decimal::from(body.allocation_quantity);
    let tani_spent = tani_per_unit * qty;
    let treasury_amount = tani_spent * Decimal::new(70, 2);
    let burn_amount = tani_spent * Decimal::new(30, 2);

    // Generate NFT ID
    let nft_id = format!(
        "SEEDRYM-PLOT-{}-{:06}",
        body.plot_id.to_uppercase(),
        rand_nft_id()
    );

    // Insert allocation
    let allocation = sqlx::query_as!(
        Allocation,
        r#"
        INSERT INTO allocations
            (wallet_address, plot_id, allocation_quantity, tani_spent,
             treasury_amount, burn_amount, nft_id, tx_hash, status)
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8, 'success')
        RETURNING *
        "#,
        auth_user.wallet,
        body.plot_id.to_uppercase(),
        body.allocation_quantity,
        tani_spent,
        treasury_amount,
        burn_amount,
        nft_id,
        body.tx_hash
    )
    .fetch_one(&state.db)
    .await?;

    // Update kapasitas plot
    let new_allocated = plot.remaining_capacity - body.allocation_quantity;
    let new_status = if new_allocated == 0 {
        "filled"
    } else if new_allocated < 50 {
        "limited"
    } else {
        &plot.status
    };

    sqlx::query!(
        r#"
        UPDATE plots
        SET allocated_capacity = allocated_capacity + $1,
            remaining_capacity = remaining_capacity - $1,
            status = $2,
            updated_at = NOW()
        WHERE plot_id = $3
        "#,
        body.allocation_quantity,
        new_status,
        body.plot_id.to_uppercase()
    )
    .execute(&state.db)
    .await?;

    // Insert NFT record
    sqlx::query!(
        r#"
        INSERT INTO nft_records
            (nft_id, wallet_address, plot_id, allocation_id, legal_reference_id, mint_tx_hash, status)
        VALUES ($1, $2, $3, $4, 'LEGAL-REF-001', $5, 'active')
        "#,
        nft_id,
        auth_user.wallet,
        body.plot_id.to_uppercase(),
        allocation.allocation_id,
        body.tx_hash
    )
    .execute(&state.db)
    .await?;

    Ok(Json(allocation))
}

fn rand_nft_id() -> u32 {
    use std::time::{SystemTime, UNIX_EPOCH};
    (SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .subsec_nanos()
        % 999_999)
        + 1
}
