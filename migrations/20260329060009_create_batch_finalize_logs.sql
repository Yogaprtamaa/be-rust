-- Rekonstruksi: log finalisasi batch
CREATE TABLE IF NOT EXISTS batch_finalize_logs (
    id          UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    batch_id    UUID NOT NULL REFERENCES batches(id),
    action      VARCHAR(50) NOT NULL,
    notes       TEXT,
    created_at  TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_batch_finalize_logs_batch ON batch_finalize_logs(batch_id);
