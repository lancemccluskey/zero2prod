-- We dont need the salt column anymore
-- Now that were using the PHC representation
ALTER TABLE
  users DROP COLUMN salt;