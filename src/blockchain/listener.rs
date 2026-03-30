use std::sync::Arc;
use crate::AppState;
use super::solana_client::SolanaClient;

pub async fn start_event_listener(state: Arc<AppState>) {
    let program_id = state.config.program_id.clone();
    let rpc_url = state.config.solana_rpc_url.clone();

    if program_id.is_empty() {
        tracing::warn!("PROGRAM_ID tidak diset, event listener tidak dijalankan");
        return;
    }

    tokio::spawn(async move {
        tracing::info!("Event listener dimulai untuk program {}", program_id);

        let client = SolanaClient::new(&rpc_url, &program_id);
        let mut last_signature: Option<String> = None;

        loop {
            match client.get_signatures_for_address(&program_id).await {
                Ok(signatures) => {
                    // Process signatures dari yang terlama ke terbaru
                    let mut new_sigs: Vec<String> = vec![];
                    for sig in &signatures {
                        if let Some(ref last) = last_signature {
                            if sig == last { break; }
                        }
                        new_sigs.push(sig.clone());
                    }

                    // Reverse supaya urutan kronologis
                    new_sigs.reverse();

                    for sig in new_sigs {
                        tracing::info!("Transaksi baru: {}", sig);

                        // Fetch transaction detail untuk cek event type
                        match client.get_transaction(&sig).await {
                            Ok(tx_data) => {
                                process_transaction(&sig, &tx_data, &state).await;
                            }
                            Err(e) => {
                                tracing::error!("Error fetch tx {}: {}", sig, e);
                            }
                        }
                    }

                    if let Some(first) = signatures.first() {
                        last_signature = Some(first.clone());
                    }
                }
                Err(e) => {
                    tracing::error!("Error poll signatures: {}", e);
                }
            }

            tokio::time::sleep(tokio::time::Duration::from_secs(10)).await;
        }
    });
}

async fn process_transaction(
    sig: &str,
    tx_data: &serde_json::Value,
    state: &Arc<AppState>,
) {
    // Cek log messages untuk identify event type
    let logs = tx_data["meta"]["logMessages"]
        .as_array()
        .cloned()
        .unwrap_or_default();

    let log_strings: Vec<String> = logs.iter()
        .filter_map(|l| l.as_str().map(|s| s.to_string()))
        .collect();

    // Detect TokenPurchased event
    if log_strings.iter().any(|l| l.contains("Token Sale:")) {
        tracing::info!("TokenPurchased event: {}", sig);
        handle_token_purchase(sig, &log_strings, state).await;
    }

    // Detect PlotAllocated event
    if log_strings.iter().any(|l| l.contains("Allocation:")) {
        tracing::info!("PlotAllocated event: {}", sig);
        handle_plot_allocation(sig, &log_strings, state).await;
    }
}

async fn handle_token_purchase(
    sig: &str,
    logs: &[String],
    state: &Arc<AppState>,
) {
    // Parse log: "Token Sale: X USDT → Y TANI"
    let sale_log = logs.iter().find(|l| l.contains("Token Sale:"));

    if let Some(log) = sale_log {
        tracing::info!("Processing token purchase: {}", log);

        // Update status token_purchase jadi confirmed
        match sqlx::query!(
            "UPDATE token_purchases SET status = 'success' WHERE tx_hash = $1 AND status = 'pending'",
            sig
        )
        .execute(&state.db)
        .await
        {
            Ok(result) => {
                if result.rows_affected() > 0 {
                    tracing::info!("Token purchase confirmed: {}", sig);
                }
            }
            Err(e) => tracing::error!("DB error update token_purchase: {}", e),
        }
    }
}

async fn handle_plot_allocation(
    sig: &str,
    logs: &[String],
    state: &Arc<AppState>,
) {
    // Parse log: "Allocation: plot=C7 tani=100 treasury=70 burn=30 nft=Z4-PLOT-C7-xxx"
    let alloc_log = logs.iter().find(|l| l.contains("Allocation:"));

    if let Some(log) = alloc_log {
        tracing::info!("Processing allocation: {}", log);

        // Extract plot_id dari log
        let plot_id = extract_value(log, "plot=");
        let nft_id = extract_value(log, "nft=");

        if let (Some(plot_id), Some(nft_id)) = (plot_id, nft_id) {
            // Update allocation status
            match sqlx::query!(
                "UPDATE allocations SET status = 'success' WHERE tx_hash = $1 AND status = 'pending'",
                sig
            )
            .execute(&state.db)
            .await
            {
                Ok(result) => {
                    if result.rows_affected() > 0 {
                        tracing::info!("Allocation confirmed: {} plot={} nft={}", sig, plot_id, nft_id);
                    }
                }
                Err(e) => tracing::error!("DB error update allocation: {}", e),
            }

            // Update plot capacity
            match sqlx::query!(
                r#"
                UPDATE plots
                SET allocated_capacity = allocated_capacity + 1,
                    remaining_capacity = remaining_capacity - 1,
                    status = CASE
                        WHEN remaining_capacity - 1 = 0 THEN 'filled'
                        WHEN remaining_capacity - 1 < 50 THEN 'limited'
                        ELSE status
                    END,
                    updated_at = NOW()
                WHERE plot_id = $1
                "#,
                plot_id
            )
            .execute(&state.db)
            .await
            {
                Ok(_) => tracing::info!("Plot {} capacity updated", plot_id),
                Err(e) => tracing::error!("DB error update plot: {}", e),
            }
        }
    }
}

fn extract_value(log: &str, key: &str) -> Option<String> {
    let start = log.find(key)? + key.len();
    let rest = &log[start..];
    let end = rest.find(' ').unwrap_or(rest.len());
    Some(rest[..end].to_string())
}