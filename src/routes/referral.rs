use axum::{
    extract::{Path, State},
    routing::get,
    Json, Router,
};
use std::sync::Arc;

use crate::{
    errors::AppResult,
    models::referral::ReferralStats,
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
    let earnings = sqlx::query_as!(
        crate::models::referral::ReferralEarning,
        "SELECT * FROM referral_earnings
         WHERE referrer_wallet = $1
         ORDER BY created_at DESC",
        wallet
    )
    .fetch_all(&state.db)
    .await?;

    let total_bonus: i64 = earnings.iter().map(|e| e.referral_bonus).sum();

    Ok(Json(ReferralStats {
        referrer_wallet: wallet,
        total_referrals: earnings.len() as i64,
        total_bonus_usdt_raw: total_bonus,
        history: earnings,
    }))
}
