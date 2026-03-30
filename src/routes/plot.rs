use axum::{
    extract::{Path, Query, State},
    routing::get,
    Json, Router,
};
use serde::Deserialize;
use std::sync::Arc;

use crate::{
    errors::{AppError, AppResult},
    models::plot::PlotResponse,
    AppState,
};

#[derive(Deserialize)]
pub struct PlotFilter {
    pub block_id: Option<String>,
    pub status: Option<String>,
}

pub fn router() -> Router<Arc<AppState>> {
    Router::new()
        .route("/plots", get(get_all_plots))
        .route("/plots/blocks", get(get_blocks_summary))
        .route("/plots/:plot_id", get(get_plot_by_id))
}

async fn get_all_plots(
    State(state): State<Arc<AppState>>,
    Query(filter): Query<PlotFilter>,
) -> AppResult<Json<Vec<PlotResponse>>> {
    let plots = match (filter.block_id, filter.status) {
        (Some(block), Some(status)) => {
            sqlx::query_as!(
                crate::models::plot::Plot,
                "SELECT * FROM plots WHERE block_id = $1 AND status = $2 ORDER BY plot_id",
                block,
                status
            )
            .fetch_all(&state.db)
            .await?
        }
        (Some(block), None) => {
            sqlx::query_as!(
                crate::models::plot::Plot,
                "SELECT * FROM plots WHERE block_id = $1 AND status != 'hidden' ORDER BY plot_id",
                block
            )
            .fetch_all(&state.db)
            .await?
        }
        (None, Some(status)) => {
            sqlx::query_as!(
                crate::models::plot::Plot,
                "SELECT * FROM plots WHERE status = $1 ORDER BY block_id, plot_id",
                status
            )
            .fetch_all(&state.db)
            .await?
        }
        (None, None) => {
            sqlx::query_as!(
                crate::models::plot::Plot,
                "SELECT * FROM plots WHERE status != 'hidden' ORDER BY block_id, plot_id"
            )
            .fetch_all(&state.db)
            .await?
        }
    };

    Ok(Json(plots.into_iter().map(PlotResponse::from).collect()))
}

async fn get_plot_by_id(
    State(state): State<Arc<AppState>>,
    Path(plot_id): Path<String>,
) -> AppResult<Json<PlotResponse>> {
    let plot = sqlx::query_as!(
        crate::models::plot::Plot,
        "SELECT * FROM plots WHERE plot_id = $1",
        plot_id.to_uppercase()
    )
    .fetch_optional(&state.db)
    .await?
    .ok_or_else(|| AppError::NotFound(format!("Plot {} tidak ditemukan", plot_id)))?;

    Ok(Json(PlotResponse::from(plot)))
}

// Summary per block untuk Plot Explorer
async fn get_blocks_summary(State(state): State<Arc<AppState>>) -> AppResult<Json<serde_json::Value>> {
    let rows = sqlx::query!(
        r#"
        SELECT
            block_id,
            COUNT(*) as total_plots,
            COUNT(*) FILTER (WHERE status = 'available') as available,
            COUNT(*) FILTER (WHERE status = 'limited') as limited,
            COUNT(*) FILTER (WHERE status = 'filled') as filled,
            SUM(remaining_capacity) as total_remaining,
            SUM(total_allocation_capacity) as total_capacity
        FROM plots
        WHERE status != 'hidden'
        GROUP BY block_id
        ORDER BY block_id
        "#
    )
    .fetch_all(&state.db)
    .await?;

    let blocks: Vec<serde_json::Value> = rows
        .iter()
        .map(|r| {
            serde_json::json!({
                "block_id": r.block_id,
                "total_plots": r.total_plots,
                "available": r.available,
                "limited": r.limited,
                "filled": r.filled,
                "total_remaining": r.total_remaining,
                "total_capacity": r.total_capacity,
            })
        })
        .collect();

    Ok(Json(serde_json::json!({ "blocks": blocks })))
}
