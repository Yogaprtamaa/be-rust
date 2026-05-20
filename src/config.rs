use std::env;

#[derive(Clone, Debug)]
pub struct Config {
    pub database_url: String,
    pub jwt_secret: String,
    pub port: u16,
    pub solana_rpc_url: String,
    pub program_id: String,
    // Token addresses
    pub tani_mint: String,
    pub usdt_mint: String,
    // Wallet addresses
    pub sale_inventory_wallet: String,
    pub usdt_treasury_wallet: String,
    // Rate
    pub tani_per_usdt: f64,
}

impl Config {
    pub fn from_env() -> Self {
        Config {
            database_url: env::var("DATABASE_URL")
                .expect("DATABASE_URL wajib diset"),
            jwt_secret: env::var("JWT_SECRET")
                .expect("JWT_SECRET wajib diset"),
            port: env::var("PORT")
                .unwrap_or("3001".to_string())
                .parse()
                .expect("PORT harus angka"),
            solana_rpc_url: env::var("SOLANA_RPC_URL")
                .unwrap_or("https://api.devnet.solana.com".to_string()),
            program_id: env::var("PROGRAM_ID")
                .unwrap_or_default(),
            tani_mint: env::var("TANI_MINT")
                .unwrap_or_default(),
            usdt_mint: env::var("USDT_MINT")
                .unwrap_or_default(),
            sale_inventory_wallet: env::var("SALE_INVENTORY_WALLET")
                .unwrap_or_default(),
            usdt_treasury_wallet: env::var("USDT_TREASURY_WALLET")
                .unwrap_or_default(),
            tani_per_usdt: env::var("TANI_PER_USDT")
                .unwrap_or("10.0".to_string())
                .parse()
                .unwrap_or(10.0),
        }
    }
}