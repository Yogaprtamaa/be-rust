use axum::{
    extract::{Path, State},
    routing::get,
    Json, Router,
};
use rust_decimal::Decimal;
use rust_decimal::prelude::ToPrimitive;
use std::sync::Arc;

use crate::{
    errors::AppResult,
    models::referral::{ReferralEarning, ReferralStats},
    AppState,
};

pub fn router() -> Router<Arc<AppState>> {
    Router::new()
        .route("/referral/:wallet", get(get_referral_stats))
}

async fn get_referral_stats(
    Path(wallet): Path<String>,
    State(state): State<Arc<AppState>>,
) -> AppResult<Json<ReferralStats>> {
    // Query token_purchases directly — source of truth for referral history.
    // referral_earnings table depends on the blockchain listener which can lag;
    // token_purchases is written immediately by recordTokenPurchase on the FE.
    let rows = sqlx::query!(
        r#"SELECT purchase_id, wallet_address, usdt_amount, tani_amount, tx_hash, created_at
           FROM token_purchases
           WHERE referrer_wallet = $1 AND status = 'success'
           ORDER BY created_at DESC"#,
        wallet
    )
    .fetch_all(&state.db)
    .await?;

    let earnings: Vec<ReferralEarning> = rows
        .into_iter()
        .map(|r| {
            let usdt_raw = (r.usdt_amount * Decimal::new(1_000_000, 0))
                .to_i64()
                .unwrap_or(0);
            let bonus_raw  = usdt_raw * 5 / 100;
            let tani_raw   = (r.tani_amount * Decimal::new(1_000_000_000, 0))
                .to_i64()
                .unwrap_or(0);

            ReferralEarning {
                id:               r.purchase_id,
                referrer_wallet:  wallet.clone(),
                buyer_wallet:     r.wallet_address,
                usdt_amount:      usdt_raw,
                referral_bonus:   bonus_raw,
                tani_received:    tani_raw,
                tx_hash:          r.tx_hash,
                created_at:       r.created_at,
            }
        })
        .collect();

    let total_bonus     = earnings.iter().map(|e| e.referral_bonus).sum();
    let total_referrals = earnings.len() as i64;

    Ok(Json(ReferralStats {
        referrer_wallet:    wallet,
        total_referrals,
        total_bonus_usdt_raw: total_bonus,
        history:            earnings,
    }))
}
