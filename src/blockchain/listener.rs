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

    // Detect ReferralEvent (buy_tani_referred) — check first, more specific
    if log_strings.iter().any(|l| l.contains("Referral:")) {
        tracing::info!("ReferralEvent: {}", sig);
        handle_referral(sig, &log_strings, state).await;
    // Detect TokenSaleEvent (buy_tani without referrer)
    } else if log_strings.iter().any(|l| l.contains("Token Sale:")) {
        tracing::info!("TokenSaleEvent: {}", sig);
        handle_token_purchase(sig, &log_strings, state).await;
    }
}

async fn handle_token_purchase(
    sig: &str,
    logs: &[String],
    state: &Arc<AppState>,
) {
    // Parse log: "Token Sale: wallet=... usdt=X tani_received=Y"
    // Field mapping: event.wallet (bukan buyer), event.tani_received (bukan tani_amount)
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

async fn handle_referral(
    sig: &str,
    logs: &[String],
    state: &Arc<AppState>,
) {
    // Actual log format from contract msg!:
    // "Token Sale (Referral): {usdt_raw} USDT → {tani_raw} TANI | referrer={pubkey} bonus={bonus_raw} USDT"
    let ref_log = match logs.iter().find(|l| l.contains("Referral:")) {
        Some(l) => l,
        None => return,
    };

    tracing::info!("Processing referral: {}", ref_log);

    let referrer = extract_value(ref_log, "referrer=");
    let bonus    = extract_value(ref_log, "bonus=").and_then(|v| v.parse::<i64>().ok());
    let usdt     = parse_number_after(ref_log, "Referral): ");
    let tani     = parse_number_before(ref_log, " TANI");

    // buyer is not in the msg! log — get it from token_purchases (inserted by recordTokenPurchase)
    let buyer = sqlx::query_scalar!(
        "SELECT wallet_address FROM token_purchases WHERE tx_hash = $1",
        sig
    )
    .fetch_optional(&state.db)
    .await
    .ok()
    .flatten();

    match (buyer, referrer, usdt, bonus, tani) {
        (Some(buyer), Some(referrer), Some(usdt_amt), Some(bonus_amt), Some(tani_recv)) => {
            let sig_str: &str = sig;
            let referrer_str: &str = &referrer;
            let buyer_str: &str = &buyer;

            match sqlx::query!(
                r#"INSERT INTO referral_earnings
                   (referrer_wallet, buyer_wallet, usdt_amount, referral_bonus, tani_received, tx_hash)
                   VALUES ($1, $2, $3, $4, $5, $6)
                   ON CONFLICT (tx_hash) DO NOTHING"#,
                referrer_str, buyer_str, usdt_amt, bonus_amt, tani_recv, sig_str
            )
            .execute(&state.db)
            .await
            {
                Ok(r) => tracing::info!("Referral earning recorded ({} rows): {}", r.rows_affected(), sig),
                Err(e) => tracing::error!("DB error insert referral_earning: {}", e),
            }

            sqlx::query!(
                "UPDATE token_purchases SET status = 'success', referrer_wallet = $2
                 WHERE tx_hash = $1 AND status = 'pending'",
                sig_str, referrer_str
            )
            .execute(&state.db)
            .await
            .ok();
        }
        _ => tracing::warn!("Could not parse Referral log: {}", ref_log),
    }
}

fn extract_value(log: &str, key: &str) -> Option<String> {
    let start = log.find(key)? + key.len();
    let rest = &log[start..];
    let end = rest.find(' ').unwrap_or(rest.len());
    Some(rest[..end].to_string())
}

// Parse the first number immediately after `key` (e.g., "Referral): " → "1000000")
fn parse_number_after(log: &str, key: &str) -> Option<i64> {
    let start = log.find(key)? + key.len();
    let rest = &log[start..];
    let end = rest.find(' ').unwrap_or(rest.len());
    rest[..end].parse().ok()
}

// Parse the number immediately before `suffix` (e.g., " TANI" → "1700000000")
fn parse_number_before(log: &str, suffix: &str) -> Option<i64> {
    let pos = log.find(suffix)?;
    let before = log[..pos].trim();
    let num_start = before.rfind(' ').map(|i| i + 1).unwrap_or(0);
    before[num_start..].parse().ok()
}