-- Rekonstruksi: perluas VARCHAR status allocations untuk status flow lengkap
ALTER TABLE allocations ALTER COLUMN status TYPE VARCHAR(20);

ALTER TABLE allocations DROP CONSTRAINT IF EXISTS allocation_status_check;
ALTER TABLE allocations ADD CONSTRAINT allocation_status_check CHECK (
    status IN ('pending', 'pending_batch', 'success', 'confirmed', 'minting', 'minted', 'failed', 'refunded')
);
