use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize, FromRow, Clone)]
pub struct ReferralEarning {
    pub id: Uuid,
    pub referrer_wallet: String,
    pub buyer_wallet: String,
    pub usdt_amount: i64,
    pub referral_bonus: i64,
    pub tani_received: i64,
    pub tx_hash: Option<String>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Serialize)]
pub struct ReferralStats {
    pub referrer_wallet: String,
    pub total_referrals: i64,
    pub total_bonus_usdt_raw: i64,
    pub history: Vec<ReferralEarning>,
}
