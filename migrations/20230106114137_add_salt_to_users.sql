-- Add salt column to users table
ALTER TABLE
  users
ADD
  COLUMN salt TEXT NOT NULL;