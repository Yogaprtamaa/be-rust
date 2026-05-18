-- Rekonstruksi: tambah batch_id dan refund_tx_hash ke allocations
ALTER TABLE allocations ADD COLUMN IF NOT EXISTS batch_id UUID REFERENCES batches(id);
ALTER TABLE allocations ADD COLUMN IF NOT EXISTS refund_tx_hash VARCHAR(88);
