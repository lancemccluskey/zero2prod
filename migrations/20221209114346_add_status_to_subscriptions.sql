-- Add `status` as an optional column
-- This way we can do a db migration that wont break prod
ALTER TABLE
  subscriptions
ADD
  COLUMN status TEXT NULL;