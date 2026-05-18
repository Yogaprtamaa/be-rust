CREATE TABLE IF NOT EXISTS allocations (
    allocation_id       UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    wallet_address      VARCHAR(44) NOT NULL,
    plot_id             VARCHAR(10) NOT NULL REFERENCES plots(plot_id),
    allocation_quantity INTEGER NOT NULL DEFAULT 1,
    tani_spent          DECIMAL(20,2) NOT NULL,
    treasury_amount     DECIMAL(20,2) NOT NULL,
    burn_amount         DECIMAL(20,2) NOT NULL,
    nft_id              VARCHAR(50),
    tx_hash             VARCHAR(88) UNIQUE,
    status              VARCHAR(10) NOT NULL DEFAULT 'pending',
    created_at          TIMESTAMPTZ NOT NULL DEFAULT NOW(),

    CONSTRAINT allocation_status_check CHECK (
        status IN ('pending', 'success', 'failed')
    ),
    CONSTRAINT routing_check CHECK (
        treasury_amount + burn_amount = tani_spent
    )
);

CREATE INDEX IF NOT EXISTS idx_allocations_wallet ON allocations(wallet_address);
CREATE INDEX IF NOT EXISTS idx_allocations_plot ON allocations(plot_id);
CREATE INDEX IF NOT EXISTS idx_allocations_status ON allocations(status);
CREATE INDEX IF NOT EXISTS idx_allocations_tx ON allocations(tx_hash);