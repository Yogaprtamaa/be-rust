-- Early access waitlist signups
CREATE TABLE IF NOT EXISTS waitlist (
  id         UUID        PRIMARY KEY DEFAULT gen_random_uuid(),
  email      TEXT        NOT NULL UNIQUE,
  source     TEXT,
  created_at TIMESTAMPTZ NOT NULL DEFAULT now()
);
