use axum::{
    extract::{Extension, State},
    routing::get,
    Json, Router,
};
use serde::Serialize;
use std::sync::Arc;

use crate::{
    errors::AppResult,
    middleware::auth::AuthUser,
    AppState,
};

#[derive(Debug, Serialize)]
pub struct PortfolioNft {
    pub nft_id: String,
    pub plot_id: String,
    pub block_id: String,
    pub location_name: String,
    pub asset_type: String,
    pub tani_spent: rust_decimal::Decimal,
    pub treasury_amount: rust_decimal::Decimal,
    pub burn_amount: rust_decimal::Decimal,
    pub status: String,
    pub minted_at: chrono::DateTime<chrono::Utc>,
    pub legal_reference_id: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct PortfolioResponse {
    pub wallet_address: String,
    pub total_nfts: i64,
    pub nfts: Vec<PortfolioNft>,
    pub purchase_history: Vec<serde_json::Value>,
}

pub fn router() -> Router<Arc<AppState>> {
    Router::new().route("/portfolio", get(get_portfolio))
}

async fn get_portfolio(
    State(state): State<Arc<AppState>>,
    Extension(auth_user): Extension<AuthUser>,
) -> AppResult<Json<PortfolioResponse>> {
    // Ambil semua NFT milik user
    let nfts = sqlx::query_as!(
        PortfolioNft,
        r#"
        SELECT
            n.nft_id,
            n.plot_id,
            p.block_id,
            p.location_name,
            p.asset_type,
            a.tani_spent,
            a.treasury_amount,
            a.burn_amount,
            n.status,
            n.minted_at,
            n.legal_reference_id
        FROM nft_records n
        JOIN plots p ON p.plot_id = n.plot_id
        JOIN allocations a ON a.allocation_id = n.allocation_id
        WHERE n.wallet_address = $1
        ORDER BY n.minted_at DESC
        "#,
        auth_user.wallet
    )
    .fetch_all(&state.db)
    .await?;

    let total = nfts.len() as i64;

    // Riwayat pembelian $TANI
    let purchases = sqlx::query!(
        r#"
        SELECT purchase_id, usdt_amount, tani_amount, rate_used, tx_hash, status, created_at, referrer_wallet
        FROM token_purchases
        WHERE wallet_address = $1
        ORDER BY created_at DESC
        LIMIT 20
        "#,
        auth_user.wallet
    )
    .fetch_all(&state.db)
    .await?;

    let purchase_history: Vec<serde_json::Value> = purchases
        .iter()
        .map(|p| {
            serde_json::json!({
                "purchase_id": p.purchase_id,
                "usdt_amount": p.usdt_amount,
                "tani_amount": p.tani_amount,
                "rate_used": p.rate_used,
                "tx_hash": p.tx_hash,
                "status": p.status,
                "created_at": p.created_at,
                "referrer_wallet": p.referrer_wallet
            })
        })
        .collect();

    Ok(Json(PortfolioResponse {
        wallet_address: auth_user.wallet,
        total_nfts: total,
        nfts,
        purchase_history,
    }))
}
