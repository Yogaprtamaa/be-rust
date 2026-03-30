CREATE TABLE IF NOT EXISTS legal_references (
    legal_reference_id  VARCHAR(50) PRIMARY KEY,
    title               VARCHAR(255) NOT NULL,
    entity_reference    VARCHAR(255),
    contract_reference  VARCHAR(255),
    disclosure_version  VARCHAR(20) NOT NULL DEFAULT 'v1.0',
    effective_date      DATE NOT NULL,
    created_at          TIMESTAMPTZ NOT NULL DEFAULT NOW()
);