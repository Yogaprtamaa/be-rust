use axum::{
    extract::{Request, State},
    http::header,
    middleware::Next,
    response::Response,
};
use jsonwebtoken::{decode, DecodingKey, Validation};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use uuid::Uuid;
use crate::{errors::AppError, AppState};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Claims {
    pub sub: String,            // user_id
    pub wallet: String,         // wallet address
    pub exp: usize,             // expiry
}

// Extractor — bisa dipakai di handler sebagai parameter
#[derive(Debug, Clone)]
pub struct AuthUser {
    pub user_id: Uuid,
    pub wallet: String,
}

pub async fn require_auth(
    State(state): State<Arc<AppState>>,
    mut request: Request,
    next: Next,
) -> Result<Response, AppError> {
    let token = request
        .headers()
        .get(header::AUTHORIZATION)
        .and_then(|v| v.to_str().ok())
        .and_then(|v| v.strip_prefix("Bearer "))
        .ok_or_else(|| AppError::Unauthorized("Token tidak ada".to_string()))?;

    // TODO(mainnet-blocker): implement wallet signature verification
    // Reference: https://docs.solana.com/developing/clients/javascript-api#signing-messages
    // Tanpa ini, siapapun bisa claim jadi wallet apapun — BLOCKER sebelum mainnet.
    let token_data = decode::<Claims>(
        token,
        &DecodingKey::from_secret(state.config.jwt_secret.as_bytes()),
        &Validation::default(),
    )
    .map_err(|_| AppError::Unauthorized("Token tidak valid".to_string()))?;

    let auth_user = AuthUser {
        user_id: Uuid::parse_str(&token_data.claims.sub)
            .map_err(|_| AppError::Unauthorized("User ID tidak valid".to_string()))?,
        wallet: token_data.claims.wallet,
    };

    // Inject AuthUser ke request extensions, bisa diambil di handler
    request.extensions_mut().insert(auth_user);

    Ok(next.run(request).await)
}
