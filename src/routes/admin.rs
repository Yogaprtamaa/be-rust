use axum::{extract::State, routing::get, Json, Router};
use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use serde::Serialize;
use std::sync::Arc;

use crate::{errors::AppResult, AppState};

#[derive(Serialize)]
pub struct AdminPurchase {
    pub wallet_address: String,
    pub usdt_amount: Decimal,
    pub tani_amount: Decimal,
    pub tx_hash: Option<String>,
    pub status: String,
    pub referrer_wallet: Option<String>,
    pub created_at: DateTime<Utc>,
}

#[derive(Serialize)]
pub struct AdminSummary {
    pub total_tani_sold: Decimal,
    pub total_usdt_in: Decimal,
    pub total_buyers: i64,
    pub purchases: Vec<AdminPurchase>,
}

#[derive(Serialize)]
pub struct WaitlistEntry {
    pub email: String,
    pub source: Option<String>,
    pub created_at: DateTime<Utc>,
}

pub fn router() -> Router<Arc<AppState>> {
    Router::new()
        .route("/admin/purchases", get(list_purchases))
        .route("/admin/waitlist", get(list_waitlist))
}

async fn list_purchases(State(state): State<Arc<AppState>>) -> AppResult<Json<AdminSummary>> {
    // Hanya status 'success' yang dihitung ke total — pending/failed belum jadi uang.
    let rows = sqlx::query!(
        r#"SELECT wallet_address, usdt_amount, tani_amount, tx_hash, status,
                  referrer_wallet, created_at
           FROM token_purchases
           ORDER BY created_at DESC"#
    )
    .fetch_all(&state.db)
    .await?;

    let totals = sqlx::query!(
        r#"SELECT COALESCE(SUM(tani_amount), 0) AS "tani!: Decimal",
                  COALESCE(SUM(usdt_amount), 0) AS "usdt!: Decimal",
                  COUNT(DISTINCT wallet_address) AS "buyers!: i64"
           FROM token_purchases WHERE status = 'success'"#
    )
    .fetch_one(&state.db)
    .await?;

    Ok(Json(AdminSummary {
        total_tani_sold: totals.tani,
        total_usdt_in: totals.usdt,
        total_buyers: totals.buyers,
        purchases: rows
            .into_iter()
            .map(|r| AdminPurchase {
                wallet_address: r.wallet_address,
                usdt_amount: r.usdt_amount,
                tani_amount: r.tani_amount,
                tx_hash: r.tx_hash,
                status: r.status,
                referrer_wallet: r.referrer_wallet,
                created_at: r.created_at,
            })
            .collect(),
    }))
}

async fn list_waitlist(State(state): State<Arc<AppState>>) -> AppResult<Json<Vec<WaitlistEntry>>> {
    let rows = sqlx::query!(
        r#"SELECT email, source, created_at FROM waitlist ORDER BY created_at DESC"#
    )
    .fetch_all(&state.db)
    .await?;

    Ok(Json(
        rows.into_iter()
            .map(|r| WaitlistEntry {
                email: r.email,
                source: r.source,
                created_at: r.created_at,
            })
            .collect(),
    ))
}
