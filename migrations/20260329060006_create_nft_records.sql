CREATE TABLE IF NOT EXISTS nft_records (
    nft_id              VARCHAR(50) PRIMARY KEY,
    wallet_address      VARCHAR(44) NOT NULL,
    plot_id             VARCHAR(10) NOT NULL REFERENCES plots(plot_id),
    allocation_id       UUID REFERENCES allocations(allocation_id),
    metadata_uri        TEXT,
    legal_reference_id  VARCHAR(50) REFERENCES legal_references(legal_reference_id),
    mint_tx_hash        VARCHAR(88),
    status              VARCHAR(15) NOT NULL DEFAULT 'active',
    minted_at           TIMESTAMPTZ NOT NULL DEFAULT NOW(),

    CONSTRAINT nft_status_check CHECK (
        status IN ('active', 'transferred', 'burned')
    )
);

CREATE INDEX idx_nft_wallet ON nft_records(wallet_address);
CREATE INDEX idx_nft_plot ON nft_records(plot_id);