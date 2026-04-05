CREATE TABLE IF NOT EXISTS generations (
  id UUID PRIMARY KEY,
  api_key TEXT NOT NULL,
  prompt TEXT NOT NULL,
  markdown TEXT NOT NULL,
  output_formats JSONB NOT NULL,
  outputs JSONB NOT NULL,
  style JSONB,
  word_count INTEGER NOT NULL CHECK (word_count >= 0),
  created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_generations_api_key_created_at
  ON generations (api_key, created_at DESC);
