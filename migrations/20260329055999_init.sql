-- Init: tabel awal batches dan purchases (versi lama, kompatibel IF NOT EXISTS)
CREATE TABLE IF NOT EXISTS batches (
    id                  UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    contract_batch_id   INTEGER,
    name                VARCHAR(255) NOT NULL DEFAULT '',
    location            VARCHAR(255) NOT NULL DEFAULT '',
    commodity           VARCHAR(100) NOT NULL DEFAULT '',
    area_hectares       DECIMAL(10,2) NOT NULL DEFAULT 0,
    total_units         INTEGER NOT NULL DEFAULT 0,
    sold_units          INTEGER NOT NULL DEFAULT 0,
    price_per_unit_wei  VARCHAR(78) NOT NULL DEFAULT '0',
    price_per_unit_eth  DECIMAL(20,8) NOT NULL DEFAULT 0,
    status              VARCHAR(20) NOT NULL DEFAULT 'open',
    description         TEXT,
    image_url           TEXT,
    created_at          TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at          TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE IF NOT EXISTS purchases (
    id              UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id         UUID NOT NULL,
    batch_id        UUID NOT NULL REFERENCES batches(id),
    units           INTEGER NOT NULL DEFAULT 0,
    total_paid_wei  VARCHAR(78) NOT NULL DEFAULT '0',
    total_paid_eth  DECIMAL(20,8) NOT NULL DEFAULT 0,
    tx_hash         VARCHAR(88) UNIQUE,
    block_number    BIGINT,
    status          VARCHAR(20) NOT NULL DEFAULT 'pending',
    created_at      TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_batches_status ON batches(status);
CREATE INDEX IF NOT EXISTS idx_purchases_batch ON purchases(batch_id);
CREATE INDEX IF NOT EXISTS idx_purchases_user ON purchases(user_id);
