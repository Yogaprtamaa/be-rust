use axum::{extract::State, routing::post, Json, Router};
use serde::Deserialize;
use serde_json::{json, Value};
use std::sync::Arc;

use crate::{errors::AppError, errors::AppResult, AppState};

#[derive(Deserialize)]
pub struct WaitlistRequest {
    pub email: String,
    pub source: Option<String>,
}

pub fn router() -> Router<Arc<AppState>> {
    Router::new().route("/waitlist", post(join_waitlist))
}

fn valid_email(email: &str) -> bool {
    let (local, domain) = match email.split_once('@') {
        Some(parts) => parts,
        None => return false,
    };
    !local.is_empty()
        && domain.contains('.')
        && !domain.starts_with('.')
        && !domain.ends_with('.')
        && !email.contains(char::is_whitespace)
        && email.len() <= 254
}

async fn join_waitlist(
    State(state): State<Arc<AppState>>,
    Json(body): Json<WaitlistRequest>,
) -> AppResult<Json<Value>> {
    let email = body.email.trim().to_lowercase();
    if !valid_email(&email) {
        return Err(AppError::BadRequest("Invalid email".into()));
    }

    // Duplicate signup is not an error for the user — idempotent insert.
    sqlx::query!(
        r#"INSERT INTO waitlist (email, source) VALUES ($1, $2)
           ON CONFLICT (email) DO NOTHING"#,
        email,
        body.source,
    )
    .execute(&state.db)
    .await?;

    Ok(Json(json!({ "ok": true })))
}

#[cfg(test)]
mod tests {
    use super::valid_email;

    #[test]
    fn email_validation() {
        assert!(valid_email("a@b.co"));
        assert!(!valid_email("nope"));
        assert!(!valid_email("a@b"));
        assert!(!valid_email("@b.co"));
        assert!(!valid_email("a b@c.co"));
        assert!(!valid_email("a@b.co."));
    }
}
