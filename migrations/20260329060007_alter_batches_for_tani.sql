-- Rekonstruksi: optimisasi index batches untuk sistem TANI
CREATE INDEX IF NOT EXISTS idx_batches_created_at ON batches(created_at DESC);
CREATE INDEX IF NOT EXISTS idx_batches_updated_at ON batches(updated_at DESC);
