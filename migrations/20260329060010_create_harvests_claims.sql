-- Placeholder V2: harvest claims (belum diimplementasi)
CREATE TABLE IF NOT EXISTS harvest_claims (
    id              UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    wallet_address  VARCHAR(44) NOT NULL,
    batch_id        UUID REFERENCES batches(id),
    nft_id          VARCHAR(50),
    claim_amount    DECIMAL(20,6),
    tx_hash         VARCHAR(88) UNIQUE,
    status          VARCHAR(20) NOT NULL DEFAULT 'pending',
    created_at      TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_harvest_claims_wallet ON harvest_claims(wallet_address);
CREATE INDEX IF NOT EXISTS idx_harvest_claims_batch ON harvest_claims(batch_id);
