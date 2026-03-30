use std::sync::Arc;
use crate::AppState;

pub async fn start_event_listener(state: Arc<AppState>) {
    let program_id = state.config.program_id.clone();
    let rpc_url = state.config.solana_rpc_url.clone();

    if program_id.is_empty() {
        tracing::warn!("PROGRAM_ID tidak diset, event listener tidak dijalankan");
        return;
    }

    tokio::spawn(async move {
        tracing::info!("Event listener dimulai untuk program {}", program_id);

        let client = super::solana_client::SolanaClient::new(&rpc_url, &program_id);
        let mut last_signature: Option<String> = None;

        loop {
            match client.get_signatures_for_address(&program_id).await {
                Ok(signatures) => {
                    for sig in &signatures {
                        if let Some(ref last) = last_signature {
                            if sig == last {
                                break;
                            }
                        }

                        tracing::info!("Transaksi baru terdeteksi: {}", sig);

                        if let Err(e) = sqlx::query!(
                            "UPDATE token_purchases
                             SET status = 'success'
                             WHERE tx_hash = $1 AND status = 'pending'",
                            sig
                        )
                        .execute(&state.db)
                        .await
                        {
                            tracing::error!("DB error: {}", e);
                        }

                        if let Err(e) = sqlx::query!(
                            "UPDATE allocations
                             SET status = 'success'
                             WHERE tx_hash = $1 AND status = 'pending'",
                            sig
                        )
                        .execute(&state.db)
                        .await
                        {
                            tracing::error!("DB error: {}", e);
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