use axum::{extract::State, routing::post, Json, Router};
use jsonwebtoken::{encode, EncodingKey, Header};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use chrono::Utc;

use crate::{
    errors::{AppError, AppResult},
    middleware::auth::Claims,
    AppState,
};

#[derive(Deserialize)]
pub struct LoginRequest {
    pub wallet_address: String,
}

#[derive(Serialize)]
pub struct LoginResponse {
    pub token: String,
    pub user_id: String,
    pub wallet_address: String,
    pub is_admin: bool,
}

pub fn router() -> Router<Arc<AppState>> {
    Router::new().route("/auth/login", post(login))
}

async fn login(
    State(state): State<Arc<AppState>>,
    Json(body): Json<LoginRequest>,
) -> AppResult<Json<LoginResponse>> {
    let wallet = body.wallet_address.trim().to_string();

    if !crate::config::is_valid_pubkey(&wallet) {
        return Err(AppError::BadRequest("Wallet address tidak valid".to_string()));
    }

    // Upsert user — kalau belum ada buat baru, kalau ada return yang existing
    let user = sqlx::query!(
        r#"
        INSERT INTO users (wallet_address)
        VALUES ($1)
        ON CONFLICT (wallet_address)
        DO UPDATE SET updated_at = NOW()
        RETURNING user_id, wallet_address
        "#,
        wallet
    )
    .fetch_one(&state.db)
    .await?;

    // Generate JWT expire 30 hari
    let exp = (Utc::now() + chrono::Duration::days(30)).timestamp() as usize;

    let claims = Claims {
        sub: user.user_id.to_string(),
        wallet: user.wallet_address.clone(),
        exp,
    };

    let token = encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(state.config.jwt_secret.as_bytes()),
    )
    .map_err(|e| AppError::Internal(e.to_string()))?;

    Ok(Json(LoginResponse {
        token,
        user_id: user.user_id.to_string(),
        is_admin: user.wallet_address == state.config.admin_wallet,
        wallet_address: user.wallet_address,
    }))
}
