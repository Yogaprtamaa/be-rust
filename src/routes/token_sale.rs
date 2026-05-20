use axum::{
    extract::{Extension, State},
    http::HeaderMap,
    routing::{get, post},
    Json, Router,
};
use rust_decimal::Decimal;
use std::sync::Arc;

use crate::{
    errors::{AppError, AppResult},
    middleware::auth::AuthUser,
    models::token_purchase::{CreateTokenPurchaseRequest, TokenPurchase},
    AppState,
};

pub fn router() -> Router<Arc<AppState>> {
    Router::new()
        .route("/token-sale/preview", get(preview_purchase))
        .route("/token-sale/buy", post(buy_tani))
}

async fn preview_purchase(
    State(state): State<Arc<AppState>>,
    Extension(auth_user): Extension<AuthUser>,
) -> AppResult<Json<serde_json::Value>> {
    let rate = state.config.tani_per_usdt;

    Ok(Json(serde_json::json!({
        "rate_tani_per_usdt": rate,
        "wallet": auth_user.wallet,
        "tani_mint": state.config.tani_mint,
        "usdt_mint": state.config.usdt_mint,
    })))
}

async fn buy_tani(
    State(state): State<Arc<AppState>>,
    Extension(auth_user): Extension<AuthUser>,
    Json(body): Json<CreateTokenPurchaseRequest>,
) -> AppResult<(HeaderMap, Json<TokenPurchase>)> {
    if body.usdt_amount <= Decimal::ZERO {
        return Err(AppError::BadRequest("Jumlah USDT harus lebih dari 0".to_string()));
    }

    if body.tx_hash.len() < 32 {
        return Err(AppError::BadRequest("tx_hash tidak valid".to_string()));
    }

    let existing = sqlx::query!(
        "SELECT purchase_id FROM token_purchases WHERE tx_hash = $1",
        body.tx_hash
    )
    .fetch_optional(&state.db)
    .await?;

    if existing.is_some() {
        return Err(AppError::BadRequest("Transaksi sudah diproses".to_string()));
    }

    let rate = Decimal::try_from(state.config.tani_per_usdt)
        .map_err(|e| AppError::Internal(e.to_string()))?;

    let tani_amount = body.usdt_amount * rate;

    let purchase = sqlx::query_as!(
        TokenPurchase,
        r#"
        INSERT INTO token_purchases
            (wallet_address, usdt_amount, tani_amount, rate_used, tx_hash, status, referrer_wallet)
        VALUES ($1, $2, $3, $4, $5, 'success', $6)
        RETURNING *
        "#,
        auth_user.wallet,
        body.usdt_amount,
        tani_amount,
        rate,
        body.tx_hash,
        body.referrer_wallet,
    )
    .fetch_one(&state.db)
    .await?;

    let mut headers = HeaderMap::new();
    headers.insert("X-Tx-Verified", "false".parse().unwrap());

    Ok((headers, Json(purchase)))
}
