CREATE TABLE IF NOT EXISTS token_purchases (
    purchase_id     UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    wallet_address  VARCHAR(44) NOT NULL,
    usdt_amount     DECIMAL(20,6) NOT NULL,
    tani_amount     DECIMAL(20,2) NOT NULL,
    rate_used       DECIMAL(20,6) NOT NULL,
    tx_hash         VARCHAR(88) UNIQUE,
    status          VARCHAR(10) NOT NULL DEFAULT 'pending',
    created_at      TIMESTAMPTZ NOT NULL DEFAULT NOW(),

    CONSTRAINT purchase_status_check CHECK (
        status IN ('pending', 'success', 'failed')
    )
);

CREATE INDEX idx_purchases_wallet ON token_purchases(wallet_address);
CREATE INDEX idx_purchases_status ON token_purchases(status);
CREATE INDEX idx_purchases_tx ON token_purchases(tx_hash);