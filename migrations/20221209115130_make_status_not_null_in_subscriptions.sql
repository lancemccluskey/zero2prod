-- Wrap whole migration in transaction to make sure
-- it succeeds or fails atomically.
BEGIN;

-- Backfil 'status' for historical entries
UPDATE
  subscriptions
SET
  status = 'confirmed'
WHERE
  status IS NULL;

-- Make 'status' mandatory
ALTER TABLE
  subscriptions
ALTER COLUMN
  status
SET
  NOT NULL;

COMMIT;