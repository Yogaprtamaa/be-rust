use anyhow::Result;
use reqwest::Client;
use serde_json::{json, Value};

pub struct SolanaClient {
    pub rpc_url: String,
    pub program_id: String,
    pub http: Client,
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

    pub async fn get_transaction(&self, signature: &str) -> Result<Value> {
        let result = self.rpc_call(
            "getTransaction",
            json!([
                signature,
                {
                    "encoding": "json",
                    "commitment": "confirmed",
                    "maxSupportedTransactionVersion": 0
                }
            ])
        ).await?;

        Ok(result["result"].clone())
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
}