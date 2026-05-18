CREATE TABLE IF NOT EXISTS plots (
    plot_id                     VARCHAR(10) PRIMARY KEY,
    block_id                    VARCHAR(1) NOT NULL,
    location_name               VARCHAR(255) NOT NULL DEFAULT 'Rantau Harapan, Banyuasin, Sumatera Selatan',
    asset_type                  VARCHAR(100) NOT NULL DEFAULT 'Rice Field Allocation',
    total_area                  DECIMAL(10,2) NOT NULL,
    total_allocation_capacity   INTEGER NOT NULL,
    allocated_capacity          INTEGER NOT NULL DEFAULT 0,
    remaining_capacity          INTEGER NOT NULL,
    price_in_tani               DECIMAL(20,2) NOT NULL,
    price_in_usdt_reference     DECIMAL(20,2),
    status                      VARCHAR(10) NOT NULL DEFAULT 'available',
    legal_reference_id          VARCHAR(50),
    metadata_uri                TEXT,
    created_at                  TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at                  TIMESTAMPTZ NOT NULL DEFAULT NOW(),

    CONSTRAINT status_check CHECK (
        status IN ('available', 'limited', 'filled', 'paused', 'locked', 'hidden')
    ),
    CONSTRAINT capacity_check CHECK (
        allocated_capacity + remaining_capacity = total_allocation_capacity
    )
);

CREATE INDEX IF NOT EXISTS idx_plots_block ON plots(block_id);
CREATE INDEX IF NOT EXISTS idx_plots_status ON plots(status);