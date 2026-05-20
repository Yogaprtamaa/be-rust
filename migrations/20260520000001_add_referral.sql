-- Add referrer tracking to token_purchases
ALTER TABLE token_purchases
  ADD COLUMN IF NOT EXISTS referrer_wallet TEXT,
  ADD COLUMN IF NOT EXISTS referral_bonus  BIGINT;  -- raw USDT units (6 decimals)

-- Referral earnings ledger
CREATE TABLE IF NOT EXISTS referral_earnings (
  id              UUID        PRIMARY KEY DEFAULT gen_random_uuid(),
  referrer_wallet TEXT        NOT NULL,
  buyer_wallet    TEXT        NOT NULL,
  usdt_amount     BIGINT      NOT NULL,
  referral_bonus  BIGINT      NOT NULL,
  tani_received   BIGINT      NOT NULL,
  tx_hash         TEXT        UNIQUE,
  created_at      TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE INDEX IF NOT EXISTS idx_referral_earnings_referrer
  ON referral_earnings (referrer_wallet);
