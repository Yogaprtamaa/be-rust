use anyhow::Result;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};

pub struct SolanaClient {
    pub rpc_url: String,
    pub program_id: String,
    pub http: Client,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct BatchOnChain {
    pub batch_id: u64,
    pub total_units: u64,
    pub sold_units: u64,
    pub price_per_unit: u64,
    pub is_active: bool,
}

impl SolanaClient {
    pub fn new(rpc_url: &str, program_id: &str) -> Self {
        SolanaClient {
            rpc_url: rpc_url.to_string(),
            program_id: program_id.to_string(),
            http: Client::new(),
        }
    }

    pub async fn rpc_call(&self, method: &str, params: Value) -> Result<Value> {
        let body = json!({
            "jsonrpc": "2.0",
            "id": 1,
            "method": method,
            "params": params
        });

        let response = self.http
            .post(&self.rpc_url)
            .json(&body)
            .send()
            .await?
            .json::<Value>()
            .await?;

        Ok(response)
    }

    pub async fn get_account_info(&self, pubkey: &str) -> Result<Option<Value>> {
        let result = self.rpc_call(
            "getAccountInfo",
            json!([pubkey, {"encoding": "base64"}])
        ).await?;

        if result["result"]["value"].is_null() {
            return Ok(None);
        }

        Ok(Some(result["result"]["value"].clone()))
    }

    pub async fn get_signatures_for_address(&self, address: &str) -> Result<Vec<String>> {
        let result = self.rpc_call(
            "getSignaturesForAddress",
            json!([address, {"limit": 10}])
        ).await?;

        let signatures = result["result"]
            .as_array()
            .unwrap_or(&vec![])
            .iter()
            .filter_map(|s| s["signature"].as_str().map(|s| s.to_string()))
            .collect();

        Ok(signatures)
    }
}